use phlow_sdk::prelude::*;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub enum AwsAction {
    // S3
    S3PutObject,
    S3GetObject,
    S3DeleteObject,
    S3ListObjects,
    S3CreateBucket,
    S3DeleteBucket,
    S3GetBucketLocation,
    S3PutBucketVersioning,
    S3ListBuckets,
    S3GetObjectAttributes,
    // SQS
    SqsSendMessage,
    SqsReceiveMessages,
    SqsDeleteMessage,
    SqsCreateQueue,
    SqsDeleteQueue,
    SqsGetQueueAttributes,
    SqsSetQueueAttributes,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AwsApi {
    // S3
    S3PutObject(S3PutObjectBody),
    S3GetObject(S3GetObjectBody),
    S3DeleteObject(S3DeleteObjectBody),
    S3ListObjects(S3ListObjectsBody),
    S3CreateBucket(S3CreateBucketBody),
    S3DeleteBucket(S3DeleteBucketBody),
    S3GetBucketLocation(S3GetBucketLocationBody),
    S3PutBucketVersioning(S3PutBucketVersioningBody),
    S3ListBuckets,
    S3GetObjectAttributes(S3GetObjectAttributesBody),
    // SQS
    SqsSendMessage(SqsSendMessageBody),
    SqsReceiveMessages(SqsReceiveMessagesBody),
    SqsDeleteMessage(SqsDeleteMessageBody),
    SqsCreateQueue(SqsCreateQueueBody),
    SqsDeleteQueue(SqsDeleteQueueBody),
    SqsGetQueueAttributes(SqsGetQueueAttributesBody),
    SqsSetQueueAttributes(SqsSetQueueAttributesBody),
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
pub struct S3CreateBucketBody {
    pub bucket: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct S3DeleteBucketBody {
    pub bucket: String,
    pub force: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct S3GetBucketLocationBody {
    pub bucket: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct S3PutBucketVersioningBody {
    pub bucket: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct S3GetObjectAttributesBody {
    pub bucket: String,
    pub key: String,
    pub version_id: Option<String>,
    pub attributes: Option<Vec<String>>,
    pub object_parts: Option<Value>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct SqsCreateQueueBody {
    pub queue_name: String,
    pub attributes: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SqsDeleteQueueBody {
    pub queue_url: Option<String>,
    pub queue_name: Option<String>,
    pub force: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SqsGetQueueAttributesBody {
    pub queue_url: Option<String>,
    pub queue_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SqsSetQueueAttributesBody {
    pub queue_url: Option<String>,
    pub queue_name: Option<String>,
    pub attributes: Value,
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
                "get_object_attributes" => {
                    let bucket = value
                        .get("bucket")
                        .ok_or_else(|| {
                            "missing field 'bucket' for get_object_attributes".to_string()
                        })?
                        .to_string();
                    let key = value
                        .get("key")
                        .ok_or_else(|| "missing field 'key' for get_object_attributes".to_string())?
                        .to_string();

                    let version_id = value.get("version_id").map(|v| v.to_string());

                    let attributes = value
                        .get("attributes")
                        .and_then(|v| v.as_array().cloned())
                        .map(|arr| arr.into_iter().map(|it| it.to_string()).collect());

                    let object_parts = value.get("object_parts").cloned();

                    (
                        AwsAction::S3GetObjectAttributes,
                        AwsApi::S3GetObjectAttributes(S3GetObjectAttributesBody {
                            bucket,
                            key,
                            version_id,
                            attributes,
                            object_parts,
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
                "create_bucket" => {
                    let bucket = value
                        .get("bucket")
                        .ok_or_else(|| "missing field 'bucket' for create_bucket".to_string())?
                        .to_string();
                    let location = value.get("location").map(|v| v.to_string());
                    (
                        AwsAction::S3CreateBucket,
                        AwsApi::S3CreateBucket(S3CreateBucketBody { bucket, location }),
                    )
                }
                "delete_bucket" => {
                    let bucket = value
                        .get("bucket")
                        .ok_or_else(|| "missing field 'bucket' for delete_bucket".to_string())?
                        .to_string();
                    let force = value.get("force").and_then(|v| v.as_bool().cloned());
                    (
                        AwsAction::S3DeleteBucket,
                        AwsApi::S3DeleteBucket(S3DeleteBucketBody { bucket, force }),
                    )
                }
                "get_bucket_location" => {
                    let bucket = value
                        .get("bucket")
                        .ok_or_else(|| {
                            "missing field 'bucket' for get_bucket_location".to_string()
                        })?
                        .to_string();
                    (
                        AwsAction::S3GetBucketLocation,
                        AwsApi::S3GetBucketLocation(S3GetBucketLocationBody { bucket }),
                    )
                }
                "put_bucket_versioning" => {
                    let bucket = value
                        .get("bucket")
                        .ok_or_else(|| {
                            "missing field 'bucket' for put_bucket_versioning".to_string()
                        })?
                        .to_string();
                    let enabled = value
                        .get("enabled")
                        .and_then(|v| v.as_bool().cloned())
                        .ok_or_else(|| {
                            "missing field 'enabled' for put_bucket_versioning".to_string()
                        })?;
                    (
                        AwsAction::S3PutBucketVersioning,
                        AwsApi::S3PutBucketVersioning(S3PutBucketVersioningBody {
                            bucket,
                            enabled,
                        }),
                    )
                }
                "list_buckets" => (AwsAction::S3ListBuckets, AwsApi::S3ListBuckets),
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
                "create_queue" => {
                    let queue_name = value
                        .get("queue_name")
                        .ok_or_else(|| "missing field 'queue_name' for create_queue".to_string())?
                        .to_string();
                    let attributes = value.get("attributes").cloned();
                    (
                        AwsAction::SqsCreateQueue,
                        AwsApi::SqsCreateQueue(SqsCreateQueueBody {
                            queue_name,
                            attributes,
                        }),
                    )
                }
                "delete_queue" => {
                    let queue_url = value.get("queue_url").map(|v| v.to_string());
                    let queue_name = value.get("queue_name").map(|v| v.to_string());
                    let force = value.get("force").and_then(|v| v.as_bool().cloned());
                    (
                        AwsAction::SqsDeleteQueue,
                        AwsApi::SqsDeleteQueue(SqsDeleteQueueBody {
                            queue_url,
                            queue_name,
                            force,
                        }),
                    )
                }
                "get_queue_attributes" => {
                    let queue_url = value.get("queue_url").map(|v| v.to_string());
                    let queue_name = value.get("queue_name").map(|v| v.to_string());
                    (
                        AwsAction::SqsGetQueueAttributes,
                        AwsApi::SqsGetQueueAttributes(SqsGetQueueAttributesBody {
                            queue_url,
                            queue_name,
                        }),
                    )
                }
                "set_queue_attributes" => {
                    let queue_url = value.get("queue_url").map(|v| v.to_string());
                    let queue_name = value.get("queue_name").map(|v| v.to_string());
                    let attributes = value.get("attributes").cloned().ok_or_else(|| {
                        "missing field 'attributes' for set_queue_attributes".to_string()
                    })?;
                    (
                        AwsAction::SqsSetQueueAttributes,
                        AwsApi::SqsSetQueueAttributes(SqsSetQueueAttributesBody {
                            queue_url,
                            queue_name,
                            attributes,
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
