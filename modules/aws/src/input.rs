use phlow_sdk::prelude::*;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub enum AwsAction {
    // S3
    S3PutObject,
    S3GetObject,
    S3DeleteObject,
    S3ListObjects,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AwsApi {
    // S3
    S3PutObject(S3PutObjectBody),
    S3GetObject(S3GetObjectBody),
    S3DeleteObject(S3DeleteObjectBody),
    S3ListObjects(S3ListObjectsBody),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AwsInput {
    pub action: AwsAction,
    pub with: AwsApi,
}

// -----------------
// S3 bodies
// -----------------

#[derive(Debug, Clone, PartialEq)]
pub struct S3PutObjectBody {
    pub bucket: String,
    pub key: String,
    pub content: Option<String>,
    pub content_base64: Option<String>,
    pub content_type: Option<String>,
    pub acl: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct S3GetObjectBody {
    pub bucket: String,
    pub key: String,
    pub as_base64: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct S3DeleteObjectBody {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct S3ListObjectsBody {
    pub bucket: String,
    pub prefix: Option<String>,
    pub max_keys: Option<i32>,
    pub continuation_token: Option<String>,
}

impl TryFrom<Value> for AwsInput {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let action_str = value
            .get("action")
            .ok_or_else(|| "missing field 'action' for AwsInput".to_string())?
            .as_str();

        let method_str = {
            if let Some(method) = value.get("method") {
                method.as_str()
            } else {
                let args = value
                    .get("args")
                    .ok_or_else(|| "missing field 'args' for AwsInput".to_string())?;

                let method = args
                    .get(0)
                    .ok_or_else(|| "missing field 'method' for AwsInput".to_string())?
                    .as_str();

                method
            }
        };

        let (action, with) = match action_str {
            "s3" => match method_str {
                "put_object" => {
                    let bucket = value
                        .get("bucket")
                        .ok_or_else(|| "missing field 'bucket' for put_object".to_string())?
                        .to_string();
                    let key = value
                        .get("key")
                        .ok_or_else(|| "missing field 'key' for put_object".to_string())?
                        .to_string();

                    let content = value.get("content").map(|v| v.to_string());
                    let content_base64 = value.get("content_base64").map(|v| v.to_string());
                    let content_type = value.get("content_type").map(|v| v.to_string());
                    let acl = value.get("acl").map(|v| v.to_string());

                    (
                        AwsAction::S3PutObject,
                        AwsApi::S3PutObject(S3PutObjectBody {
                            bucket,
                            key,
                            content,
                            content_base64,
                            content_type,
                            acl,
                        }),
                    )
                }
                "get_object" => {
                    let bucket = value
                        .get("bucket")
                        .ok_or_else(|| "missing field 'bucket' for get_object".to_string())?
                        .to_string();
                    let key = value
                        .get("key")
                        .ok_or_else(|| "missing field 'key' for get_object".to_string())?
                        .to_string();
                    let as_base64 = value.get("as_base64").and_then(|v| v.as_bool().cloned());
                    (
                        AwsAction::S3GetObject,
                        AwsApi::S3GetObject(S3GetObjectBody {
                            bucket,
                            key,
                            as_base64,
                        }),
                    )
                }
                "delete_object" => {
                    let bucket = value
                        .get("bucket")
                        .ok_or_else(|| "missing field 'bucket' for delete_object".to_string())?
                        .to_string();
                    let key = value
                        .get("key")
                        .ok_or_else(|| "missing field 'key' for delete_object".to_string())?
                        .to_string();
                    (
                        AwsAction::S3DeleteObject,
                        AwsApi::S3DeleteObject(S3DeleteObjectBody { bucket, key }),
                    )
                }
                "list_objects" => {
                    let bucket = value
                        .get("bucket")
                        .ok_or_else(|| "missing field 'bucket' for list_objects".to_string())?
                        .to_string();
                    let prefix = value.get("prefix").map(|v| v.to_string());
                    let max_keys = value.get("max_keys").map(|v| v.to_i64().unwrap() as i32);
                    let continuation_token = value.get("continuation_token").map(|v| v.to_string());
                    (
                        AwsAction::S3ListObjects,
                        AwsApi::S3ListObjects(S3ListObjectsBody {
                            bucket,
                            prefix,
                            max_keys,
                            continuation_token,
                        }),
                    )
                }
                _ => return Err(format!("invalid action: {}", method_str)),
            },
            _ => return Err(format!("invalid action: {}", action_str)),
        };

        Ok(AwsInput { action, with })
    }
}
