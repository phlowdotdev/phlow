use std::collections::HashMap;
use valu3::value::Value;

use crate::{
    pipeline::Pipeline,
    step_worker::{StepWorker, StepWorkerError, ID},
};

#[derive(Debug)]
pub enum TransformError {
    InnerStepError(StepWorkerError),
}

pub fn transform_json(input: &Value) -> Result<HashMap<ID, Pipeline>, TransformError> {
    let mut id_counter = 0;
    let mut map = HashMap::new();

    process_raw_steps(input, &mut id_counter, &mut map);

    value_to_structs(&Value::from(map))
}

fn process_raw_steps(
    input: &Value,
    id_counter: &mut usize,
    map: &mut HashMap<String, Value>,
) -> Value {
    let key = format!("pipeline_id_{}", *id_counter);
    *id_counter += 1;

    let mut new_array = Vec::new();

    if let Value::Array(arr) = input {
        for item in arr.into_iter() {
            if let Value::Object(obj) = item {
                let mut new_obj = obj.clone();

                // Processa `condition` se existir
                if let Some(condition) = obj.get("condition") {
                    if let Value::Object(cond_obj) = condition {
                        let mut new_cond = cond_obj.clone();

                        // Substitui `then` e `else` por novos IDs
                        if let Some(then) = cond_obj.get("then") {
                            new_cond.insert(
                                "then".to_string(),
                                process_raw_steps(then, id_counter, map),
                            );
                        }
                        if let Some(els) = cond_obj.get("else") {
                            new_cond.insert(
                                "else".to_string(),
                                process_raw_steps(els, id_counter, map),
                            );
                        }

                        new_obj.insert("condition".to_string(), Value::Object(new_cond));
                    }
                }

                // Adiciona o objeto processado à nova array
                new_array.push(Value::Object(new_obj));
            } else {
                new_array.push(item.clone());
            }
        }
    }

    map.insert(key.clone(), Value::from(new_array));
    Value::from(key)
}

fn value_to_structs(value: &Value) -> Result<HashMap<ID, Pipeline>, TransformError> {
    let mut pipelines = HashMap::new();

    if let Value::Object(obj) = value {
        for (key, val) in obj.iter() {
            if let Value::Array(arr) = val {
                let mut steps = Vec::new();

                for item in arr {
                    let step_worker =
                        StepWorker::try_from(value).map_err(TransformError::InnerStepError)?;
                    steps.push(step_worker);
                }

                pipelines.insert(
                    key.as_string_b().as_string(),
                    Pipeline::new(key.as_string_b().as_string(), steps),
                );
            }
        }
    }

    Ok(pipelines)
}

#[cfg(test)]
mod test {
    use std::default;

    use crate::{
        condition::{Condition, Operator},
        payload::Payload,
    };

    use super::*;
    use valu3::value::Value as Valu3Value;

    #[test]
    fn test_transform_json() {
        let original = Valu3Value::json_to_value(
            r#"
            [
          {
            "name": "Start"
          },
          {
            "id": "step1",
            "condition": {
              "left": "context.credit",
              "right": "context.credit_used",
              "condition": "greater_than",
              "then": [
                {
                  "payload": {
                    "score": "{{context.credit - context.credit_used}}"
                  },
                  "steps": [
                    {
                      "condition": {
                        "left": "{{steps.step1.score}}",
                        "right": "10",
                        "condition": "greater_than"
                      }
                    },
                    {
                      "condition": {
                        "left": "{{steps.step1.score}}",
                        "right": "500",
                        "condition": "greater_than"
                      }
                    },
                    {
                      "condition": {
                        "left": "{{steps.step1.score}}",
                        "right": "100000",
                        "condition": "less_than"
                      }
                    },
                    {
                      "then": {
                        "name": "Credit avaliable",
                        "condition": {
                          "left": "{{steps.step1.score}}",
                          "right": "500",
                          "condition": "equal"
                        },
                        "then": {
                          "return": true
                        }
                      }
                    }
                  ]
                }
              ],
              "else": [
                {
                  "score": "{{0}}"
                }
              ]
            }
          },
          {
            "condition": {
              "left": "{{steps.step1.score}}",
              "right": "500",
              "condition": "greater_than",
              "then": [
                {
                  "name": "Credit avaliable",
                  "payload": {
                    "resul": "{{true}}"
                  }
                }
              ],
              "else": [
                {
                  "name": "Credit not avaliable",
                  "payload": {
                    "score": "{{false}}"
                  }
                }
              ]
            }
          },
          {
            "name": "End"
          }
        ]
        "#,
        )
        .unwrap();

        let expected = vec![
            Pipeline::new(
                "pipeline_id_0".to_string(),
                vec![
                    StepWorker {
                        name: Some("Start".to_string()),
                        ..default::Default::default()
                    },
                    StepWorker {
                        id: Some(ID::from("step1")),
                        condition: Some(Condition {
                            left: Payload {
                                script: "context.credit".to_string(),
                            },
                            right: Payload {
                                script: "context.credit_used".to_string(),
                            },
                            operator: Operator::GreaterThan,
                        }),
                        then_case: Some(ID::from("pipeline_id_1")),
                        else_case: Some(ID::from("pipeline_id_2")),
                        ..default::Default::default()
                    },
                    StepWorker {
                        condition: Some(Condition {
                            left: Payload {
                                script: "{{steps.step1.score}}".to_string(),
                            },
                            right: Payload {
                                script: "500".to_string(),
                            },
                            operator: Operator::GreaterThan,
                        }),
                        then_case: Some(ID::from("pipeline_id_3")),
                        else_case: Some(ID::from("pipeline_id_4")),
                        ..default::Default::default()
                    },
                ],
            ),
            Pipeline::new("pipeline_id_1".to_string(), vec![]),
            Pipeline::new("pipeline_id_2".to_string(), vec![]),
            Pipeline::new("pipeline_id_3".to_string(), vec![]),
        ];

        let result = transform_json(&original).unwrap();
    }
}
