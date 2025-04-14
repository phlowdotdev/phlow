use phlow_sdk::{prelude::*, valu3};
use rhai::Engine;
use std::collections::HashMap;
use std::sync::Arc;
use valu3::{traits::ToValueBehavior, value::Value};

use crate::{
    phlow::PipelineMap,
    pipeline::Pipeline,
    step_worker::{StepReference, StepWorker, StepWorkerError},
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
                            StepReference {
                                pipeline: pipeline_index,
                                step: step_index,
                            },
                        );
                    }
                }
            }
        }
    }

    let parents = map_parents(&pipelines_raw);

    let mut pipelines = Vec::new();

    for (pipeline_index, pipeline_value) in pipelines_raw.iter().enumerate() {
        if let Value::Array(arr) = pipeline_value {
            let mut new_steps = Vec::new();

            for (step_index, step_value) in arr.into_iter().enumerate() {
                if let Value::Object(step) = step_value {
                    let mut new_step = step.clone();

                    if let Some(to) = step.get("to") {
                        if let Some(go_to_step) = go_to_step_id.get(to.to_string().as_str()) {
                            new_step.insert("to".to_string(), go_to_step.to_value());
                        }
                    } else {
                        if let Some(target) = parents.get(&StepReference {
                            pipeline: pipeline_index,
                            step: step_index,
                        }) {
                            let next_step = get_next_step(&pipelines_raw, target);
                            new_step.insert("to".to_string(), next_step.to_value());
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

fn get_next_step(pipelines: &Vec<Value>, target: &StepReference) -> StepReference {
    if let Value::Array(arr) = &pipelines[target.pipeline] {
        let next_step_index = target.step + 1;
        if arr.get(next_step_index).is_some() {
            return StepReference {
                pipeline: target.pipeline,
                step: next_step_index,
            };
        }
    }

    return StepReference {
        pipeline: target.pipeline,
        step: target.step,
    };
}

fn map_parents(pipelines: &Vec<Value>) -> HashMap<StepReference, StepReference> {
    let mut parents = HashMap::new();

    for (pipeline_index, steps) in pipelines.iter().enumerate() {
        if let Value::Array(arr) = steps {
            for (step_index, step) in arr.into_iter().enumerate() {
                if let Value::Object(step) = step {
                    let to = if let Some(to) = step.get("to") {
                        let to_pipeline = to
                            .get("pipeline")
                            .expect("pipeline not found")
                            .to_u64()
                            .unwrap() as usize;
                        let to_step =
                            to.get("step").expect("step not found").to_u64().unwrap() as usize;
                        Some(StepReference {
                            pipeline: to_pipeline,
                            step: to_step,
                        })
                    } else {
                        None
                    };

                    if let Some(then_case) = step.get("then") {
                        let then_value =
                            then_case.to_u64().expect("then value should be u64") as usize;
                        parents.insert(
                            StepReference {
                                pipeline: then_value,
                                step: 0,
                            },
                            to.clone().unwrap_or(StepReference {
                                pipeline: pipeline_index,
                                step: step_index,
                            }),
                        );
                    }

                    if let Some(else_case) = step.get("else") {
                        let else_value =
                            else_case.to_u64().expect("else value should be u64") as usize;

                        parents.insert(
                            StepReference {
                                pipeline: else_value,
                                step: 0,
                            },
                            to.unwrap_or(StepReference {
                                pipeline: pipeline_index,
                                step: step_index,
                            }),
                        );
                    }
                }
            }
        }
    }

    parents
}

fn value_to_structs(
    engine: Arc<Engine>,
    modules: Arc<Modules>,
    pipelines_raw: &Vec<Value>,
) -> Result<PipelineMap, TransformError> {
    let pipelines_with_to = resolve_go_to_step(pipelines_raw);

    println!(
        "pipelines_with_to: {}",
        pipelines_with_to.to_value().to_json(JsonMode::Indented)
    );

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
