use crate::{
    context::Context,
    pipeline::{Pipeline, PipelineError},
    step_worker::NextStep,
    transform::{value_to_pipelines, TransformError},
};
use rhai::Engine;
use std::collections::HashMap;
use valu3::prelude::*;

#[derive(Debug)]
pub enum V8Error {
    TransformError(TransformError),
    PipelineError(PipelineError),
    PipelineNotFound,
}

pub type PipelineMap<'a> = HashMap<usize, Pipeline<'a>>;

#[derive(Debug, Default)]
pub struct V8<'a> {
    pipelines: PipelineMap<'a>,
    params: Option<Value>,
}

impl<'a> V8<'a> {
    pub fn try_from_value(
        engine: &'a Engine,
        value: &Value,
        params: Option<Value>,
    ) -> Result<Self, V8Error> {
        let pipelines = value_to_pipelines(&engine, value).map_err(V8Error::TransformError)?;

        Ok(Self { pipelines, params })
    }

    pub fn execute_context(&self, context: &mut Context) -> Result<Option<Value>, V8Error> {
        if self.pipelines.is_empty() {
            return Ok(None);
        }

        let mut current = self.pipelines.len() - 1;

        loop {
            let pipeline = self
                .pipelines
                .get(&current)
                .ok_or(V8Error::PipelineNotFound)?;

            match pipeline.execute(context) {
                Ok(step_output) => match step_output {
                    Some(step_output) => match step_output.next_step {
                        NextStep::Next | NextStep::Stop => {
                            return Ok(step_output.payload);
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
                    return Err(V8Error::PipelineError(err));
                }
            }
        }
    }

    pub fn execute(&self) -> Result<Context, V8Error> {
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
    fn test_v8_original_1() {
        let original = get_original();
        let engine = Script::create_engine();
        let v8 = V8::try_from_value(&engine, &original, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 10000.00,
            "score": 0.6
        })));

        let result = v8.execute_context(&mut context).unwrap();

        assert_eq!(result, Some(json!(10000.0)));
    }

    #[test]
    fn test_v8_original_2() {
        let original = get_original();
        let engine = Script::create_engine();
        let v8 = V8::try_from_value(&engine, &original, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 500.00,
            "score": 0.6
        })));

        let result = v8.execute_context(&mut context).unwrap();

        assert_eq!(result, Some(json!(3500.0)));
    }

    #[test]
    fn test_v8_original_3() {
        let original = get_original();
        let engine = Script::create_engine();
        let v8 = V8::try_from_value(&engine, &original, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 500.00,
            "score": 0.2
        })));

        let result = v8.execute_context(&mut context).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn test_v8_original_4() {
        let original = get_original();
        let engine = Script::create_engine();
        let v8 = V8::try_from_value(&engine, &original, None).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 9999.00,
            "score": 0.6
        })));

        let result = v8.execute_context(&mut context).unwrap();

        assert_eq!(result, Some(json!(10000.0)));
    }
}
