use crate::{
    phlow::PipelineMap,
    pipeline::Pipeline,
    step_worker::{StepReference, StepWorker, StepWorkerError},
};
use phlow_sdk::{
    prelude::{log::debug, *},
    valu3,
};
use rhai::Engine;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};
use valu3::{traits::ToValueBehavior, value::Value};

#[derive(Debug)]
pub enum TransformError {
    InnerStepError(StepWorkerError),
    Parser(valu3::Error),
}

impl Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::InnerStepError(err) => write!(f, "Inner step error: {}", err),
            TransformError::Parser(_) => write!(f, "Parser error: Non parseable"),
        }
    }
}

impl std::error::Error for TransformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TransformError::InnerStepError(err) => Some(err),
            TransformError::Parser(_) => None, // valu3::Error doesn't implement std::error::Error
        }
    }
}

pub(crate) fn value_to_pipelines(
    engine: Arc<Engine>,
    modules: Arc<Modules>,
    input: &Value,
) -> Result<PipelineMap, TransformError> {
    let mut map = Vec::new();

    process_raw_steps(input, &mut map);
    debug!("{}", map.to_value().to_json(JsonMode::Indented));
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

/// Function to transform a value into a pipeline map
/// This function takes a value and transforms it into a pipeline map.
/// It uses the `value_to_structs` function to convert the value into a pipeline map.
/// It also uses the `resolve_go_to_step` function to resolve the "go to" step.
/// The function returns a `Result` with the pipeline map or an error.
fn value_to_structs(
    engine: Arc<Engine>,
    modules: Arc<Modules>,
    pipelines_raw: &Vec<Value>,
) -> Result<PipelineMap, TransformError> {
    let (parents, go_to_step_id) = map_parents(pipelines_raw);
    log::debug!("Parent mappings: {:?}", parents);
    log::debug!(
        "Pipeline structure: {}",
        pipelines_raw.to_value().to_json(JsonMode::Indented)
    );
    let mut pipelines = HashMap::new();

    for (pipeline_index, pipeline_value) in pipelines_raw.iter().enumerate() {
        if let Value::Array(arr) = pipeline_value {
            let mut steps = Vec::new();

            for (step_index, step_value) in arr.into_iter().enumerate() {
                if let Value::Object(step) = step_value {
                    let mut new_step = step.clone();
                    #[cfg(debug_assertions)]
                    {
                        log::debug!("new_step {:?}", new_step.to_value().to_string());
                    }

                    if let Some(to) = step.get("to") {
                        if let Some(go_to_step) = go_to_step_id.get(to.to_string().as_str()) {
                            new_step.insert("to".to_string(), go_to_step.to_value());
                        }
                    } else {
                        if step.get("return").is_none() && step_index == arr.len() - 1
                        {
                            if let Some(target) = parents.get(&StepReference {
                                pipeline: pipeline_index,
                                step: 0,
                            }) {
                                if let Some(next_step_ref) =
                                    next_step_if_exists(&pipelines_raw, target)
                                {
                                    log::debug!("Setting up parent return: pipeline {} → pipeline {} step {}", pipeline_index, next_step_ref.pipeline, next_step_ref.step);
                                    new_step.insert("to".to_string(), next_step_ref.to_value());
                                } else if let Some(valid_step) =
                                    find_valid_continuation(&pipelines_raw, &parents, target)
                                {
                                    log::debug!("Found valid continuation: pipeline {} → pipeline {} step {}", pipeline_index, valid_step.pipeline, valid_step.step);
                                    new_step.insert("to".to_string(), valid_step.to_value());
                                } else {
                                    log::warn!(
                                        "No valid continuation found for pipeline {}",
                                        pipeline_index
                                    );
                                }
                            } else {
                                // BUGFIX: Se não tem parent e não é a pipeline principal,
                                // esta pipeline pode ser órfã e deve retornar ao pipeline principal
                                let main_pipeline = pipelines_raw.len() - 1;
                                if pipeline_index != main_pipeline {
                                    // Check if this pipeline is referenced as a then/else branch
                                    let mut found_parent = false;

                                    // Search through all pipelines to find if this one is referenced as a then/else branch
                                    for (parent_pipeline_idx, parent_pipeline) in
                                        pipelines_raw.iter().enumerate()
                                    {
                                        if let Value::Array(parent_steps) = parent_pipeline {
                                            for (parent_step_idx, parent_step) in
                                                parent_steps.values.iter().enumerate()
                                            {
                                                if let Value::Object(step_obj) = parent_step {
                                                    // Check if this step references our pipeline as a then branch
                                                    if let Some(then_val) = step_obj
                                                        .get("then")
                                                        .and_then(|v| v.to_u64())
                                                    {
                                                        if then_val as usize == pipeline_index {
                                                            // Find the next available step in the parent pipeline
                                                            let next_step_idx = parent_step_idx + 1;
                                                            if next_step_idx
                                                                < parent_steps.values.len()
                                                            {
                                                                let next_step = StepReference {
                                                                    pipeline: parent_pipeline_idx,
                                                                    step: next_step_idx,
                                                                };
                                                                log::debug!("Setting up then branch return: pipeline {} → pipeline {} step {}", pipeline_index, next_step.pipeline, next_step.step);
                                                                new_step.insert(
                                                                    "to".to_string(),
                                                                    next_step.to_value(),
                                                                );
                                                                found_parent = true;
                                                                break;
                                                            } else {
                                                                // No more steps in parent pipeline, need to find its parent
                                                                log::debug!("Then branch pipeline {} has no next step in parent pipeline {}", pipeline_index, parent_pipeline_idx);
                                                                // For now, let's see if this parent pipeline has a parent
                                                                if let Some(parent_target) = parents
                                                                    .get(&StepReference {
                                                                        pipeline:
                                                                            parent_pipeline_idx,
                                                                        step: 0,
                                                                    })
                                                                {
                                                                    if let Some(next_step) =
                                                                        find_valid_continuation(
                                                                            &pipelines_raw,
                                                                            &parents,
                                                                            parent_target,
                                                                        )
                                                                    {
                                                                        log::debug!("Setting up then branch return via grandparent: pipeline {} → pipeline {} step {}", pipeline_index, next_step.pipeline, next_step.step);
                                                                        new_step.insert(
                                                                            "to".to_string(),
                                                                            next_step.to_value(),
                                                                        );
                                                                        found_parent = true;
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    // Check if this step references our pipeline as an else branch
                                                    if let Some(else_val) = step_obj
                                                        .get("else")
                                                        .and_then(|v| v.to_u64())
                                                    {
                                                        if else_val as usize == pipeline_index {
                                                            // Find the next available step in the parent pipeline
                                                            let next_step_idx = parent_step_idx + 1;
                                                            if next_step_idx
                                                                < parent_steps.values.len()
                                                            {
                                                                let next_step = StepReference {
                                                                    pipeline: parent_pipeline_idx,
                                                                    step: next_step_idx,
                                                                };
                                                                log::debug!("Setting up else branch return: pipeline {} → pipeline {} step {}", pipeline_index, next_step.pipeline, next_step.step);
                                                                new_step.insert(
                                                                    "to".to_string(),
                                                                    next_step.to_value(),
                                                                );
                                                                found_parent = true;
                                                                break;
                                                            } else {
                                                                // No more steps in parent pipeline, need to find its parent
                                                                log::debug!("Else branch pipeline {} has no next step in parent pipeline {}", pipeline_index, parent_pipeline_idx);
                                                                // For now, let's see if this parent pipeline has a parent
                                                                if let Some(parent_target) = parents
                                                                    .get(&StepReference {
                                                                        pipeline:
                                                                            parent_pipeline_idx,
                                                                        step: 0,
                                                                    })
                                                                {
                                                                    if let Some(next_step) =
                                                                        find_valid_continuation(
                                                                            &pipelines_raw,
                                                                            &parents,
                                                                            parent_target,
                                                                        )
                                                                    {
                                                                        log::debug!("Setting up else branch return via grandparent: pipeline {} → pipeline {} step {}", pipeline_index, next_step.pipeline, next_step.step);
                                                                        new_step.insert(
                                                                            "to".to_string(),
                                                                            next_step.to_value(),
                                                                        );
                                                                        found_parent = true;
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if found_parent {
                                                break;
                                            }
                                        }
                                    }

                                    if !found_parent {
                                        // Esta é uma sub-pipeline órfã - deve retornar ao pipeline principal
                                        // Return to the next step after the first step (which is the assert/conditional)
                                        let next_step = StepReference {
                                            pipeline: main_pipeline,
                                            step: 1, // Continue from step 1 in main pipeline
                                        };
                                        log::debug!("Setting up orphan pipeline return: pipeline {} → pipeline {} step {}", pipeline_index, next_step.pipeline, next_step.step);
                                        new_step.insert("to".to_string(), next_step.to_value());
                                    }
                                }
                            }
                        }
                    }

                    // Use para debugar a saida do step
                    // println!("{}", new_step.to_value().to_json(JsonMode::Indented));

                    let step_worker = StepWorker::try_from_value(
                        engine.clone(),
                        modules.clone(),
                        &new_step.to_value(),
                    )
                    .map_err(TransformError::InnerStepError)?;

                    steps.push(step_worker);
                }
            }

            pipelines.insert(
                pipeline_index,
                Pipeline {
                    steps,
                    id: pipeline_index,
                },
            );
        }
    }

    Ok(pipelines)
}

/// Function to check if a step reference is valid
fn is_valid_step(pipelines: &Vec<Value>, step_ref: &StepReference) -> bool {
    if step_ref.pipeline >= pipelines.len() {
        return false;
    }

    if let Value::Array(arr) = &pipelines[step_ref.pipeline] {
        return step_ref.step < arr.len();
    }

    false
}

fn next_step_if_exists(pipelines: &Vec<Value>, target: &StepReference) -> Option<StepReference> {
    if let Value::Array(arr) = &pipelines[target.pipeline] {
        let next_step_index = target.step + 1;
        if next_step_index < arr.len() {
            return Some(StepReference {
                pipeline: target.pipeline,
                step: next_step_index,
            });
        }
    }

    None
}

/// Function to find a valid continuation point when the direct next step is invalid
fn find_valid_continuation(
    pipelines: &Vec<Value>,
    parents: &HashMap<StepReference, StepReference>,
    target: &StepReference,
) -> Option<StepReference> {
    let mut current = target.clone();
    let mut depth = 0usize;

    loop {
        let next_step_ref = StepReference {
            pipeline: current.pipeline,
            step: current.step + 1,
        };

        if is_valid_step(pipelines, &next_step_ref) {
            return Some(next_step_ref);
        }

        let parent_key = StepReference {
            pipeline: current.pipeline,
            step: 0,
        };
        let Some(parent) = parents.get(&parent_key) else {
            return None;
        };

        if parent.pipeline == current.pipeline && parent.step == current.step {
            return None;
        }

        current = parent.clone();
        depth += 1;
        if depth > pipelines.len() {
            return None;
        }
    }
}

/// Function to map parents
/// This function takes a vector of pipelines and builds a parent map.
fn map_parents(
    pipelines: &Vec<Value>,
) -> (
    HashMap<StepReference, StepReference>,
    HashMap<String, StepReference>,
) {
    let (parents, go_to_step_references) = build_parent_map(pipelines);
    (parents, go_to_step_references)
}

/// Function to build the parent map
/// This function takes a vector of pipelines and builds a parent map.
/// It uses a hashmap to store the step references.
fn build_parent_map(
    pipelines: &Vec<Value>,
) -> (
    HashMap<StepReference, StepReference>,
    HashMap<String, StepReference>,
) {
    let mut parents = HashMap::new();
    let mut go_to_step_id = HashMap::new();

    for (pipeline_index, steps) in pipelines.iter().enumerate() {
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

                    // Adiciona relações de "then" e "else" ao mapa de pais
                    if let Some(then_value) = step.get("then").and_then(|v| v.to_u64()) {
                        parents.insert(
                            StepReference {
                                pipeline: then_value as usize,
                                step: 0,
                            },
                            StepReference {
                                pipeline: pipeline_index,
                                step: step_index,
                            },
                        );
                    }

                    if let Some(else_value) = step.get("else").and_then(|v| v.to_u64()) {
                        parents.insert(
                            StepReference {
                                pipeline: else_value as usize,
                                step: 0,
                            },
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

    (parents, go_to_step_id)
}
