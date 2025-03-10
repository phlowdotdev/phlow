use crate::{
    id::ID,
    pipeline::{Pipeline, PipelineError},
    step_worker::NextStep,
    transform::{json_to_pipelines, TransformError},
};
use serde::Serialize;
use std::collections::HashMap;
use valu3::value::Value;

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

struct V8 {
    pipelines: PipelineMap,
    main: usize,
    params: Option<Value>,
}

impl V8 {
    pub fn execute_context(&self, context: &mut Context) -> Result<(), Error> {
        let mut current = self.main;
        loop {
            let pipeline = self
                .pipelines
                .get(&current)
                .ok_or(Error::PipelineNotFound)?;

            match pipeline.execute(context) {
                Ok(next_step) => match next_step {
                    Some(next) => match next {
                        NextStep::Pipeline(id) => {
                            current = id as usize;
                        }
                        _ => {
                            break Ok(());
                        }
                    },
                    None => {
                        break Ok(());
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

        Ok(Self {
            pipelines,
            main: 0,
            params,
        })
    }
}

#[cfg(test)]
mod tests {
    use valu3::json;
    use valu3::prelude::*;
    use super::*;
    use std::fs;

    #[test]
    fn test_v8() {
        let original = json!({
            "steps": [
              {
                "condition": {
                  "left": "params.requested",
                  "right": "params.pre-approved",
                  "operator": "less_than"
                },
                "then": {
                  "payload": "params.requested"
                },
                "else": {
                  "steps": [
                    {
                      "condition": {
                        "left": "params.score",
                        "right": 0.5,
                        "operator": "greater_than"
                      }
                    },
                    {
                      "id": "approved",
                      "payload": {
                        "total": "(params.requested * 0.3) + params.pre-approved"
                      }
                    },
                    {
                      "condition": {
                        "left": "steps.approved.total",
                        "right": "params.requested",
                        "operator": "greater_than"
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
          });
          
        let v8 = V8::try_from(&original).unwrap();
        let context = {
            let mut map = HashMap::new();
            map.insert("name".to_string(), Value::String("John".to_string()));
            Value::Object(map)
        }
        let context = v8.execute_context().unwrap();

        assert_eq!()
    }
}
