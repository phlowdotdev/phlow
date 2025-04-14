use crate::{
    build_engine_async,
    context::Context,
    pipeline::{Pipeline, PipelineError},
    step_worker::NextStep,
    transform::{value_to_pipelines, TransformError},
};
use phlow_sdk::prelude::*;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub enum PhlowError {
    TransformError(TransformError),
    PipelineError(PipelineError),
    PipelineNotFound,
    ParentError,
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
    ) -> Result<Self, PhlowError> {
        let engine = build_engine_async(None);

        let modules = if let Some(modules) = modules {
            modules
        } else {
            Arc::new(Modules::default())
        };
        let pipelines =
            value_to_pipelines(engine, modules, value).map_err(PhlowError::TransformError)?;

        Ok(Self { pipelines })
    }

    pub async fn execute(&self, context: &mut Context) -> Result<Option<Value>, PhlowError> {
        if self.pipelines.is_empty() {
            return Ok(None);
        }

        let mut current_pipeline = self.pipelines.len() - 1;
        let mut current_step = 0;

        loop {
            let pipeline = self
                .pipelines
                .get(&current_pipeline)
                .ok_or(PhlowError::PipelineNotFound)?;

            match pipeline.execute(context, current_step).await {
                Ok(step_output) => match step_output {
                    Some(step_output) => match step_output.next_step {
                        NextStep::Next | NextStep::Stop => {
                            return Ok(step_output.output);
                        }
                        NextStep::Pipeline(id) => {
                            current_pipeline = id;
                            current_step = 0;
                        }
                        NextStep::GoToStep(to) => {
                            current_pipeline = to.pipeline;
                            current_step = to.step;
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
    use phlow_sdk::valu3;
    use valu3::json;

    fn get_original() -> Value {
        json!({
          "steps": [
            {
              "condition": {
                "left": "{{payload.requested}}",
                "right": "{{payload.pre_approved}}",
                "operator": "less_than_or_equal"
              },
              "then": {
                "return": "{{payload.requested}}"
              },
              "else": {
                "steps": [
                  {
                    "condition": {
                      "left": "{{payload.score}}",
                      "right": 0.5,
                      "operator": "greater_than_or_equal"
                    },
                    "then": [
                        {
                            "id": "approved",
                            "payload": {
                                "total": "{{(payload.requested * 0.3) + payload.pre_approved}}"
                            }
                            },
                            {
                            "condition": {
                                "left": "{{steps.approved.total}}",
                                "right": "{{payload.requested}}",
                                "operator": "greater_than_or_equal"
                            },
                            "then": {
                                "return": "{{payload.requested}}"
                            },
                            "else": {
                                "return": "{{steps.approved.total}}"
                            }
                        }
                    ]
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
        let phlow = Phlow::try_from_value(&original, None).unwrap();
        let mut context = Context::from_payload(json!({
            "requested": 10000.00,
            "pre_approved": 10000.00,
            "score": 0.6
        }));

        let result = phlow.execute(&mut context).await.unwrap();

        assert_eq!(result, Some(json!(10000.0)));
    }

    #[tokio::test]
    async fn test_phlow_original_2() {
        let original = get_original();
        let phlow = Phlow::try_from_value(&original, None).unwrap();
        let mut context = Context::from_payload(json!({
            "requested": 10000.00,
            "pre_approved": 500.00,
            "score": 0.6
        }));

        let result = phlow.execute(&mut context).await.unwrap();

        assert_eq!(result, Some(json!(3500.0)));
    }

    #[tokio::test]
    async fn test_phlow_original_3() {
        let original = get_original();
        let phlow = Phlow::try_from_value(&original, None).unwrap();
        let mut context = Context::from_payload(json!({
            "requested": 10000.00,
            "pre_approved": 500.00,
            "score": 0.2
        }));

        let result = phlow.execute(&mut context).await.unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_phlow_original_4() {
        let original = get_original();
        let phlow = Phlow::try_from_value(&original, None).unwrap();
        let mut context = Context::from_payload(json!({
            "requested": 10000.00,
            "pre_approved": 9999.00,
            "score": 0.6
        }));

        let result = phlow.execute(&mut context).await.unwrap();

        assert_eq!(result, Some(json!(10000.0)));
    }
}
