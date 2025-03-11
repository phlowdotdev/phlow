use crate::{
    collector::ContextSender,
    context::Context,
    pipeline::{Pipeline, PipelineError},
    step_worker::NextStep,
    transform::{value_to_pipelines, TransformError},
};
use rhai::Engine;
use std::collections::HashMap;
use valu3::prelude::*;

#[derive(Debug)]
pub enum AnyflowError {
    TransformError(TransformError),
    PipelineError(PipelineError),
    PipelineNotFound,
}

pub type PipelineMap<'a> = HashMap<usize, Pipeline<'a>>;

#[derive(Debug, Default)]
pub struct AnyFlow<'a> {
    pipelines: PipelineMap<'a>,
    params: Option<Value>,
}

impl<'a> AnyFlow<'a> {
    pub fn try_from_value(
        engine: &'a Engine,
        value: &Value,
        params: Option<Value>,
        sender: Option<ContextSender>,
    ) -> Result<Self, AnyflowError> {
        let pipelines =
            value_to_pipelines(&engine, sender, value).map_err(AnyflowError::TransformError)?;

        Ok(Self { pipelines, params })
    }

    pub fn execute_context(&self, context: &mut Context) -> Result<Option<Value>, AnyflowError> {
        if self.pipelines.is_empty() {
            return Ok(None);
        }

        let mut current = self.pipelines.len() - 1;

        loop {
            let pipeline = self
                .pipelines
                .get(&current)
                .ok_or(AnyflowError::PipelineNotFound)?;

            match pipeline.execute(context) {
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
                    return Err(AnyflowError::PipelineError(err));
                }
            }
        }
    }

    pub fn execute(&self) -> Result<Context, AnyflowError> {
        let mut context = Context::new(self.params.clone());
        self.execute_context(&mut context)?;
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::Script;
    use valu3::json;

    fn get_original() -> Value {
        json!({
          "steps": [
            {
              "condition": {
                "left": "params.requested",
                "right": "params.pre_approved",
                "operator": "less_than_or_equal"
              },
              "then": {
                "return": "params.requested"
              },
              "else": {
                "steps": [
                  {
                    "condition": {
                      "left": "params.score",
                      "right": 0.5,
                      "operator": "greater_than_or_equal"
                    }
                  },
                  {
                    "id": "approved",
                    "payload": {
                      "total": "(params.requested * 0.3) + params.pre_approved"
                    }
                  },
                  {
                    "condition": {
                      "left": "steps.approved.total",
                      "right": "params.requested",
                      "operator": "greater_than_or_equal"
                    },
                    "then": {
                      "return": "params.requested"
                    },
                    "else": {
                      "return": "steps.approved.total"
                    }
                  }
                ]
              }
            }
          ]
        })
    }

    #[test]
    fn test_anyflow_original_1() {
        let original = get_original();
        let engine = Script::create_engine();
        let anyflow = AnyFlow::try_from_value(&engine, &original, None, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 10000.00,
            "score": 0.6
        })));

        let result = anyflow.execute_context(&mut context).unwrap();

        assert_eq!(result, Some(json!(10000.0)));
    }

    #[test]
    fn test_anyflow_original_2() {
        let original = get_original();
        let engine = Script::create_engine();
        let anyflow = AnyFlow::try_from_value(&engine, &original, None, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 500.00,
            "score": 0.6
        })));

        let result = anyflow.execute_context(&mut context).unwrap();

        assert_eq!(result, Some(json!(3500.0)));
    }

    #[test]
    fn test_anyflow_original_3() {
        let original = get_original();
        let engine = Script::create_engine();
        let anyflow = AnyFlow::try_from_value(&engine, &original, None, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 500.00,
            "score": 0.2
        })));

        let result = anyflow.execute_context(&mut context).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn test_anyflow_original_4() {
        let original = get_original();
        let engine = Script::create_engine();
        let anyflow = AnyFlow::try_from_value(&engine, &original, None, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 9999.00,
            "score": 0.6
        })));

        let result = anyflow.execute_context(&mut context).unwrap();

        assert_eq!(result, Some(json!(10000.0)));
    }
}
