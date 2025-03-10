use crate::{
    id::ID,
    pipeline::{Pipeline, PipelineError},
    step_worker::NextStep,
    transform::{json_to_pipelines, value_to_pipelines, TransformError},
};
use serde::Serialize;
use std::collections::HashMap;
use valu3::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Context {
    pub(crate) params: Option<Value>,
    pub(crate) steps: HashMap<ID, Value>,
}

impl Context {
    pub fn new(params: Option<Value>) -> Self {
        Self {
            params,
            steps: HashMap::new(),
        }
    }

    pub fn add_step_output(&mut self, id: ID, output: Value) {
        self.steps.insert(id, output);
    }

    pub fn get_step_output(&self, id: &ID) -> Option<&Value> {
        self.steps.get(&id)
    }
}
#[derive(Debug)]
pub enum Error {
    TransformError(TransformError),
    PipelineError(PipelineError),
    PipelineNotFound,
}

pub type PipelineMap = HashMap<usize, Pipeline>;

#[derive(Debug, Default)]
struct V8 {
    pipelines: PipelineMap,
    params: Option<Value>,
}

impl V8 {
    pub fn execute_context(&self, context: &mut Context) -> Result<Option<Value>, Error> {
        if self.pipelines.is_empty() {
            return Ok(None);
        }

        let mut current = self.pipelines.len() - 1;

        loop {
            let pipeline = self
                .pipelines
                .get(&current)
                .ok_or(Error::PipelineNotFound)?;

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
                    return Err(Error::PipelineError(err));
                }
            }
        }
    }

    pub fn execute(&self) -> Result<Context, Error> {
        let mut context = Context::new(self.params.clone());
        self.execute_context(&mut context)?;
        Ok(context)
    }
}

impl TryFrom<&String> for V8 {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let (pipelines, params) = json_to_pipelines(value).map_err(Error::TransformError)?;

        Ok(Self { pipelines, params })
    }
}

impl TryFrom<&Value> for V8 {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let pipelines = value_to_pipelines(&value).map_err(Error::TransformError)?;

        Ok(Self {
            pipelines,
            params: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let v8 = V8::try_from(&original).unwrap();
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
        let v8 = V8::try_from(&original).unwrap();
        let mut context = Context::new(Some(json!({
            "requested": 10000.00,
            "pre_approved": 500.00,
            "score": 0.6
        })));

        let result = v8.execute_context(&mut context).unwrap();

        assert_eq!(result, Some(json!(10000.0)));
    }
}
