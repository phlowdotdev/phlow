use phlow_sdk::{prelude::*, valu3};
use rhai::Engine;
use std::{collections::HashMap, sync::Arc};
use valu3::{traits::ToValueBehavior, value::Value};

use crate::{
    phlow::PipelineMap,
    pipeline::Pipeline,
    step_worker::{StepWorker, StepWorkerError},
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
    add_parents_to_map(&mut map);

    println!("map: {}", map.to_value().to_json(JsonMode::Indented));
    value_to_structs(engine, modules, &map)
}

pub fn add_parents_to_map(map: &mut Vec<Value>) {
    let mut relations: Vec<(usize, Vec<usize>)> = Vec::new();

    fn collect_relations(
        map: &Vec<Value>,
        map_index: usize,
        current_path: Vec<usize>,
        relations: &mut Vec<(usize, Vec<usize>)>,
    ) {
        if let Some(Value::Array(steps)) = map.get(map_index) {
            for (i, step) in steps.into_iter().enumerate() {
                if let Value::Object(step_obj) = step {
                    let mut caller_path = current_path.clone();
                    caller_path.push(i); // adiciona o índice local

                    for key in ["then", "else"] {
                        if let Some(Value::Number(child_idx)) = step_obj.get(key) {
                            if let Some(child_index) = child_idx.to_u64() {
                                relations.push((child_index as usize, caller_path.clone()));
                                collect_relations(
                                    map,
                                    child_index as usize,
                                    caller_path.clone(),
                                    relations,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // inicia para todos os pipelines no nível mais alto (por segurança)
    for i in 0..map.len() {
        collect_relations(map, i, vec![i], &mut relations);
    }

    // aplica os paths como Value::Array
    for (child_index, parent_path) in relations {
        if let Some(Value::Array(child_steps)) = map.get_mut(child_index) {
            for step in child_steps {
                if let Value::Object(step_obj) = step {
                    let array = parent_path
                        .iter()
                        .map(|i| (*i).to_value())
                        .collect::<Vec<_>>();
                    step_obj.insert("parent".to_string(), array.to_value());
                }
            }
        }
    }
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

    let json = (map.len() - 1).to_value().to_json(JsonMode::Inline);
    match Value::json_to_value(&json) {
        Ok(value) => value,
        Err(err) => {
            error!("Error parsing json: {:?}", err);
            Value::Null
        }
    }
}

fn value_to_structs(
    engine: Arc<Engine>,
    modules: Arc<Modules>,

    map: &Vec<Value>,
) -> Result<PipelineMap, TransformError> {
    let mut pipelines = HashMap::new();

    for (pipeline_id, steps) in map.iter().enumerate() {
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
