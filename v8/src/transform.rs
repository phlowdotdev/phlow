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

    if let Value::Object(pipeline) = input {
        let mut new_pipeline = pipeline.clone();

        if let Some(then) = pipeline.get("then") {
            new_pipeline.insert("then".to_string(), process_raw_steps(then, id_counter, map));
        }
        if let Some(els) = pipeline.get("else") {
            new_pipeline.insert("else".to_string(), process_raw_steps(els, id_counter, map));
        }

        if let Some(steps) = pipeline.get("steps") {
            let mut new_steps = Vec::new();

            if let Value::Array(steps) = steps {
                for step in steps.into_iter() {
                    let mut new_step = step.clone();

                    if let Some(then) = step.get("then") {
                        new_step
                            .insert("then".to_string(), process_raw_steps(then, id_counter, map));
                    }
                    if let Some(els) = step.get("else") {
                        new_step
                            .insert("else".to_string(), process_raw_steps(els, id_counter, map));
                    }

                    new_steps.push(new_step);
                }
            }

            new_pipeline.insert("steps".to_string(), Value::from(new_steps));
        }

        map.insert(key.clone(), Value::from(new_pipeline));
    }

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
    use std::{default, fs};

    use crate::{
        condition::{Condition, Operator},
        payload::Payload,
    };

    use super::*;
    use valu3::{traits::ToValueBehavior, value::Value as Valu3Value};

    #[test]
    fn test_transform_value() {
        let mut id_counter = 0;
        let mut map = HashMap::new();
        let original = fs::read_to_string("assets/original.json").unwrap();
        let target = fs::read_to_string("assets/target.json").unwrap();

        process_raw_steps(
            &Valu3Value::json_to_value(&original).unwrap(),
            &mut id_counter,
            &mut map,
        );

        assert_eq!(map.to_value(), Valu3Value::json_to_value(&target).unwrap());
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
        let original = fs::read_to_string("assets/pipeline.json").unwrap();

        let result = transform_json(&Valu3Value::json_to_value(&original).unwrap()).unwrap();

        assert_eq!(result, expected);
    }
}
