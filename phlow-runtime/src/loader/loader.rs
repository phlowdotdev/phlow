use super::Error;
use crate::yaml::yaml_helpers_transform;
use flate2::read::GzDecoder;
use git2::Repository;
use phlow_sdk::prelude::*;
use reqwest::Client;
use std::fs::{self, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::Archive;
use zip::ZipArchive;

pub async fn load_main(main_target: &str) -> Result<Value, Error> {
    let main_file_path = match load_remote_main(main_target).await {
        Ok(path) => path,
        Err(_) => return Err(Error::ModuleNotFound(main_target.to_string())),
    };

    let file: String = match std::fs::read_to_string(&main_file_path) {
        Ok(file) => file,
        Err(_) => return Err(Error::ModuleNotFound(main_file_path.to_string())),
    };

    resolve_main(&file, main_file_path)
        .map_err(|_| Error::ModuleLoaderError("Module not found".to_string()))
}

fn clone_git_repo(url: &str, branch: Option<&str>) -> Result<PathBuf, Error> {
    let remote_path = PathBuf::from("phlow_remote");

    // Check if the directory already exists, if so, remove it
    if remote_path.exists() {
        fs::remove_dir_all(&remote_path).map_err(|e| {
            Error::ModuleLoaderError(format!("Failed to remove existing dir: {}", e))
        })?;
    }

    fs::create_dir_all(&remote_path)
        .map_err(|e| Error::ModuleLoaderError(format!("Failed to create remote dir: {}", e)))?;

    let repo = Repository::clone(url, &remote_path)
        .map_err(|e| Error::ModuleLoaderError(format!("Git clone failed: {}", e)))?;

    if let Some(branch_name) = branch {
        let (object, reference) = repo.revparse_ext(branch_name).map_err(|e| {
            Error::ModuleLoaderError(format!("Branch `{}` not found: {}", branch_name, e))
        })?;

        repo.set_head(
            reference
                .and_then(|r| r.name().map(|s| s.to_string()))
                .ok_or_else(|| Error::ModuleLoaderError("Invalid branch ref".to_string()))?
                .as_str(),
        )
        .map_err(|e| Error::ModuleLoaderError(format!("Failed to set HEAD: {}", e)))?;

        repo.checkout_tree(&object, None)
            .map_err(|e| Error::ModuleLoaderError(format!("Checkout failed: {}", e)))?;
    }

    Ok(remote_path)
}

async fn load_remote_main(main_target: &str) -> Result<String, Error> {
    let (target, branch) = if main_target.contains('#') {
        let parts: Vec<&str> = main_target.splitn(2, '#').collect();
        (parts[0], Some(parts[1]))
    } else {
        (main_target, None)
    };

    if target.trim_end().ends_with(".git") {
        let repo_path = clone_git_repo(target, branch)?;
        return Ok(repo_path.join("main.yaml").to_str().unwrap().to_string());
    }

    if target.starts_with("http://") || target.starts_with("https://") {
        let client = Client::new();
        let response = client
            .get(target)
            .send()
            .await
            .map_err(Error::GetFileError)?;
        let bytes = response.bytes().await.map_err(Error::BufferError)?;

        let remote_path = PathBuf::from("remote");
        fs::create_dir_all(&remote_path).map_err(Error::FileCreateError)?;

        if let Ok(_) = Archive::new(GzDecoder::new(Cursor::new(bytes.clone()))).unpack(&remote_path)
        {
            return Ok(remote_path.join("main.yaml").to_str().unwrap().to_string());
        }

        if let Ok(mut archive) = ZipArchive::new(Cursor::new(bytes.clone())) {
            archive
                .extract(&remote_path)
                .map_err(Error::ZipErrorError)?;
            return Ok(remote_path.join("main.yaml").to_str().unwrap().to_string());
        }

        let file_path = remote_path.join("main.yaml");
        File::create(&file_path)
            .and_then(|mut f| std::io::copy(&mut Cursor::new(bytes), &mut f))
            .map_err(Error::CopyError)?;

        return Ok(file_path.to_str().unwrap().to_string());
    }

    if PathBuf::from(target).is_dir() {
        if let Some(main_path) = find_default_file(target) {
            if PathBuf::from(&main_path).exists() {
                return Ok(main_path);
            }
        }
    } else if PathBuf::from(target).exists() {
        return Ok(target.to_string());
    }

    Err(Error::MainNotFound(main_target.to_string()))
}

fn resolve_main(file: &str, main_file_path: String) -> Result<Value, Error> {
    let mut value: Value = {
        let yaml_path = Path::new(&main_file_path)
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let yaml: String = yaml_helpers_transform(&file, yaml_path);

        if let Ok(yaml_show) = std::env::var("PHLOW_YAML_SHOW") {
            if yaml_show == "true" {
                println!("YAML: {}", yaml);
            }
        }

        serde_yaml::from_str(&yaml).map_err(Error::LoaderErrorYaml)?
    };

    if value.get("steps").is_none() {
        return Err(Error::StepsNotDefined);
    }

    if let Some(modules) = value.get("modules") {
        if !modules.is_array() {
            return Err(Error::ModuleLoaderError("Modules not an array".to_string()));
        }

        let modules_array = modules.as_array().unwrap();
        let mut module_list = Vec::new();

        for item in modules_array {
            let mut module = item.clone();

            let module_name = module.get("module").unwrap().to_string();
            let module_info = load_external_module_info(&module_name);

            module.insert("info", module_info);
            module_list.push(module);
        }

        value.insert("modules", module_list.to_value());
    } else {
        return Err(Error::ModuleLoaderError("Modules not found".to_string()));
    }

    Ok(value)
}

fn load_external_module_info(module: &str) -> Value {
    let module_path = format!("phlow_packages/{}/phlow.yaml", module);
    if !Path::new(&module_path).exists() {
        return Value::Null;
    }

    let file = match std::fs::read_to_string(&module_path) {
        Ok(file) => file,
        Err(_) => return Value::Null,
    };

    let mut input_order = Vec::new();

    {
        let value: serde_yaml::Value = serde_yaml::from_str::<serde_yaml::Value>(&file)
            .map_err(Error::LoaderErrorYaml)
            .unwrap();

        if let Some(input) = value.get("input") {
            if let serde_yaml::Value::Mapping(input) = input {
                if let Some(serde_yaml::Value::String(input_type)) = input.get("type") {
                    if input_type == "object" {
                        if let Some(serde_yaml::Value::Mapping(properties)) =
                            input.get(&serde_yaml::Value::String("properties".to_string()))
                        {
                            for (key, _) in properties {
                                if let serde_yaml::Value::String(key) = key {
                                    input_order.push(key.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        drop(value)
    }

    let mut value: Value = serde_yaml::from_str::<Value>(&file)
        .map_err(Error::LoaderErrorYaml)
        .unwrap();

    value.insert("input_order".to_string(), input_order.to_value());

    value
}

fn find_default_file(base: &str) -> Option<String> {
    let files = vec!["main.yaml", "main.yml"];

    for file in files {
        let path = if base.is_empty() || base == "." {
            file.to_string()
        } else {
            format!("{}/{}", base, file)
        };

        if std::path::Path::new(&path).exists() {
            return Some(path.to_string());
        }
    }

    None
}
