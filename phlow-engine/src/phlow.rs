use crate::{
    build_engine_async,
    collector::ContextSender,
    context::Context,
    modules::Modules,
    pipeline::{Pipeline, PipelineError},
    step_worker::NextStep,
    transform::{value_to_pipelines, TransformError},
};
use sdk::tracing;
use std::{collections::HashMap, sync::Arc};
use valu3::prelude::*;

#[derive(Debug)]
pub enum PhlowError {
    TransformError(TransformError),
    PipelineError(PipelineError),
    PipelineNotFound,
}

pub type PipelineMap = HashMap<usize, Pipeline>;

#[derive(Debug, Default)]
pub struct Phlow {
    pipelines: PipelineMap,
}

impl Phlow {
    pub fn try_from_value(
        value: &Value,
        modules: Option<Arc<Modules>>,
        trace_sender: Option<ContextSender>,
    ) -> Result<Self, PhlowError> {
        let engine = build_engine_async(None);

        let modules = if let Some(modules) = modules {
            modules
        } else {
            Arc::new(Modules::default())
        };
        let pipelines = value_to_pipelines(engine, modules, trace_sender, value)
            .map_err(PhlowError::TransformError)?;

        Ok(Self { pipelines })
    }

    pub async fn execute(&self, context: &mut Context) -> Result<Option<Value>, PhlowError> {
        if self.pipelines.is_empty() {
            return Ok(None);
        }

        let mut current = self.pipelines.len() - 1;

        loop {
            let pipeline = self
                .pipelines
                .get(&current)
                .ok_or(PhlowError::PipelineNotFound)?;

            match pipeline.execute(context).await {
                Ok(step_output) => match step_output {
                    Some(step_output) => match step_output.next_step {
                        NextStep::Next | NextStep::Stop => {
                            return Ok(step_output.output);
                        }
                        NextStep::Pipeline(id) => {
                            current = id;
                        }
                    },
                    None => {
                        return Ok(None);
                    }
                },
                Err(err) => {
                    return Err(PhlowError::PipelineError(err));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{collector::Step, id::ID};
    use crossbeam::channel;
    use valu3::json;

    fn get_original() -> Value {
        json!({
          "steps": [
            {
              "condition": {
                "left": "{{params.requested}}",
                "right": "{{params.pre_approved}}",
                "operator": "less_than_or_equal"
              },
              "then": {
                "return": "{{params.requested}}"
              },
              "else": {
                "steps": [
                  {
                    "condition": {
                      "left": "{{params.score}}",
                      "right": 0.5,
                      "operator": "greater_than_or_equal"
                    }
                  },
                  {
                    "id": "approved",
                    "payload": {
                      "total": "{{(params.requested * 0.3) + params.pre_approved}}"
                    }
                  },
                  {
                    "condition": {
                      "left": "{{steps.approved.total}}",
                      "right": "{{params.requested}}",
                      "operator": "greater_than_or_equal"
                    },
                    "then": {
                      "return": "{{params.requested}}"
                    },
                    "else": {
                      "return": "{{steps.approved.total}}"
                    }
                  }
                ]
              }
            }
          ]
        })
    }

    #[tokio::test]
    async fn test_phlow_original_1() {
        let original = get_original();
        let phlow = Phlow::try_from_value(&original, None, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 10000.00,
            "score": 0.6
        })));

        let result = phlow.execute(&mut context).await.unwrap();

        assert_eq!(result, Some(json!(10000.0)));
    }

    #[tokio::test]
    async fn test_phlow_original_2() {
        let original = get_original();
        let phlow = Phlow::try_from_value(&original, None, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 500.00,
            "score": 0.6
        })));

        let result = phlow.execute(&mut context).await.unwrap();

        assert_eq!(result, Some(json!(3500.0)));
    }

    #[tokio::test]
    async fn test_phlow_original_3() {
        let original = get_original();
        let phlow = Phlow::try_from_value(&original, None, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 500.00,
            "score": 0.2
        })));

        let result = phlow.execute(&mut context).await.unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_phlow_original_4() {
        let original = get_original();
        let phlow = Phlow::try_from_value(&original, None, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 9999.00,
            "score": 0.6
        })));

        let result = phlow.execute(&mut context).await.unwrap();

        assert_eq!(result, Some(json!(10000.0)));
    }

    #[tokio::test]
    async fn test_phlow_channel() {
        let original = get_original();
        let (sender, receiver) = channel::unbounded::<Step>();
        let phlow = Phlow::try_from_value(&original, None, Some(sender.clone())).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 9999.00,
            "score": 0.6
        })));

        let target = vec![
            Step {
                id: ID::new(),
                label: None,
                module: None,
                input: None,
                condition: Some("{{params.requested <= params.pre_approved}}".to_value()),
                payload: None,
                return_case: None,
            },
            Step {
                id: ID::new(),
                label: None,
                module: None,
                input: None,
                condition: Some("{{params.score >= 0.5}}".to_value()),
                payload: None,
                return_case: None,
            },
            Step {
                id: ID::from("approved"),
                label: None,
                module: None,
                input: None,
                condition: None,
                payload: Some(json!({
                    "total": 12999.0
                })),
                return_case: None,
            },
        ];

        phlow.execute(&mut context).await.unwrap();

        let mut result: Vec<Step> = Vec::new();

        for message in receiver.iter() {
            result.push(message);

            if result.len() == 3 {
                break;
            }
        }

        assert_eq!(result, target);
    }
}
