use std::collections::HashMap;
use valu3::{prelude::*, traits::ToValueBehavior, value::Value};

use crate::{
    id::ID,
    pipeline::{self, Pipeline},
    step_worker::{StepWorker, StepWorkerError},
    v8::PipelineMap,
};

#[derive(Debug)]
pub enum TransformError {
    InnerStepError(StepWorkerError),
    Parser(valu3::Error),
}

pub(crate) fn json_to_pipelines(
    input: &str,
) -> Result<(PipelineMap, Option<Value>), TransformError> {
    let value = Value::json_to_value(input).map_err(TransformError::Parser)?;
    let params = value.get("params").cloned();
    let pipelines = transform_json(&value)?;

    Ok((pipelines, params))
}

pub(crate) fn transform_json(input: &Value) -> Result<PipelineMap, TransformError> {
    let mut id_counter = 0;
    let mut map = Vec::new();

    process_raw_steps(input, &mut map);

    value_to_structs(&map)
}

pub(crate) fn get_pipeline_id(index: usize) -> u64 {
    index as u64
}

pub(crate) fn process_raw_steps(input: &Value, map: &mut Vec<Value>) -> Value {
    if let Value::Object(pipeline) = input {
        let mut new_pipeline = pipeline.clone();

        new_pipeline.remove(&"steps");

        if let Some(then) = pipeline.get("then") {
            new_pipeline.insert("then".to_string(), process_raw_steps(then, map));
        }
        if let Some(els) = pipeline.get("else") {
            new_pipeline.insert("else".to_string(), process_raw_steps(els, map));
        }

        let mut new_steps = if new_pipeline.is_empty() {
            vec![]
        } else {
            vec![new_pipeline.to_value()]
        };

        if let Some(steps) = pipeline.get("steps") {
            if let Value::Array(steps) = steps {
                for step in steps.into_iter() {
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
    }

    Number::from((map.len() - 1) as u128).to_value()
}

fn value_to_structs(map: &Vec<Value>) -> Result<PipelineMap, TransformError> {
    let mut pipelines = HashMap::new();

    for (pipeline_id, steps) in map.iter().enumerate() {
        if let Value::Array(arr) = steps {
            let mut steps = Vec::new();

            for step in arr.into_iter() {
                let step_worker =
                    StepWorker::try_from(step).map_err(TransformError::InnerStepError)?;
                steps.push(step_worker);
            }

            pipelines.insert(
                pipeline_id,
                Pipeline {
                    id: pipeline_id,
                    steps,
                },
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
    use valu3::{prelude::JsonMode, traits::ToValueBehavior, value::Value as Valu3Value};

    #[test]
    fn test_transform_value() {
        let mut map = Vec::new();
        let original = fs::read_to_string("assets/original.json").unwrap();
        let target = fs::read_to_string("assets/target.json").unwrap();

        process_raw_steps(&Valu3Value::json_to_value(&original).unwrap(), &mut map);

        println!("{:?}", map.to_value().to_json(JsonMode::Inline));

        assert_eq!(map.to_value(), Valu3Value::json_to_value(&target).unwrap());
    }

    #[test]
    fn test_transform_struct() {
        let expected = {
            let mut map = Vec::new();

            map.push(Pipeline::new(
                map.len(),
                vec![
                    StepWorker {
                        name: Some("Start".to_string()),
                        ..default::Default::default()
                    },
                    StepWorker {
                        id: ID::from("step1"),
                        condition: Some(Condition {
                            left: Payload::from("context.credit".to_value()),
                            right: Payload::from("context.credit_used".to_value()),
                            operator: Operator::GreaterThan,
                        }),
                        then_case: Some(1),
                        else_case: Some(4),
                        ..default::Default::default()
                    },
                    StepWorker {
                        condition: Some(Condition {
                            left: Payload::from("steps.step1.score".to_value()),
                            right: Payload::from(&500.to_value()),
                            operator: Operator::GreaterThan,
                        }),
                        then_case: Some(5),
                        else_case: Some(6),
                        ..default::Default::default()
                    },
                    StepWorker {
                        name: Some("End".to_string()),
                        ..default::Default::default()
                    },
                ],
            ));
            map.push(Pipeline::new(
                map.len(),
                vec![
                    StepWorker {
                        payload: Some(Payload::from(
                            r#"{"score": "context.credit - context.credit_used"}"#.to_value(),
                        )),
                        ..default::Default::default()
                    },
                    StepWorker {
                        condition: Some(Condition {
                            left: Payload::from("steps.step1.score".to_value()),
                            right: Payload::from(10.to_value()),
                            operator: Operator::GreaterThan,
                        }),
                        ..default::Default::default()
                    },
                    StepWorker {
                        condition: Some(Condition {
                            left: Payload::from("steps.step1.score".to_value()),
                            right: Payload::from(500.to_value()),
                            operator: Operator::GreaterThan,
                        }),
                        ..default::Default::default()
                    },
                    StepWorker {
                        condition: Some(Condition {
                            left: Payload::from("steps.step1.score".to_value()),
                            right: Payload::from(100000.to_value()),
                            operator: Operator::LessThan,
                        }),
                        ..default::Default::default()
                    },
                    StepWorker {
                        then_case: Some(2),
                        ..default::Default::default()
                    },
                ],
            ));
            map.push(Pipeline::new(
                map.len(),
                vec![StepWorker {
                    condition: Some(Condition {
                        left: Payload::from("steps.step1.score".to_value()),
                        right: Payload::from(500.to_value()),
                        operator: Operator::Equal,
                    }),
                    then_case: Some(3),
                    ..default::Default::default()
                }],
            ));
            map.push(Pipeline::new(
                map.len(),
                vec![StepWorker {
                    return_case: Some(Payload::from(r#"{"result": true}"#.to_value())),
                    ..default::Default::default()
                }],
            ));
            map.push(Pipeline::new(
                map.len(),
                vec![StepWorker {
                    payload: Some(Payload::from(r#"{"score": 0}"#.to_value())),
                    ..default::Default::default()
                }],
            ));
            map.push(Pipeline::new(
                map.len(),
                vec![StepWorker {
                    name: Some("Credit avaliable".to_string()),
                    payload: Some(Payload::from(r#"{"result": true}"#.to_value())),
                    ..default::Default::default()
                }],
            ));
            map.push(Pipeline::new(
                map.len(),
                vec![StepWorker {
                    name: Some("Credit avaliable".to_string()),
                    payload: Some(Payload::from(r#"{"score": false}"#.to_value())),
                    ..default::Default::default()
                }],
            ));

            map
        };
        let original = fs::read_to_string("assets/original.json").unwrap();

        let result = transform_json(&Valu3Value::json_to_value(&original).unwrap()).unwrap();

        assert_eq!(
            result.get(&0).unwrap().steps.get(0).unwrap().id,
            expected.get(0).unwrap().steps.get(0).unwrap().id
        );

        assert_eq!(
            result
                .get(&0)
                .unwrap()
                .steps
                .get(1)
                .unwrap()
                .condition
                .as_ref()
                .unwrap()
                .left,
            expected
                .get(0)
                .unwrap()
                .steps
                .get(1)
                .unwrap()
                .condition
                .as_ref()
                .unwrap()
                .left
        );
    }
}
