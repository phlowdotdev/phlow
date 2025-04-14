use phlow_sdk::{prelude::*, valu3};
use rhai::Engine;
use std::collections::HashMap;
use std::sync::Arc;
use valu3::{traits::ToValueBehavior, value::Value};

use crate::{
    phlow::PipelineMap,
    pipeline::Pipeline,
    step_worker::{GoToStep, StepWorker, StepWorkerError},
};

#[derive(Debug)]
pub enum TransformError {
    InnerStepError(StepWorkerError),
    Parser(valu3::Error),
}

pub(crate) fn value_to_pipelines(
    engine: Arc<Engine>,
    modules: Arc<Modules>,
    input: &Value,
) -> Result<PipelineMap, TransformError> {
    let mut map = Vec::new();

    process_raw_steps(input, &mut map);
    value_to_structs(engine, modules, &map)
}

pub(crate) fn process_raw_steps(input: &Value, map: &mut Vec<Value>) -> Value {
    if let Value::Object(pipeline) = input {
        let mut new_pipeline = pipeline.clone();

        new_pipeline.remove(&"steps");

        // Tratamento para THEN
        if let Some(then) = pipeline.get("then") {
            let then_value = process_raw_steps(then, map);
            new_pipeline.insert("then".to_string(), then_value);
        }

        // Tratamento para ELSE
        if let Some(els) = pipeline.get("else") {
            let else_value = process_raw_steps(els, map);
            new_pipeline.insert("else".to_string(), else_value);
        }

        let mut new_steps = if new_pipeline.is_empty() {
            vec![]
        } else {
            vec![new_pipeline.to_value()]
        };

        if let Some(steps) = pipeline.get("steps") {
            if let Value::Array(steps) = steps {
                for step in steps {
                    let mut new_step = step.clone();

                    if let Some(then) = step.get("then") {
                        new_step.insert("then".to_string(), process_raw_steps(then, map));
                    }

                    if let Some(els) = step.get("else") {
                        new_step.insert("else".to_string(), process_raw_steps(els, map));
                    }

                    new_steps.push(new_step);
                }
            }
        }

        map.push(new_steps.to_value());
    } else if let Value::Array(pipeline) = input {
        let mut new_steps = Vec::new();

        for step in pipeline {
            if let Value::Object(step) = step {
                let mut new_step = step.clone();

                if let Some(then) = step.get("then") {
                    new_step.insert("then".to_string(), process_raw_steps(then, map));
                }

                if let Some(els) = step.get("else") {
                    new_step.insert("else".to_string(), process_raw_steps(els, map));
                }

                new_steps.push(new_step);
            }
        }

        map.push(new_steps.to_value());
    }

    (map.len() - 1).to_value()
}

fn resolve_go_to_step(pipelines_raw: &Vec<Value>) -> Vec<Value> {
    let mut go_to_step_id = HashMap::new();

    for (pipeline_index, steps) in pipelines_raw.iter().enumerate() {
        if let Value::Array(arr) = steps {
            for (step_index, step) in arr.into_iter().enumerate() {
                if let Value::Object(step) = step {
                    if let Some(id) = step.get("id") {
                        go_to_step_id.insert(
                            id.to_string(),
                            GoToStep {
                                pipeline: pipeline_index,
                                step: step_index,
                            },
                        );
                    }
                }
            }
        }
    }

    let mut pipelines = Vec::new();

    for pipeline in pipelines_raw.iter() {
        if let Value::Array(arr) = pipeline {
            let mut new_steps = Vec::new();

            for step in arr.into_iter() {
                if let Value::Object(step) = step {
                    let mut new_step = step.clone();

                    if let Some(to) = step.get("to") {
                        if let Some(go_to_step) = go_to_step_id.get(to.to_string().as_str()) {
                            new_step.insert("to".to_string(), go_to_step.to_value());
                        }
                    }

                    new_steps.push(new_step.to_value());
                }
            }

            pipelines.push(new_steps.to_value());
        }
    }

    pipelines
}

