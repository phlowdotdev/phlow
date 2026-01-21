use super::Error;
use crate::preprocessor::preprocessor;
use crate::settings::PrintOutput;
use flate2::read::GzDecoder;
use log::debug;
use phlow_sdk::{prelude::*, valu3};
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::Archive;
use zip::ZipArchive;

pub struct ScriptLoaded {
    pub script: Value,
    pub script_file_path: String,
}

use crate::analyzer::Analyzer;

pub async fn load_script(
    script_target: &str,
    print_yaml: bool,
    print_output: PrintOutput,
    analyzer: Option<&Analyzer>,
) -> Result<ScriptLoaded, Error> {
    let script_file_path = match resolve_script_path(script_target).await {
        Ok(path) => path,
        Err(err) => return Err(err),
    };

    let file: String = match std::fs::read_to_string(&script_file_path) {
        Ok(file) => file,
        Err(_) => return Err(Error::ModuleNotFound(script_file_path.to_string())),
    };

    let script = resolve_script(&file, script_file_path.clone(), print_yaml, print_output)
        .map_err(|err| {
            Error::ModuleLoaderError(format!(
                "Failed to resolve script: {}. Error: {}",
                script_file_path, err
            ))
        })?;

    // If analyzer was provided and is enabled, run it using the script target
    if let Some(a) = analyzer {
        if a.enabled {
            // run analyzer but ignore errors (we don't want to fail loading because of analyzer)
            match a.run().await {
                Ok(result) => {
                    a.display(&result);
                }
                Err(err) => {
                    eprintln!("Analyzer error: {:?}", err);
                }
            }
        }
    }

    Ok(ScriptLoaded {
        script,
        script_file_path,
    })
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
    use git2::{FetchOptions, RemoteCallbacks, build::RepoBuilder};

    let remote_path = get_remote_path()?;

    let mut callbacks = RemoteCallbacks::new();

    // Add certificate check callback to handle SSH host key verification
    callbacks.certificate_check(|_cert, _valid| {
        // Accept all certificates - you might want to implement proper host key checking
        // in a production environment by verifying against known_hosts
        Ok(git2::CertificateCheckStatus::CertificateOk)
    });

    if url.contains("@") {
        debug!("Using SSH authentication for Git: {}", url);
        if let Some(ssh_user) = url.split('@').next() {
            let id_rsa_path: String = match std::env::var("PHLOW_REMOTE_ID_RSA_PATH") {
                Ok(path) => path,
                Err(_) => {
                    let home = std::env::var("HOME").map_err(|_| {
                        Error::ModuleLoaderError(
                            "HOME not set and PHLOW_REMOTE_ID_RSA_PATH not set".to_string(),
                        )
                    })?;
                    format!("{}/.ssh/id_rsa", home)
                }
            };

            debug!("Using SSH user: {}", ssh_user);
            debug!("Using SSH key path: {}", id_rsa_path);

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
                    None,
                    std::path::Path::new(&id_rsa_path),
                    None,
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

    // Check if a specific file is requested via environment variable
    let file_path = if let Ok(main_file) = std::env::var("PHLOW_MAIN_FILE") {
        let specific_file_path = remote_path.join(&main_file);
        if specific_file_path.exists() {
            specific_file_path.to_str().unwrap_or_default().to_string()
        } else {
            return Err(Error::MainNotFound(format!(
                "Specified file '{}' not found in repository '{}'",
                main_file, url
            )));
        }
    } else {
        find_default_file(&remote_path).ok_or_else(|| Error::MainNotFound(url.to_string()))?
    };

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

    // Check if a specific file is requested via environment variable
    let main_path = if let Ok(main_file) = std::env::var("PHLOW_MAIN_FILE") {
        println!("Using specified main file: {}", main_file);
        let specific_file_path = effective_path.join(&main_file);
        if specific_file_path.exists() {
            specific_file_path.to_str().unwrap_or_default().to_string()
        } else {
            return Err(Error::MainNotFound(format!(
                "Specified file '{}' not found in downloaded archive '{}'",
                main_file, url
            )));
        }
    } else {
        find_default_file(&effective_path).ok_or_else(|| Error::MainNotFound(url.to_string()))?
    };

    Ok(main_path)
}

async fn resolve_script_path(script_path: &str) -> Result<String, Error> {
    let (target, branch) = if script_path.contains('#') {
        let parts: Vec<&str> = script_path.split('#').collect();
        (parts[0], Some(parts[1]))
    } else {
        (script_path, None)
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
            .ok_or_else(|| Error::MainNotFound(script_path.to_string()));
    } else if target_path.exists() {
        return Ok(target.to_string());
    }

    Err(Error::MainNotFound(script_path.to_string()))
}

fn resolve_script(
    file: &str,
    main_file_path: String,
    print_yaml: bool,
    print_output: PrintOutput,
) -> Result<Value, Error> {
    let mut value: Value = {
        let script_path = Path::new(&main_file_path)
            .parent()
            .unwrap_or_else(|| Path::new("."));

        // Se a extensão do arquivo for yaml ou yml, não executar o preprocessor
        let extension = Path::new(&main_file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let script: String = if extension == "yaml" || extension == "yml" || extension == "json" {
            // Usar o conteúdo original do arquivo quando for YAML
            file.to_string()
        } else {
            preprocessor(&file, script_path, print_yaml, print_output).map_err(|errors| {
                eprintln!("❌ Failed to transform YAML file: {}", main_file_path);
                Error::ModuleLoaderError(format!(
                    "YAML transformation failed with {} error(s)",
                    errors.len()
                ))
            })?
        };

        if let Ok(yaml_show) = std::env::var("PHLOW_SCRIPT_SHOW") {
            if yaml_show == "true" {
                println!("YAML: {}", script);
            }
        }

        if extension == "json" {
            println!("Parsing JSON script");
            valu3::value::Value::json_to_value(&script).map_err(Error::LoaderErrorJsonValu3)?
        } else {
            serde_yaml::from_str::<Value>(&script).map_err(Error::LoaderErrorScript)?
        }
    };

    if value.get("steps").is_none() {
        return Err(Error::StepsNotDefined);
    }

    if let Some(modules) = value.get("modules") {
        if !modules.is_array() {
            return Err(Error::ModuleLoaderError("Modules not an array".to_string()));
        }

        value.insert("modules", modules.clone());
    } else {
        // Se modules não foi definido, criar uma lista vazia
        value.insert("modules", Value::Array(phlow_sdk::prelude::Array::new()));
    }

    Ok(value)
}

pub fn load_external_module_info(module: &str) -> Value {
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
        let value: serde_yaml::Value = match serde_yaml::from_str::<serde_yaml::Value>(&file) {
            Ok(value) => value,
            Err(err) => {
                debug!(
                    "Failed to parse module metadata {}: {}",
                    module_path, err
                );
                return Value::Null;
            }
        };

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

    let mut value: Value = match serde_yaml::from_str::<Value>(&file) {
        Ok(value) => value,
        Err(err) => {
            debug!(
                "Failed to parse module metadata {}: {}",
                module_path, err
            );
            return Value::Null;
        }
    };

    value.insert("input_order".to_string(), input_order.to_value());

    value
}

pub fn load_local_module_info(local_path: &str) -> Value {
    debug!("load_local_module_info");
    let module_path = format!("{}/phlow.yaml", local_path);

    if !Path::new(&module_path).exists() {
        debug!("phlow.yaml not exists");
        return Value::Null;
    }

    let file = match std::fs::read_to_string(&module_path) {
        Ok(file) => file,
        Err(_) => return Value::Null,
    };

    let mut input_order = Vec::new();

    {
        let value: serde_yaml::Value = match serde_yaml::from_str::<serde_yaml::Value>(&file) {
            Ok(value) => value,
            Err(err) => {
                debug!(
                    "Failed to parse module metadata {}: {}",
                    module_path, err
                );
                return Value::Null;
            }
        };

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

    let mut value: Value = match serde_yaml::from_str::<Value>(&file) {
        Ok(value) => value,
        Err(err) => {
            debug!(
                "Failed to parse module metadata {}: {}",
                module_path, err
            );
            return Value::Null;
        }
    };

    value.insert("input_order".to_string(), input_order.to_value());

    value
}

fn find_default_file(base: &PathBuf) -> Option<String> {
    if base.is_file() {
        return Some(base.to_str().unwrap_or_default().to_string());
    }

    if base.is_dir() {
        {
            let mut base_path = base.clone();
            base_path.set_extension("phlow");

            if base_path.exists() {
                return Some(base_path.to_str().unwrap_or_default().to_string());
            }
        }

        let files = vec!["main.phlow", "mod.phlow", "module.phlow"];

        for file in files {
            let file_path = base.join(file);

            if file_path.exists() {
                return Some(file_path.to_str().unwrap_or_default().to_string());
            }
        }
    }

    None
}
