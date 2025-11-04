use phlow_sdk::prelude::*;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub enum AwsAction {
    // S3
    S3PutObject,
    S3GetObject,
    S3DeleteObject,
    S3ListObjects,
    // SQS
    SqsSendMessage,
    SqsReceiveMessages,
    SqsDeleteMessage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AwsApi {
    // S3
    S3PutObject(S3PutObjectBody),
    S3GetObject(S3GetObjectBody),
    S3DeleteObject(S3DeleteObjectBody),
    S3ListObjects(S3ListObjectsBody),
    // SQS
    SqsSendMessage(SqsSendMessageBody),
    SqsReceiveMessages(SqsReceiveMessagesBody),
    SqsDeleteMessage(SqsDeleteMessageBody),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AwsInput {
    pub action: AwsAction,
    pub with: AwsApi,
}

// -----------------
// S3 bodies
// -----------------

// -----------------
// SQS bodies
// -----------------

#[derive(Debug, Clone, PartialEq)]
pub struct S3PutObjectBody {
    pub bucket: String,
    pub key: String,
    pub path: Option<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct SqsSendMessageBody {
    pub queue_url: Option<String>,
    pub queue_name: Option<String>,
    pub message_body: String,
    pub delay_seconds: Option<i32>,
    pub message_group_id: Option<String>,
    pub message_deduplication_id: Option<String>,
    pub message_attributes: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SqsReceiveMessagesBody {
    pub queue_url: Option<String>,
    pub queue_name: Option<String>,
    pub max_number_of_messages: Option<i32>,
    pub wait_time_seconds: Option<i32>,
    pub visibility_timeout: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SqsDeleteMessageBody {
    pub queue_url: Option<String>,
    pub queue_name: Option<String>,
    pub receipt_handle: String,
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

                    let path = value.get("path").map(|v| v.to_string());
                    let content = value.get("content").map(|v| v.to_string());
                    let content_base64 = value.get("content_base64").map(|v| v.to_string());
                    let content_type = value.get("content_type").map(|v| v.to_string());
                    let acl = value.get("acl").map(|v| v.to_string());

                    (
                        AwsAction::S3PutObject,
                        AwsApi::S3PutObject(S3PutObjectBody {
                            bucket,
                            key,
                            path,
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
            "sqs" => match method_str {
                "send_message" => {
                    let message_body = value
                        .get("message_body")
                        .ok_or_else(|| "missing field 'message_body' for send_message".to_string())?
                        .to_string();

                    let queue_url = value.get("queue_url").map(|v| v.to_string());
                    let queue_name = value.get("queue_name").map(|v| v.to_string());
                    let delay_seconds = value
                        .get("delay_seconds")
                        .and_then(|v| v.to_i64())
                        .map(|v| v as i32);
                    let message_group_id = value.get("message_group_id").map(|v| v.to_string());
                    let message_deduplication_id =
                        value.get("message_deduplication_id").map(|v| v.to_string());
                    let message_attributes = value.get("message_attributes").cloned();

                    (
                        AwsAction::SqsSendMessage,
                        AwsApi::SqsSendMessage(SqsSendMessageBody {
                            queue_url,
                            queue_name,
                            message_body,
                            delay_seconds,
                            message_group_id,
                            message_deduplication_id,
                            message_attributes,
                        }),
                    )
                }
                "receive_messages" => {
                    let queue_url = value.get("queue_url").map(|v| v.to_string());
                    let queue_name = value.get("queue_name").map(|v| v.to_string());
                    let max_number_of_messages = value
                        .get("max_number_of_messages")
                        .and_then(|v| v.to_i64())
                        .map(|v| v as i32);
                    let wait_time_seconds = value
                        .get("wait_time_seconds")
                        .and_then(|v| v.to_i64())
                        .map(|v| v as i32);
                    let visibility_timeout = value
                        .get("visibility_timeout")
                        .and_then(|v| v.to_i64())
                        .map(|v| v as i32);

                    (
                        AwsAction::SqsReceiveMessages,
                        AwsApi::SqsReceiveMessages(SqsReceiveMessagesBody {
                            queue_url,
                            queue_name,
                            max_number_of_messages,
                            wait_time_seconds,
                            visibility_timeout,
                        }),
                    )
                }
                "delete_message" => {
                    let receipt_handle = value
                        .get("receipt_handle")
                        .ok_or_else(|| {
                            "missing field 'receipt_handle' for delete_message".to_string()
                        })?
                        .to_string();
                    let queue_url = value.get("queue_url").map(|v| v.to_string());
                    let queue_name = value.get("queue_name").map(|v| v.to_string());

                    (
                        AwsAction::SqsDeleteMessage,
                        AwsApi::SqsDeleteMessage(SqsDeleteMessageBody {
                            queue_url,
                            queue_name,
                            receipt_handle,
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
