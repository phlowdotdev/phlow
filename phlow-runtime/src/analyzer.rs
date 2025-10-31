use crate::loader::error::Error as LoaderError;
use crate::loader::loader::load_script;
use phlow_sdk::prelude::*;
use regex::Regex;
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn collect_includes_recursive(
    path: &Path,
    visited: &mut HashSet<String>,
    result: &mut HashSet<String>,
) {
    let path_str = match path.canonicalize() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    };

    if visited.contains(&path_str) {
        return;
    }
    visited.insert(path_str.clone());

    if let Ok(content) = fs::read_to_string(path) {
        result.insert(path_str.clone());

        // find !include and !import occurrences
        let include_re = Regex::new(r"!include\s+([^\s]+)").unwrap();
        let import_re = Regex::new(r"!import\s+(\S+)").unwrap();

        let base = path.parent().unwrap_or(Path::new("."));

        for cap in include_re.captures_iter(&content) {
            if let Some(rel) = cap.get(1) {
                let mut full = base.join(rel.as_str());
                if full.extension().is_none() {
                    full.set_extension("phlow");
                }
                if full.exists() {
                    collect_includes_recursive(&full, visited, result);
                } else {
                    // still add referenced path even if it does not exist
                    result.insert(full.to_string_lossy().to_string());
                }
            }
        }

        for cap in import_re.captures_iter(&content) {
            if let Some(rel) = cap.get(1) {
                let full = base.join(rel.as_str());
                if full.exists() {
                    collect_includes_recursive(&full, visited, result);
                } else {
                    result.insert(full.to_string_lossy().to_string());
                }
            }
        }
    }
}

fn normalize_module_name(module_name: &str) -> String {
    if module_name.starts_with("./modules/") {
        module_name[10..].to_string()
    } else if module_name.contains('/') {
        module_name
            .split('/')
            .last()
            .unwrap_or(module_name)
            .to_string()
    } else {
        module_name.to_string()
    }
}

