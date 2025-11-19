use phlow_sdk::prelude::*;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub enum FsAction {
    Read(FsReadBody),
    Write(FsWriteBody),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsReadBody {
    pub path: String,
    pub recursive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsWriteBody {
    pub path: String,
    pub content: Option<Value>,
    pub recursive: bool,
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsInput {
    pub action: FsAction,
}

impl TryFrom<Value> for FsInput {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // Expect: action = "read" | "write"
        let action = value
            .get("action")
            .ok_or_else(|| "missing field 'action'".to_string())?
            .as_str();

        let path = value
            .get("path")
            .ok_or_else(|| "missing field 'path'".to_string())?
            .to_string();

        match action {
            "read" => {
                let recursive = value
                    .get("recursive")
                    .and_then(|v| v.as_bool().cloned())
                    .unwrap_or(false);
                Ok(FsInput {
                    action: FsAction::Read(FsReadBody { path, recursive }),
                })
            }
            "write" => {
                let content = value.get("content").cloned();
                let recursive = value
                    .get("recursive")
                    .and_then(|v| v.as_bool().cloned())
                    .unwrap_or(true);
                let force = value
                    .get("force")
                    .and_then(|v| v.as_bool().cloned())
                    .unwrap_or(false);
                Ok(FsInput {
                    action: FsAction::Write(FsWriteBody {
                        path,
                        content,
                        recursive,
                        force,
                    }),
                })
            }
            _ => Err("invalid action: expected 'read' or 'write'".to_string()),
        }
    }
}
