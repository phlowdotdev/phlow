use super::Error;
use crate::yaml::yaml_helpers_transform;
use flate2::read::GzDecoder;
use phlow_sdk::prelude::*;
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::Archive;
use zip::ZipArchive;

pub async fn load_main(main_target: &str) -> Result<Value, Error> {
    let main_file_path = match load_remote_main(main_target).await {
        Ok(path) => path,
        Err(err) => return Err(err),
    };

    let file: String = match std::fs::read_to_string(&main_file_path) {
        Ok(file) => file,
        Err(_) => return Err(Error::ModuleNotFound(main_file_path.to_string())),
    };

    resolve_main(&file, main_file_path)
        .map_err(|_| Error::ModuleLoaderError("Module not found".to_string()))
}

fn get_remote_path() -> Result<PathBuf, Error> {
    let remote_path = PathBuf::from("phlow_remote");

    if remote_path.exists() {
        // remove
        fs::remove_dir_all(&remote_path).map_err(|e| {
            Error::ModuleLoaderError(format!("Failed to remove remote path: {}", e))
        })?;
    }

    fs::create_dir_all(&remote_path)
        .map_err(|e| Error::ModuleLoaderError(format!("Failed to create remote dir: {}", e)))?;

    Ok(remote_path)
}

fn clone_git_repo(url: &str, branch: Option<&str>) -> Result<String, Error> {
    use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};

    let remote_path = get_remote_path()?;

    let mut callbacks = RemoteCallbacks::new();

    if url.contains("@") {
        if let Some(ssh_user) = url.split('@').next() {
            let id_rsa_path = std::env::var("PHLOW_REMOTE_ID_RSA_PATH")
                .unwrap_or_else(|_| format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap()));

            if !Path::new(&id_rsa_path).exists() {
                return Err(Error::ModuleLoaderError(format!(
                    "SSH key not found at path: {}",
                    id_rsa_path
                )));
            }

            let id_rsa_path = id_rsa_path.clone();

            callbacks.credentials(move |_url, username_from_url, _allowed_types| {
                git2::Cred::ssh_key(
                    username_from_url.unwrap_or(ssh_user),
                    None, // usa ~/.ssh/id_rsa.pub
                    std::path::Path::new(&id_rsa_path),
                    None, // sem passphrase
                )
            });
        }
    }

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);

    if let Some(branch_name) = branch {
        builder.branch(branch_name);
    }

    let repo = builder
        .clone(url, &remote_path)
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

    let file_path =
        find_default_file(&remote_path).ok_or_else(|| Error::MainNotFound(url.to_string()))?;

    Ok(file_path)
}

async fn download_file(url: &str, inner_folder: Option<&str>) -> Result<String, Error> {
    let client = Client::new();

    let mut request = client.get(url);

    if let Ok(auth_header) = std::env::var("PHLOW_REMOTE_HEADER_AUTHORIZATION") {
        request = request.header(AUTHORIZATION, auth_header);
    }

    let response = request.send().await.map_err(Error::GetFileError)?;
    let bytes = response.bytes().await.map_err(Error::BufferError)?;

    let remote_path = get_remote_path()?;

    if Archive::new(GzDecoder::new(Cursor::new(bytes.clone())))
        .unpack(&remote_path)
        .is_err()
    {
        if let Ok(mut archive) = ZipArchive::new(Cursor::new(bytes.clone())) {
            archive
                .extract(&remote_path)
                .map_err(Error::ZipErrorError)?;
        }
    };

    let effective_path = if let Some(inner_folder) = inner_folder {
        remote_path.join(inner_folder)
    } else {
        let entries: Vec<_> = fs::read_dir(&remote_path)
            .map_err(|e| Error::ModuleLoaderError(format!("Failed to read remote dir: {}", e)))?
            .filter_map(Result::ok)
            .collect();

        if entries.len() == 1 && entries[0].path().is_dir() {
            entries[0].path()
        } else {
            remote_path
        }
    };

    let main_path =
        find_default_file(&effective_path).ok_or_else(|| Error::MainNotFound(url.to_string()))?;

    Ok(main_path)
}

async fn load_remote_main(main_target: &str) -> Result<String, Error> {
    let (target, branch) = if main_target.contains('#') {
        let parts: Vec<&str> = main_target.split('#').collect();
        (parts[0], Some(parts[1]))
    } else {
        (main_target, None)
    };

    if target.trim_end().ends_with(".git") {
        return clone_git_repo(target, branch);
    }

    if target.starts_with("http://") || target.starts_with("https://") {
        return download_file(target, branch).await;
    }

    let target_path = PathBuf::from(target);
    if target_path.is_dir() {
        return find_default_file(&target_path)
            .ok_or_else(|| Error::MainNotFound(main_target.to_string()));
    } else if target_path.exists() {
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

fn find_default_file(base: &PathBuf) -> Option<String> {
    if base.is_file() {
        return Some(base.to_str().unwrap_or_default().to_string());
    }

    if base.is_dir() {
        let files = vec!["main.yaml", "main.yml"];

        for file in files {
            let file_path = base.join(file);

            if file_path.exists() {
                return Some(file_path.to_str().unwrap_or_default().to_string());
            }
        }
    }

    None
}