fn count_pipelines_recursive(value: &Value) -> usize {
    use phlow_sdk::prelude::Value::*;

    match value {
        Object(map) => {
            let mut count = 1; // this object is a pipeline

            if let Some(then) = map.get("then") {
                count += count_pipelines_recursive(then);
            }
            if let Some(els) = map.get("else") {
                count += count_pipelines_recursive(els);
            }

            if let Some(steps) = map.get("steps") {
                match steps {
                    Array(arr) => {
                        for step in arr {
                            if let Object(step_obj) = step {
                                if let Some(t) = step_obj.get("then") {
                                    count += count_pipelines_recursive(t);
                                }
                                if let Some(e) = step_obj.get("else") {
                                    count += count_pipelines_recursive(e);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            count
        }
        Array(arr) => {
            let mut count = 1; // this array is a pipeline
            for step in arr {
                if let Value::Object(step_obj) = step {
                    if let Some(t) = step_obj.get("then") {
                        count += count_pipelines_recursive(t);
                    }
                    if let Some(e) = step_obj.get("else") {
                        count += count_pipelines_recursive(e);
                    }
                }
            }
            count
        }
        _ => 0,
    }
}

fn count_steps_recursive(value: &Value) -> usize {
    use phlow_sdk::prelude::Value::*;

    match value {
        Object(map) => {
            let mut steps_total = 0;
            if let Some(steps) = map.get("steps") {
                if let Array(arr) = steps {
                    steps_total += arr.len();
                    for step in arr {
                        if let Object(step_obj) = step {
                            if let Some(t) = step_obj.get("then") {
                                steps_total += count_steps_recursive(t);
                            }
                            if let Some(e) = step_obj.get("else") {
                                steps_total += count_steps_recursive(e);
                            }
                        }
                    }
                }
            }

            if let Some(then) = map.get("then") {
                steps_total += count_steps_recursive(then);
            }
            if let Some(els) = map.get("else") {
                steps_total += count_steps_recursive(els);
            }

            steps_total
        }
        Array(arr) => {
            let mut steps_total = 0;
            steps_total += arr.len();
            for step in arr {
                if let Value::Object(step_obj) = step {
                    if let Some(t) = step_obj.get("then") {
                        steps_total += count_steps_recursive(t);
                    }
                    if let Some(e) = step_obj.get("else") {
                        steps_total += count_steps_recursive(e);
                    }
                }
            }
            steps_total
        }
        _ => 0,
    }
}

pub async fn analyze(
    script_target: &str,
    include_files: bool,
    include_modules: bool,
    include_total_steps: bool,
    include_total_pipelines: bool,
) -> Result<serde_json::Value, LoaderError> {
    // Try load with loader (preferred) and fallback to tolerant analysis on failure
    let mut files_set: HashSet<String> = HashSet::new();
    let mut modules_json: Vec<serde_json::Value> = Vec::new();
    let mut total_pipelines = 0usize;
    let mut total_steps = 0usize;

    match load_script(script_target, false).await {
        Ok(script_loaded) => {
            if include_files {
                let mut visited: HashSet<String> = HashSet::new();
                let path = Path::new(&script_loaded.script_file_path).to_path_buf();
                collect_includes_recursive(&path, &mut visited, &mut files_set);
            }

            if include_modules {
                if let Some(modules) = script_loaded.script.get("modules") {
                    if let Value::Array(arr) = modules {
                        for module in arr {
                            if let Value::Object(map) = module {
                                let mut module_name = None;
                                if let Some(Value::String(s)) = map.get("module") {
                                    module_name = Some(s.clone());
                                } else if let Some(Value::String(s)) = map.get("name") {
                                    module_name = Some(s.clone());
                                }

                                if let Some(mn) = module_name {
                                    let mn_str = mn.to_string();
                                    let clean = normalize_module_name(&mn_str);
                                    let downloaded =
                                        Path::new(&format!("phlow_packages/{}", clean)).exists();
                                    modules_json.push(json!({"declared": mn_str, "name": clean, "downloaded": downloaded}));
                                }
                            }
                        }
                    }
                }
            }

            if include_total_pipelines || include_total_steps {
                if let Some(steps_val) = script_loaded.script.get("steps") {
                    total_pipelines = count_pipelines_recursive(steps_val);
                    total_steps = count_steps_recursive(steps_val);
                }
            }
        }
        Err(_) => {
            // Fallback: tolerant analysis without running preprocessor - best-effort parsing of the raw file
            let target_path = Path::new(script_target);
            let main_path = if target_path.is_dir() {
                let mut base_path = target_path.to_path_buf();
                base_path.set_extension("phlow");
                if base_path.exists() {
                    base_path
                } else {
                    let candidates = ["main.phlow", "mod.phlow", "module.phlow"];
                    let mut found = None;
                    for c in &candidates {
                        let p = target_path.join(c);
                        if p.exists() {
                            found = Some(p);
                            break;
                        }
                    }
                    if let Some(p) = found {
                        p
                    } else {
                        return Err(LoaderError::MainNotFound(script_target.to_string()));
                    }
                }
            } else if target_path.exists() {
                target_path.to_path_buf()
            } else {
                return Err(LoaderError::MainNotFound(script_target.to_string()));
            };

            let content = fs::read_to_string(&main_path).map_err(|_| {
                LoaderError::ModuleLoaderError("Failed to read main file".to_string())
            })?;

            if include_files {
                // collect referenced include/import paths even if they don't exist
                let include_re = Regex::new(r"!include\s+([^\s]+)").unwrap();
                let import_re = Regex::new(r"!import\s+(\S+)").unwrap();
                let base = main_path.parent().unwrap_or(Path::new("."));

                for cap in include_re.captures_iter(&content) {
                    if let Some(rel) = cap.get(1) {
                        let mut full = base.join(rel.as_str());
                        if full.extension().is_none() {
                            full.set_extension("phlow");
                        }
                        files_set.insert(full.to_string_lossy().to_string());
                    }
                }

                for cap in import_re.captures_iter(&content) {
                    if let Some(rel) = cap.get(1) {
                        let full = base.join(rel.as_str());
                        files_set.insert(full.to_string_lossy().to_string());
                    }
                }
            }

            if include_modules {
                let modules_re = Regex::new(r"module:\s*([^\n\r]+)").unwrap();
                for cap in modules_re.captures_iter(&content) {
                    if let Some(m) = cap.get(1) {
                        let mn_str = m.as_str().trim().to_string();
                        let clean = normalize_module_name(&mn_str);
                        let downloaded = Path::new(&format!("phlow_packages/{}", clean)).exists();
                        modules_json.push(
                            json!({"declared": mn_str, "name": clean, "downloaded": downloaded}),
                        );
                    }
                }
            }

            if include_total_pipelines || include_total_steps {
                if content.contains("steps:") {
                    let parts: Vec<&str> = content.splitn(2, "steps:").collect();
                    if parts.len() > 1 {
                        let steps_block = parts[1];
                        let steps_count = steps_block.matches("\n- ").count();
                        total_steps = steps_count;
                        total_pipelines = 1;
                    }
                }
            }
        }
    }

    let mut files_vec: Vec<String> = files_set.into_iter().collect();
    files_vec.sort();

    Ok(json!({
        "files": files_vec,
        "modules": modules_json,
        "total_steps": total_steps,
        "total_pipelines": total_pipelines
    }))
}
