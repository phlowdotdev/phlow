use std::collections::HashMap;
use valu3::{prelude::*, traits::ToValueBehavior, value::Value};

use crate::{
    id::ID,
    pipeline::Pipeline,
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
    let pipelines = value_to_pipelines(&value)?;

    Ok((pipelines, params))
}

pub(crate) fn value_to_pipelines(input: &Value) -> Result<PipelineMap, TransformError> {
    let mut map = Vec::new();

    process_raw_steps(input, &mut map);

    value_to_structs(&map)
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

    let json = (map.len() - 1).to_value().to_json(JsonMode::Inline);
    Value::json_to_value(&json).unwrap()
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
    use super::*;
    use std::fs;
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
}
