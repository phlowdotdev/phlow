mod input;

use base64::Engine;
use input::{FsAction, FsInput, FsReadBody, FsWriteBody};
use phlow_sdk::prelude::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::task;

create_step!(fs(setup));

macro_rules! ok_response {
    ($data:expr) => {
        Value::from(json!({ "success": true, "data": $data }))
    };
}
macro_rules! err_response {
    ($msg:expr) => {
        Value::from(json!({ "success": false, "error": $msg }))
    };
}

pub async fn fs(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    for package in rx {
        let input_value = package.input().unwrap_or(Value::Null);
        let parsed = match FsInput::try_from(input_value) {
            Ok(v) => v,
            Err(e) => {
                sender_safe!(package.sender, err_response!(e).into());
                continue;
            }
        };

        let resp = match parsed.action {
            FsAction::Read(body) => handle_read(body).await,
            FsAction::Write(body) => handle_write(body).await,
        };

        sender_safe!(package.sender, resp.into());
    }

    Ok(())
}

async fn handle_read(body: FsReadBody) -> Value {
    // offload blocking IO to blocking thread
    match task::spawn_blocking(move || read_impl(&body)).await {
        Ok(Ok(v)) => ok_response!(v),
        Ok(Err(e)) => err_response!(e),
        Err(e) => err_response!(format!("task join error: {}", e)),
    }
}

fn read_impl(body: &FsReadBody) -> Result<Value, String> {
    let p = Path::new(&body.path);
    if !p.exists() {
        return Err(format!("path not found: {}", body.path));
    }

    if p.is_file() {
        let metadata = fs::metadata(p).map_err(|e| e.to_string())?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let bytes = fs::read(p).map_err(|e| e.to_string())?;
        match String::from_utf8(bytes.clone()) {
            Ok(text) => Ok(json!({
                "kind": "file",
                "path": body.path,
                "encoding": "utf8",
                "content": text,
                "size": metadata.len(),
                "modified": modified,
            })
            .into()),
            Err(_) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
                Ok(json!({
                    "kind": "file",
                    "path": body.path,
                    "encoding": "base64",
                    "content_base64": b64,
                    "size": metadata.len(),
                    "modified": modified,
                })
                .into())
            }
        }
    } else {
        // Directory listing
        let items = if body.recursive {
            list_dir_recursive(p).map_err(|e| e.to_string())?
        } else {
            list_dir_shallow(p).map_err(|e| e.to_string())?
        };
        Ok(json!({
            "kind": "dir",
            "path": body.path,
            "items": items,
        })
        .into())
    }
}

fn list_dir_shallow(p: &Path) -> Result<Vec<Value>, std::io::Error> {
    let mut out = Vec::new();
    for entry in fs::read_dir(p)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let meta = entry.metadata()?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let kind = if meta.is_dir() { "dir" } else { "file" };
        out.push(json!({
            "name": name,
            "path": path.to_string_lossy(),
            "type": kind,
            "size": if meta.is_file() { Some(meta.len()) } else { None::<u64> },
            "modified": modified,
        }));
    }
    Ok(out.into_iter().map(Value::from).collect())
}

fn list_dir_recursive(p: &Path) -> Result<Vec<Value>, std::io::Error> {
    let mut acc = Vec::new();
    fn walk(path: &Path, acc: &mut Vec<Value>) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let child = entry.path();
            let meta = entry.metadata()?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let kind = if meta.is_dir() { "dir" } else { "file" };
            acc.push(
                json!({
                    "name": entry.file_name().to_string_lossy(),
                    "path": child.to_string_lossy(),
                    "type": kind,
                    "size": if meta.is_file() { Some(meta.len()) } else { None::<u64> },
                    "modified": modified,
                })
                .into(),
            );
            if meta.is_dir() {
                walk(&child, acc)?;
            }
        }
        Ok(())
    }
    walk(p, &mut acc)?;
    Ok(acc)
}

async fn handle_write(body: FsWriteBody) -> Value {
    match task::spawn_blocking(move || write_impl(&body)).await {
        Ok(Ok(v)) => ok_response!(v),
        Ok(Err(e)) => err_response!(e),
        Err(e) => err_response!(format!("task join error: {}", e)),
    }
}

fn write_impl(body: &FsWriteBody) -> Result<Value, String> {
    let p = PathBuf::from(&body.path);
    match &body.content {
        Some(content_val) => {
            // File write
            if let Some(parent) = p.parent() {
                if body.recursive {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                } else if !parent.exists() {
                    return Err(format!(
                        "parent directory does not exist: {}",
                        parent.to_string_lossy()
                    ));
                }
            }

            if p.exists() && !body.force {
                return Err(format!(
                    "file already exists (use force: true): {}",
                    body.path
                ));
            }

            let mut file = fs::File::create(&p).map_err(|e| e.to_string())?;
            let text = match content_val {
                Value::String(_) => content_val.to_string(),
                other => other.to_string(),
            };
            file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
            let written = text.as_bytes().len() as u64;

            Ok(json!({
                "kind": "file",
                "path": body.path,
                "written": written,
                "overwritten": p.exists() && body.force,
            })
            .into())
        }
        None => {
            // Directory create
            if body.recursive {
                fs::create_dir_all(&p).map_err(|e| e.to_string())?;
            } else {
                fs::create_dir(&p).map_err(|e| e.to_string())?;
            }
            Ok(json!({
                "kind": "dir",
                "path": body.path,
                "created": true
            })
            .into())
        }
    }
}
