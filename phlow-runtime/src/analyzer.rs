use crate::loader::error::Error as LoaderError;
use crate::preprocessor::preprocessor;
use crate::settings::Settings;
use phlow_sdk::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct Analyzer {
    pub enabled: bool,
    pub files: bool,
    pub modules: bool,
    pub total_steps: bool,
    pub total_pipelines: bool,
    pub json: bool,
    pub script_target: String,
    pub all: bool,
    pub inner: bool,
}

impl Analyzer {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            enabled: settings.analyzer,
            files: settings.analyzer_files,
            modules: settings.analyzer_modules,
            total_steps: settings.analyzer_total_steps,
            total_pipelines: settings.analyzer_total_pipelines,
            json: settings.analyzer_json,
            script_target: settings.script_main_absolute_path.clone(),
            all: settings.analyzer_all,
            inner: settings.analyzer_inner, // Assuming this is how inner is set from settings
        }
    }

    pub async fn run(&self) -> Result<Value, LoaderError> {
        // If no specific analyzer flags were provided, show all
        let mut af = self.files;
        let mut am = self.modules;
        let mut ats = self.total_steps;
        let mut atp = self.total_pipelines;

        if self.all {
            af = true;
            am = true;
            ats = true;
            atp = true;
        }

        let result = analyze(&self.script_target, af, am, ats, atp, self.inner).await?;
        Ok(result)
    }

    pub fn display(&self, result: &Value) {
        if self.json {
            // print valu3 Value as pretty JSON
            println!("{}", result.to_json(JsonMode::Indented));
            return;
        }

        // text output similar to previous main.rs behavior
        if self.files || self.all {
            if let Some(files) = result.get("files") {
                println!("Files:");
                if let Some(arr) = files.as_array() {
                    for f in &arr.values {
                        println!("  - {}", f.as_string());
                    }
                }
            }
        }

        if self.modules || self.all {
            if let Some(mods) = result.get("modules") {
                println!("Modules:");
                if let Some(arr) = mods.as_array() {
                    for m in &arr.values {
                        let declared = m.get("declared").map(|v| v.as_string()).unwrap_or_default();
                        let name = m.get("name").map(|v| v.as_string()).unwrap_or_default();
                        let downloaded = m
                            .get("downloaded")
                            .map(|v| v.to_string())
                            .unwrap_or_default();
                        println!("  - {} ({}): downloaded={}", declared, name, downloaded);
                    }
                }
            }
        }

        if self.total_steps || self.all {
            if let Some(ts) = result.get("total_steps") {
                println!("Total steps: {}", ts.to_string());
            }
        }

        if self.total_pipelines || self.all {
            if let Some(tp) = result.get("total_pipelines") {
                println!("Total pipelines: {}", tp.to_string());
            }
        }
    }
}
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
    if value.is_object() {
        let mut count = 1; // this object is a pipeline

        if let Some(then) = value.get("then") {
            count += count_pipelines_recursive(then);
        }
        if let Some(els) = value.get("else") {
            count += count_pipelines_recursive(els);
        }

        if let Some(steps) = value.get("steps").and_then(|v| v.as_array()) {
            for step in &steps.values {
                if let Some(t) = step.get("then") {
                    count += count_pipelines_recursive(t);
                }
                if let Some(e) = step.get("else") {
                    count += count_pipelines_recursive(e);
                }
            }
        }

        count
    } else if let Some(arr) = value.as_array() {
        let mut count = 1; // this array is a pipeline
        for step in &arr.values {
            if let Some(t) = step.get("then") {
                count += count_pipelines_recursive(t);
            }
            if let Some(e) = step.get("else") {
                count += count_pipelines_recursive(e);
            }
        }
        count
    } else {
        0
    }
}

fn count_steps_recursive(value: &Value) -> usize {
    if value.is_object() {
        let mut steps_total = 0;
        if let Some(steps) = value.get("steps").and_then(|v| v.as_array()) {
            steps_total += steps.values.len();
            for step in &steps.values {
                if let Some(t) = step.get("then") {
                    steps_total += count_steps_recursive(t);
                }
                if let Some(e) = step.get("else") {
                    steps_total += count_steps_recursive(e);
                }
            }
        }

        if let Some(then) = value.get("then") {
            steps_total += count_steps_recursive(then);
        }
        if let Some(els) = value.get("else") {
            steps_total += count_steps_recursive(els);
        }

        steps_total
    } else if let Some(arr) = value.as_array() {
        let mut steps_total = 0;
        steps_total += arr.values.len();
        for step in &arr.values {
            if let Some(t) = step.get("then") {
                steps_total += count_steps_recursive(t);
            }
            if let Some(e) = step.get("else") {
                steps_total += count_steps_recursive(e);
            }
        }
        steps_total
    } else {
        0
    }
}

