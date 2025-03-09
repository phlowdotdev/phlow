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

pub(crate) fn transform_json(input: &Value) -> Result<HashMap<ID, Pipeline>, TransformError> {
    let mut id_counter = 0;
    let mut map = HashMap::new();

    process_raw_steps(input, &mut id_counter, &mut map);

    value_to_structs(&map)
}

pub(crate) fn process_raw_steps(
    input: &Value,
    id_counter: &mut usize,
    map: &mut HashMap<String, Value>,
) -> Value {
    let key = format!("pipeline_id_{}", *id_counter);
    *id_counter += 1;

    let mut new_array = Vec::new();

    if let Value::Object(main_obj) = input {
        let main_steps = match main_obj.get("steps") {
            Some(Value::Array(arr)) => arr,
            _ => return Value::from(new_array),
        };

        for item in main_steps.into_iter() {
            if let Value::Object(obj) = item {
                let mut new_obj = obj.clone();

                if let Some(condition) = obj.get("condition") {
                    if let Value::Object(cond_obj) = condition {
                        let mut new_cond = cond_obj.clone();

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

                new_array.push(Value::Object(new_obj));
            } else {
                new_array.push(item.clone());
            }
        }
    }

    map.insert(key.clone(), Value::from(new_array));
    Value::from(key)
}

fn value_to_structs(map: &HashMap<String, Value>) -> Result<HashMap<ID, Pipeline>, TransformError> {
    let mut pipelines = HashMap::new();

    for (key, val) in map.iter() {
        if let Value::Array(arr) = val {
            let mut steps = Vec::new();

            for item in arr.into_iter() {
                let step_worker =
                    StepWorker::try_from(item).map_err(TransformError::InnerStepError)?;
                steps.push(step_worker);
            }

            pipelines.insert(
                ID::from(key.clone()),
                Pipeline::new(ID::from(key.clone()), steps),
            );
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
    use valu3::{traits::ToValueBehavior, value::Value as Valu3Value};

    const ORIGINAL: &str = r#"
    {
      "steps": [
        {
          "echo": "Start"
        },
        {
          "id": "step1",
          "condition": {
            "left": "context.credit",
            "right": "context.credit_used",
            "operator": "greater_than"
          },
          "then": {
            "payload": {
              "score": "context.credit - context.credit_used"
            },
            "steps": [
              {
                "condition": {
                  "left": "steps.step1.score",
                  "right": "10",
                  "operator": "greater_than"
                }
              },
              {
                "condition": {
                  "left": "steps.step1.score",
                  "right": "500",
                  "operator": "greater_than"
                }
              },
              {
                "condition": {
                  "left": "steps.step1.score",
                  "right": "100000",
                  "operator": "less_than"
                }
              },
              {
                "then": {
                  "condition": {
                    "left": "steps.step1.score",
                    "right": "500",
                    "operator": "equal"
                  },
                  "then": {
                    "return": true
                  }
                }
              }
            ]
          },
          "else": {
            "steps": [
              {
                "score": "{{0}}"
              }
            ]
          }
        },
        {
          "condition": {
            "left": "steps.step1.score",
            "right": "500",
            "operator": "greater_than",
            "then": {
              "steps": [
                {
                  "echo": "Credit avaliable",
                  "payload": {
                    "resul": "true"
                  }
                }
              ]
            },
            "else": {
              "steps": [
                {
                  "echo": "Credit not avaliable",
                  "payload": {
                    "score": "false"
                  }
                }
              ]
            }
          }
        },
        {
          "echo": "End"
        }
      ]
    }
  "#;

    #[test]
    fn test_transform_value() {
        let mut id_counter = 0;
        let mut map = HashMap::new();

        process_raw_steps(
            &Valu3Value::json_to_value(ORIGINAL).unwrap(),
            &mut id_counter,
            &mut map,
        );

        let transfomed = Valu3Value::json_to_value(
            r#"
                {
            "pipeline_id_0": [
              {
                "echo": "Start"
              },
              {
                "id": "step1",
                "condition": {
                  "left": "context.credit",
                  "right": "context.credit_used",
                  "condition": "greater_than",
                  "then": "pipeline_id_1",
                  "else": "pipeline_id_2"
                }
              },
              {
                "condition": {
                  "left": "steps.step1.score",
                  "right": "500",
                  "condition": "greater_than",
                  "then": "pipeline_id_3",
                  "else": "pipeline_id_4"
                }
              },
              {
                "echo": "End"
              }
            ],
            "pipeline_id_1": [
              [
                {
                  "payload": {
                    "score": "context.credit - context.credit_used"
                  },
                  "steps": [
                    {
                      "condition": {
                        "left": "steps.step1.score",
                        "right": "10",
                        "condition": "greater_than"
                      }
                    },
                    {
                      "condition": {
                        "left": "steps.step1.score",
                        "right": "500",
                        "condition": "greater_than"
                      }
                    },
                    {
                      "condition": {
                        "left": "steps.step1.score",
                        "right": "100000",
                        "condition": "less_than"
                      }
                    },
                    {
                      "then": "pipeline_id_5"
                    }
                  ]
                }
              ]
            ],
            "pipeline_id_2": [
              {
                "score": "{{0}}"
              }
            ],
            "pipeline_id_3": [
              {
                "echo": "Credit avaliable",
                "payload": {
                  "resul": "{{true}}"
                }
              }
            ],
            "pipeline_id_4": [
              {
                "echo": "Credit not avaliable",
                "payload": {
                  "score": "{{false}}"
                }
              }
            ],
            "pipeline_id_5": [
              {
                "echo": "Credit avaliable",
                "condition": {
                  "left": "steps.step1.score",
                  "right": "500",
                  "condition": "equal"
                },
                "then": {
                  "return": true
                }
              }
            ]
          }
                "#,
        )
        .unwrap();

        println!(
            "{:?}",
            map.to_value().to_json(valu3::prelude::JsonMode::Inline)
        );

        assert_eq!(map.to_value(), transfomed);
    }

    #[test]
    fn test_transform_struct() {
        let expected = {
            let mut map = HashMap::new();

            map.insert(
                ID::from("pipeline_id_0"),
                Pipeline::new(
                    ID::from("pipeline_id_0"),
                    vec![
                        StepWorker {
                            name: Some("Start".to_string()),
                            ..default::Default::default()
                        },
                        StepWorker {
                            id: Some(ID::from("step1")),
                            condition: Some(Condition {
                                left: Payload::from("context.credit"),
                                right: Payload::from("context.credit_used"),
                                operator: Operator::GreaterThan,
                            }),
                            then_case: Some(ID::from("pipeline_id_1")),
                            else_case: Some(ID::from("pipeline_id_2")),
                            ..default::Default::default()
                        },
                        StepWorker {
                            condition: Some(Condition {
                                left: Payload::from("steps.step1.score"),
                                right: Payload::from("500"),
                                operator: Operator::GreaterThan,
                            }),
                            then_case: Some(ID::from("pipeline_id_3")),
                            else_case: Some(ID::from("pipeline_id_4")),
                            ..default::Default::default()
                        },
                    ],
                ),
            );
            map.insert(
                ID::from("pipeline_id_1"),
                Pipeline::new(
                    ID::from("pipeline_id_1"),
                    vec![
                        StepWorker {
                            payload: Some(Payload::from(
                                r#"{"score": "context.credit - context.credit_used"}"#,
                            )),
                            then_case: Some(ID::from("pipeline_id_5")),
                            ..default::Default::default()
                        },
                        StepWorker {
                            condition: Some(Condition {
                                left: Payload::from("steps.step1.score"),
                                right: Payload::from("10"),
                                operator: Operator::GreaterThan,
                            }),
                            ..default::Default::default()
                        },
                        StepWorker {
                            condition: Some(Condition {
                                left: Payload::from("steps.step1.score"),
                                right: Payload::from("500"),
                                operator: Operator::GreaterThan,
                            }),
                            ..default::Default::default()
                        },
                        StepWorker {
                            condition: Some(Condition {
                                left: Payload::from("steps.step1.score"),
                                right: Payload::from("100000"),
                                operator: Operator::LessThan,
                            }),
                            ..default::Default::default()
                        },
                        StepWorker {
                            name: Some("Credit avaliable".to_string()),
                            condition: Some(Condition {
                                left: Payload::from("steps.step1.score"),
                                right: Payload::from("500"),
                                operator: Operator::Equal,
                            }),
                            then_case: Some(ID::from("pipeline_id_6")),
                            ..default::Default::default()
                        },
                    ],
                ),
            );
            map.insert(
                ID::from("pipeline_id_2"),
                Pipeline::new(
                    ID::from("pipeline_id_2"),
                    vec![StepWorker {
                        payload: Some(Payload::from(r#"{"score": "{{0}}"}"#)),
                        ..default::Default::default()
                    }],
                ),
            );
            map.insert(
                ID::from("pipeline_id_3"),
                Pipeline::new(
                    ID::from("pipeline_id_3"),
                    vec![StepWorker {
                        condition: Some(Condition {
                            left: Payload::from("steps.step1.score"),
                            right: Payload::from("500"),
                            operator: Operator::GreaterThan,
                        }),
                        then_case: Some(ID::from("pipeline_id_7")),
                        else_case: Some(ID::from("pipeline_id_8")),
                        ..default::Default::default()
                    }],
                ),
            );
            map.insert(
                ID::from("pipeline_id_4"),
                Pipeline::new(
                    ID::from("pipeline_id_4"),
                    vec![StepWorker {
                        name: Some("Credit not avaliable".to_string()),
                        payload: Some(Payload::from(r#"{"score": "{{false}}"}"#)),
                        ..default::Default::default()
                    }],
                ),
            );
            map.insert(
                ID::from("pipeline_id_5"),
                Pipeline::new(
                    ID::from("pipeline_id_5"),
                    vec![StepWorker {
                        name: Some("Credit avaliable".to_string()),
                        payload: Some(Payload::from(r#"{"resul": "{{true}}"}"#)),
                        ..default::Default::default()
                    }],
                ),
            );

            map
        };

        let result = transform_json(&Valu3Value::json_to_value(ORIGINAL).unwrap()).unwrap();

        assert_eq!(result, expected);
    }
}
