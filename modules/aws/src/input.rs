use phlow_sdk::prelude::*;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub enum AwsAction {
    // S3
    S3PutObject,
    S3GetObject,
    S3DeleteObject,
    S3ListObjects,
    // Placeholders for future services (SQS/SNS/DynamoDB)
    SqsSendMessage,
    SqsReceiveMessages,
    SqsDeleteMessage,
    SqsPurgeQueue,
    SnsPublish,
    DynamoDbPutItem,
    DynamoDbGetItem,
    DynamoDbUpdateItem,
    DynamoDbDeleteItem,
    DynamoDbQuery,
    DynamoDbScan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AwsApi {
    // S3
    S3PutObject(S3PutObjectBody),
    S3GetObject(S3GetObjectBody),
    S3DeleteObject(S3DeleteObjectBody),
    S3ListObjects(S3ListObjectsBody),
    // Others (not implemented yet)
    SqsSendMessage,
    SqsReceiveMessages,
    SqsDeleteMessage,
    SqsPurgeQueue,
    SnsPublish,
    DynamoDbPutItem,
    DynamoDbGetItem,
    DynamoDbUpdateItem,
    DynamoDbDeleteItem,
    DynamoDbQuery,
    DynamoDbScan,
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

        let (action, with) = match action_str {
            // S3
            "s3_put_object" => {
                let bucket = value
                    .get("bucket")
                    .ok_or_else(|| "missing field 'bucket' for s3_put_object".to_string())?
                    .to_string();
                let key = value
                    .get("key")
                    .ok_or_else(|| "missing field 'key' for s3_put_object".to_string())?
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
            "s3_get_object" => {
                let bucket = value
                    .get("bucket")
                    .ok_or_else(|| "missing field 'bucket' for s3_get_object".to_string())?
                    .to_string();
                let key = value
                    .get("key")
                    .ok_or_else(|| "missing field 'key' for s3_get_object".to_string())?
                    .to_string();
                let as_base64 = value.get("as_base64").and_then(|v| v.as_bool().cloned());
                (
                    AwsAction::S3GetObject,
                    AwsApi::S3GetObject(S3GetObjectBody { bucket, key, as_base64 }),
                )
            }
            "s3_delete_object" => {
                let bucket = value
                    .get("bucket")
                    .ok_or_else(|| "missing field 'bucket' for s3_delete_object".to_string())?
                    .to_string();
                let key = value
                    .get("key")
                    .ok_or_else(|| "missing field 'key' for s3_delete_object".to_string())?
                    .to_string();
                (
                    AwsAction::S3DeleteObject,
                    AwsApi::S3DeleteObject(S3DeleteObjectBody { bucket, key }),
                )
            }
            "s3_list_objects" => {
                let bucket = value
                    .get("bucket")
                    .ok_or_else(|| "missing field 'bucket' for s3_list_objects".to_string())?
                    .to_string();
                let prefix = value.get("prefix").map(|v| v.to_string());
                let max_keys = value.get("max_keys").map(|v| v.to_i64().unwrap() as i32);
                let continuation_token = value.get("continuation_token").map(|v| v.to_string());
                (
                    AwsAction::S3ListObjects,
                    AwsApi::S3ListObjects(S3ListObjectsBody { bucket, prefix, max_keys, continuation_token }),
                )
            }

            // others, not implemented yet
            "sqs_send_message" => (AwsAction::SqsSendMessage, AwsApi::SqsSendMessage),
            "sqs_receive_messages" => (AwsAction::SqsReceiveMessages, AwsApi::SqsReceiveMessages),
            "sqs_delete_message" => (AwsAction::SqsDeleteMessage, AwsApi::SqsDeleteMessage),
            "sqs_purge_queue" => (AwsAction::SqsPurgeQueue, AwsApi::SqsPurgeQueue),
            "sns_publish" => (AwsAction::SnsPublish, AwsApi::SnsPublish),
            "dynamodb_put_item" => (AwsAction::DynamoDbPutItem, AwsApi::DynamoDbPutItem),
            "dynamodb_get_item" => (AwsAction::DynamoDbGetItem, AwsApi::DynamoDbGetItem),
            "dynamodb_update_item" => (AwsAction::DynamoDbUpdateItem, AwsApi::DynamoDbUpdateItem),
            "dynamodb_delete_item" => (AwsAction::DynamoDbDeleteItem, AwsApi::DynamoDbDeleteItem),
            "dynamodb_query" => (AwsAction::DynamoDbQuery, AwsApi::DynamoDbQuery),
            "dynamodb_scan" => (AwsAction::DynamoDbScan, AwsApi::DynamoDbScan),
            _ => return Err(format!("invalid action: {}", action_str)),
        };

        Ok(AwsInput { action, with })
    }
}