fn analyze_internal<'a>(
    script_target: &'a str,
    include_files: bool,
    include_modules: bool,
    include_total_steps: bool,
    include_total_pipelines: bool,
    include_inner: bool,
    visited: &'a mut HashSet<String>,
) -> Pin<Box<dyn Future<Output = Result<Value, LoaderError>> + 'a>> {
    Box::pin(async move {
        // Try load with loader (preferred) and fallback to tolerant analysis on failure
        let mut files_set: HashSet<String> = HashSet::new();
        let mut modules_json: Vec<Value> = Vec::new();
        let mut total_pipelines = 0usize;
        let mut total_steps = 0usize;

        // First, try to run the preprocessor to obtain the final transformed YAML
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

        // protect against recursion cycles
        let canonical = match main_path.canonicalize() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => main_path.to_string_lossy().to_string(),
        };
        if visited.contains(&canonical) {
            // already analyzed -> return empty result
            return Ok(
                json!({"files": Vec::<String>::new(), "modules": Vec::<Value>::new(), "total_steps": 0, "total_pipelines": 0}),
            );
        }
        visited.insert(canonical);

        let raw = fs::read_to_string(&main_path)
            .map_err(|_| LoaderError::ModuleLoaderError("Failed to read main file".to_string()))?;

        // Try preprocessor (preferred). If it fails or the resulting YAML cannot be parsed,
        // fall back to the tolerant heuristic analysis below.
        let preprocessed = preprocessor(
            &raw,
            &main_path.parent().unwrap_or(Path::new(".")),
            false,
            crate::settings::PrintOutput::Yaml,
        );

        if let Ok(transformed) = preprocessed {
            // parse the YAML into valu3::Value for analysis
            match serde_yaml::from_str::<Value>(&transformed) {
                Ok(root) => {
                    if include_files {
                        let mut visited: HashSet<String> = HashSet::new();
                        collect_includes_recursive(&main_path, &mut visited, &mut files_set);
                    }

                    if include_modules {
                        if let Some(mods) = root.get("modules").and_then(|v| v.as_array()) {
                            for module in &mods.values {
                                // declared: the literal declared value (module: ...)
                                // name_raw: prefer `name` attribute, fallback to declared
                                let (declared, name_raw) = if module.is_object() {
                                    let declared = module
                                        .get("module")
                                        .map(|v| v.as_string())
                                        .unwrap_or_default();
                                    let name_raw = module
                                        .get("name")
                                        .map(|v| v.as_string())
                                        .unwrap_or_else(|| declared.clone());
                                    (declared, name_raw)
                                } else {
                                    let declared = module.as_string();
                                    let name_raw = declared.clone();
                                    (declared, name_raw)
                                };

                                let clean = normalize_module_name(&name_raw);

                                // determine downloaded: phlow_packages/{clean} or local module base dir
                                let mut downloaded = String::new();
                                let pp_path = format!("phlow_packages/{}", clean);
                                let pp = Path::new(&pp_path);
                                if pp.exists() {
                                    downloaded = pp.to_string_lossy().to_string();
                                }

                                if declared.starts_with('.') {
                                    let base = main_path.parent().unwrap_or(Path::new("."));
                                    let mut candidate = base.join(&declared);
                                    if candidate.is_dir() {
                                        for c in ["main.phlow", "mod.phlow", "module.phlow"] {
                                            let p = candidate.join(c);
                                            if p.exists() {
                                                candidate = p;
                                                break;
                                            }
                                        }
                                    } else if candidate.extension().is_none() {
                                        let mut with_ext = candidate.clone();
                                        with_ext.set_extension("phlow");
                                        if with_ext.exists() {
                                            candidate = with_ext;
                                        }
                                    }

                                    if candidate.exists() {
                                        if candidate.is_dir() {
                                            downloaded = candidate.to_string_lossy().to_string();
                                        } else if let Some(p) = candidate.parent() {
                                            downloaded = p.to_string_lossy().to_string();
                                        }
                                    }
                                }

                                modules_json.push(json!({"declared": declared, "name": clean, "downloaded": downloaded}));

                                // If declared is local, try recursive analyze when it resolves to main.phlow
                                // only perform recursive analysis when `include_inner` is true
                                if declared.starts_with('.') && include_inner {
                                    let base = main_path.parent().unwrap_or(Path::new("."));
                                    let mut candidate = base.join(&declared);
                                    if candidate.is_dir() {
                                        for c in ["main.phlow", "mod.phlow", "module.phlow"] {
                                            let p = candidate.join(c);
                                            if p.exists() {
                                                candidate = p;
                                                break;
                                            }
                                        }
                                    } else if candidate.extension().is_none() {
                                        let mut with_ext = candidate.clone();
                                        with_ext.set_extension("phlow");
                                        if with_ext.exists() {
                                            candidate = with_ext;
                                        }
                                    }

                                    if candidate.exists() {
                                        if let Some(fname) =
                                            candidate.file_name().and_then(|s| s.to_str())
                                        {
                                            if fname == "main.phlow" {
                                                if let Ok(nested) = analyze_internal(
                                                    &candidate.to_string_lossy(),
                                                    include_files,
                                                    include_modules,
                                                    include_total_steps,
                                                    include_total_pipelines,
                                                    include_inner,
                                                    visited,
                                                )
                                                .await
                                                {
                                                    if let Some(nfiles) = nested
                                                        .get("files")
                                                        .and_then(|v| v.as_array())
                                                    {
                                                        for f in &nfiles.values {
                                                            files_set.insert(f.to_string());
                                                        }
                                                    }
                                                    if let Some(nmods) = nested
                                                        .get("modules")
                                                        .and_then(|v| v.as_array())
                                                    {
                                                        for m in &nmods.values {
                                                            modules_json.push(m.clone());
                                                        }
                                                    }
                                                    if let Some(ns) = nested.get("total_steps") {
                                                        if let Ok(nv) =
                                                            ns.to_string().parse::<usize>()
                                                        {
                                                            total_steps += nv;
                                                        }
                                                    }
                                                    if let Some(np) = nested.get("total_pipelines")
                                                    {
                                                        if let Ok(nv) =
                                                            np.to_string().parse::<usize>()
                                                        {
                                                            total_pipelines += nv;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if include_total_pipelines || include_total_steps {
                        if let Some(steps_val) = root.get("steps") {
                            // add the pipelines/steps from the main root to any totals already accumulated from nested modules
                            total_pipelines += count_pipelines_recursive(steps_val);
                            total_steps += count_steps_recursive(steps_val);
                        }
                    }
                }
                Err(_) => {
                    // parse failed: fallthrough to tolerant fallback below
                }
            }
        }

        // If after preprocessor/parse we still don't have results (e.g. preprocessor failed or parse failed),
        // perform the original tolerant fallback directly against the raw file contents (best-effort).
        if files_set.is_empty()
            && modules_json.is_empty()
            && total_pipelines == 0
            && total_steps == 0
        {
            let content = raw;

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
                        // determine downloaded: if local module (starts with '.') check resolved path, otherwise check phlow_packages
                        let mut downloaded = String::new();
                        let pp_path = format!("phlow_packages/{}", clean);
                        let pp = Path::new(&pp_path);
                        if pp.exists() {
                            downloaded = pp.to_string_lossy().to_string();
                        }
                        if mn_str.starts_with('.') {
                            let base = main_path.parent().unwrap_or(Path::new("."));
                            let mut candidate = base.join(&mn_str);
                            if candidate.is_dir() {
                                let mut found = None;
                                for c in ["main.phlow", "mod.phlow", "module.phlow"] {
                                    let p = candidate.join(c);
                                    if p.exists() {
                                        found = Some(p);
                                        break;
                                    }
                                }
                                if let Some(p) = found {
                                    candidate = p;
                                }
                            } else if candidate.extension().is_none() {
                                let mut with_ext = candidate.clone();
                                with_ext.set_extension("phlow");
                                if with_ext.exists() {
                                    candidate = with_ext;
                                }
                            }
                            if candidate.exists() {
                                if candidate.is_dir() {
                                    downloaded = candidate.to_string_lossy().to_string();
                                } else if let Some(p) = candidate.parent() {
                                    downloaded = p.to_string_lossy().to_string();
                                }
                            }
                        }
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
                        total_steps += steps_count;
                        total_pipelines += 1;
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
    })
}

pub async fn analyze(
    script_target: &str,
    include_files: bool,
    include_modules: bool,
    include_total_steps: bool,
    include_total_pipelines: bool,
    include_inner: bool,
) -> Result<Value, LoaderError> {
    let mut visited: HashSet<String> = HashSet::new();
    analyze_internal(
        script_target,
        include_files,
        include_modules,
        include_total_steps,
        include_total_pipelines,
        include_inner,
        &mut visited,
    )
    .await
}