fn value_to_structs(
    engine: Arc<Engine>,
    modules: Arc<Modules>,
    pipelines_raw: &Vec<Value>,
) -> Result<PipelineMap, TransformError> {
    let pipelines_with_to = resolve_go_to_step(pipelines_raw);

    let mut pipelines = HashMap::new();

    for (pipeline_id, steps) in pipelines_with_to.iter().enumerate() {
        if let Value::Array(arr) = steps {
            let mut steps = Vec::new();

            for step in arr.into_iter() {
                let step_worker = StepWorker::try_from_value(engine.clone(), modules.clone(), step)
                    .map_err(TransformError::InnerStepError)?;
                steps.push(step_worker);
            }

            pipelines.insert(pipeline_id, Pipeline { steps });
        }
    }

    Ok(pipelines)
}

#[cfg(test)]
mod test {
    use super::*;
    use valu3::{json, traits::ToValueBehavior};

    #[test]
    fn test_transform_value() {
        let mut map = Vec::new();
        let original = json!({
          "steps": [
            {
              "condition": {
                "left": "payload.requested",
                "right": "payload.pre-approved",
                "operator": "less_than"
              },
              "then": {
                "payload": "payload.requested"
              },
              "else": {
                "steps": [
                  {
                    "condition": {
                      "left": "payload.score",
                      "right": 0.5,
                      "operator": "greater_than"
                    }
                  },
                  {
                    "id": "approved",
                    "payload": {
                      "total": "(payload.requested * 0.3) + payload.pre-approved"
                    }
                  },
                  {
                    "condition": {
                      "left": "steps.approved.total",
                      "right": "payload.requested",
                      "operator": "greater_than"
                    },
                    "then": {
                      "return": "payload.requested"
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
        let target = json!([[{"payload": "payload.requested"}],[{"return": "payload.requested"}],[{"return": "steps.approved.total"}],[{"condition": {"left": "payload.score","operator": "greater_than","right": 0.5}},{"id": "approved","payload": {"total": "(payload.requested * 0.3) + payload.pre-approved"}},{"else": 2,"condition": {"operator": "greater_than","right": "payload.requested","left": "steps.approved.total"},"then": 1}],[{"condition": {"right": "payload.pre-approved","left": "payload.requested","operator": "less_than"},"else": 3,"then": 0}]]);

        process_raw_steps(&original, &mut map);

        assert_eq!(map.to_value(), target);
    }

    #[test]
    fn test_transform_value_array() {
        let mut map = Vec::new();
        let original = json!({
          "steps": [
            {
              "condition": {
                "left": "payload.requested",
                "right": "payload.pre-approved",
                "operator": "less_than"
              },
              "then": {
                "payload": "payload.requested"
              },
              "else": [
                {
                  "condition": {
                    "left": "payload.score",
                    "right": 0.5,
                    "operator": "greater_than"
                  }
                },
                {
                  "id": "approved",
                  "payload": {
                    "total": "(payload.requested * 0.3) + payload.pre-approved"
                  }
                },
                {
                  "condition": {
                    "left": "steps.approved.total",
                    "right": "payload.requested",
                    "operator": "greater_than"
                  },
                  "then": {
                    "return": "payload.requested"
                  },
                  "else": {
                    "return": "steps.approved.total"
                  }
                }
              ]
            }
          ]
        });
        let target = json!([[{"payload": "payload.requested"}],[{"return": "payload.requested"}],[{"return": "steps.approved.total"}],[{"condition": {"left": "payload.score","operator": "greater_than","right": 0.5}},{"id": "approved","payload": {"total": "(payload.requested * 0.3) + payload.pre-approved"}},{"else": 2,"condition": {"operator": "greater_than","right": "payload.requested","left": "steps.approved.total"},"then": 1}],[{"condition": {"right": "payload.pre-approved","left": "payload.requested","operator": "less_than"},"else": 3,"then": 0}]]);

        process_raw_steps(&original, &mut map);

        assert_eq!(map.to_value(), target);
    }
}
