mod input;
mod setup;

use crate::input::{AwsApi, AwsInput};
use crate::setup::Setup;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::types::ObjectCannedAcl;
use aws_sdk_sqs::Client as SqsClient;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use phlow_sdk::prelude::*;

create_step!(aws(setup));

macro_rules! success_response {
    ($data:expr) => {
        Value::from(json!({ "success": true, "data": $data }))
    };
}

macro_rules! error_response {
    ($message:expr) => {
        Value::from(json!({ "success": false, "error": $message }))
    };
}

pub async fn aws(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    let setup_cfg = Setup::try_from(setup.with)?;

    // Build S3 client once; reused for all S3 actions
    let s3_client = setup_cfg.build_s3_client().await?;
    // Build SQS client once; reused for all SQS actions
    let sqs_client = setup_cfg.build_sqs_client().await?;

    for package in rx {
        let input_value = package.input().unwrap_or(Value::Null);

        let parsed = match AwsInput::try_from(input_value) {
            Ok(p) => p,
            Err(e) => {
                sender_safe!(
                    package.sender,
                    error_response!(format!("Invalid input: {}", e)).into()
                );
                continue;
            }
        };

        let response: Value = match parsed.with {
            AwsApi::S3PutObject(body) => match handle_s3_put_object(&s3_client, body).await {
                Ok(data) => success_response!(data),
                Err(e) => error_response!(e),
            },
            AwsApi::S3GetObject(body) => match handle_s3_get_object(&s3_client, body).await {
                Ok(data) => success_response!(data),
                Err(e) => error_response!(e),
            },
            AwsApi::S3DeleteObject(body) => match handle_s3_delete_object(&s3_client, body).await {
                Ok(data) => success_response!(data),
                Err(e) => error_response!(e),
            },
            AwsApi::S3ListObjects(body) => match handle_s3_list_objects(&s3_client, body).await {
                Ok(data) => success_response!(data),
                Err(e) => error_response!(e),
            },
            AwsApi::S3CreateBucket(body) => match handle_s3_create_bucket(&s3_client, body).await {
                Ok(data) => success_response!(data),
                Err(e) => error_response!(e),
            },
            AwsApi::S3DeleteBucket(body) => match handle_s3_delete_bucket(&s3_client, body).await {
                Ok(data) => success_response!(data),
                Err(e) => error_response!(e),
            },
            AwsApi::S3GetBucketLocation(body) => {
                match handle_s3_get_bucket_location(&s3_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3PutBucketVersioning(body) => {
                match handle_s3_put_bucket_versioning(&s3_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3ListBuckets => match handle_s3_list_buckets(&s3_client).await {
                Ok(data) => success_response!(data),
                Err(e) => error_response!(e),
            },
            AwsApi::SqsSendMessage(body) => {
                match handle_sqs_send_message(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsReceiveMessages(body) => {
                match handle_sqs_receive_messages(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsDeleteMessage(body) => {
                match handle_sqs_delete_message(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsCreateQueue(body) => {
                match handle_sqs_create_queue(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsDeleteQueue(body) => {
                match handle_sqs_delete_queue(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsGetQueueAttributes(body) => {
                match handle_sqs_get_queue_attributes(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsSetQueueAttributes(body) => {
                match handle_sqs_set_queue_attributes(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
        };

        sender_safe!(package.sender, response.into());
    }

    Ok(())
}

// ----------------------
// S3 Handlers
// ----------------------

async fn handle_s3_put_object(
    client: &S3Client,
    body: crate::input::S3PutObjectBody,
) -> Result<Value, String> {
    use aws_sdk_s3::primitives::ByteStream;
    use std::path::Path;

    let mut req = client.put_object().bucket(&body.bucket).key(&body.key);

    // Prefer streaming from file path if provided
    if let Some(path) = body.path.clone() {
        let p = Path::new(&path);
        let bs = ByteStream::from_path(p)
            .await
            .map_err(|e| format!("failed to open path '{}': {}", path, e))?;
        req = req.body(bs);
    } else {
        // Content
        let bytes: Vec<u8> = if let Some(b64) = body.content_base64 {
            BASE64
                .decode(b64)
                .map_err(|e| format!("invalid base64 content: {}", e))?
        } else if let Some(text) = body.content {
            text.into_bytes()
        } else {
            return Err(
                "Missing 'path' or 'content' or 'content_base64' for s3_put_object".to_string(),
            );
        };

        let bs = ByteStream::from(bytes::Bytes::from(bytes));
        req = req.body(bs);
    }

    // Content-Type
    if let Some(ct) = body.content_type {
        req = req.content_type(ct);
    }

    // ACL
    if let Some(acl_str) = body.acl {
        if let Some(acl) = parse_acl(&acl_str) {
            req = req.acl(acl);
        } else {
            return Err(format!("invalid ACL: {}", acl_str));
        }
    }

    match req.send().await {
        Ok(_) => Ok(json!({ "bucket": body.bucket, "key": body.key })),
        Err(e) => Err(format!("S3 put_object error: {}", e)),
    }
}

async fn handle_s3_get_object(
    client: &S3Client,
    body: crate::input::S3GetObjectBody,
) -> Result<Value, String> {
    let resp = client
        .get_object()
        .bucket(&body.bucket)
        .key(&body.key)
        .send()
        .await
        .map_err(|e| format!("S3 get_object error: {}", e))?;

    let data = resp
        .body
        .collect()
        .await
        .map_err(|e| format!("error collecting S3 body: {}", e))?;
    let bytes = data.into_bytes();

    if body.as_base64.unwrap_or(false) {
        let b64 = BASE64.encode(&bytes);
        Ok(json!({
            "bucket": body.bucket,
            "key": body.key,
            "encoding": "base64",
            "content": b64
        }))
    } else {
        match String::from_utf8(bytes.to_vec()) {
            Ok(text) => Ok(json!({
                "bucket": body.bucket,
                "key": body.key,
                "encoding": "utf-8",
                "content": text
            })),
            Err(_) => {
                // Fallback to base64 if not UTF-8
                let b64 = BASE64.encode(bytes);
                Ok(json!({
                    "bucket": body.bucket,
                    "key": body.key,
                    "encoding": "base64",
                    "content": b64
                }))
            }
        }
    }
}

async fn handle_s3_delete_object(
    client: &S3Client,
    body: crate::input::S3DeleteObjectBody,
) -> Result<Value, String> {
    client
        .delete_object()
        .bucket(&body.bucket)
        .key(&body.key)
        .send()
        .await
        .map_err(|e| format!("S3 delete_object error: {}", e))?;

    Ok(json!({ "bucket": body.bucket, "key": body.key, "deleted": true }))
}

async fn handle_s3_list_objects(
    client: &S3Client,
    body: crate::input::S3ListObjectsBody,
) -> Result<Value, String> {
    let mut req = client.list_objects_v2().bucket(&body.bucket);

    if let Some(prefix) = body.prefix {
        req = req.prefix(prefix);
    }
    if let Some(mk) = body.max_keys {
        req = req.max_keys(mk);
    }
    if let Some(token) = body.continuation_token {
        req = req.continuation_token(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("S3 list_objects error: {}", e))?;

    let mut items = Vec::new();
    for obj in resp.contents() {
        items.push(json!({
            "key": obj.key().unwrap_or_default(),
            "size": obj.size(),
            "last_modified": obj.last_modified().map(|d| d.to_string())
        }));
    }

    Ok(json!({
        "bucket": body.bucket,
        "items": items,
        "is_truncated": resp.is_truncated(),
        "next_continuation_token": resp.next_continuation_token()
    }))
}

fn parse_acl(s: &str) -> Option<ObjectCannedAcl> {
    match s {
        "private" => Some(ObjectCannedAcl::Private),
        "public-read" => Some(ObjectCannedAcl::PublicRead),
        "public-read-write" => Some(ObjectCannedAcl::PublicReadWrite),
        "authenticated-read" => Some(ObjectCannedAcl::AuthenticatedRead),
        "aws-exec-read" => Some(ObjectCannedAcl::AwsExecRead),
        "bucket-owner-read" => Some(ObjectCannedAcl::BucketOwnerRead),
        "bucket-owner-full-control" => Some(ObjectCannedAcl::BucketOwnerFullControl),
        _ => None,
    }
}

// ----------------------
// SQS Helpers and Handlers
// ----------------------

async fn resolve_queue_url(
    client: &SqsClient,
    queue_url: Option<String>,
    queue_name: Option<String>,
) -> Result<String, String> {
    if let Some(url) = queue_url {
        return Ok(url);
    }
    let name = queue_name
        .ok_or_else(|| "missing 'queue_url' or 'queue_name' to resolve SQS queue".to_string())?;
    let try_default = client.get_queue_url().queue_name(name.clone()).send().await;
    let out = match try_default {
        Ok(ok) => ok,
        Err(_) => {
            // Fallback for LocalStack: specify default account id
            client
                .get_queue_url()
                .queue_name(name)
                .queue_owner_aws_account_id("000000000000")
                .send()
                .await
                .map_err(|e| format!("SQS get_queue_url error: {}", e))?
        }
    };
    out.queue_url()
        .map(|s| s.to_string())
        .ok_or_else(|| "SQS get_queue_url returned no url".to_string())
}

async fn handle_sqs_send_message(
    client: &SqsClient,
    body: crate::input::SqsSendMessageBody,
) -> Result<Value, String> {
    let queue_url = resolve_queue_url(client, body.queue_url, body.queue_name).await?;
    let mut req = client
        .send_message()
        .queue_url(&queue_url)
        .message_body(body.message_body);
    if let Some(d) = body.delay_seconds {
        req = req.delay_seconds(d);
    }
    if let Some(g) = body.message_group_id {
        req = req.message_group_id(g);
    }
    if let Some(dedup) = body.message_deduplication_id {
        req = req.message_deduplication_id(dedup);
    }
    if let Some(_attrs) = body.message_attributes {
        // TODO: map message_attributes (object) to SQS MessageAttributeValue
        // For now, skipping attributes to keep implementation minimal and robust.
    }

    let out = req
        .send()
        .await
        .map_err(|e| format!("SQS send_message error: {}", e))?;
    Ok(json!({
        "queue_url": queue_url,
        "message_id": out.message_id(),
        "sequence_number": out.sequence_number()
    }))
}

async fn handle_sqs_receive_messages(
    client: &SqsClient,
    body: crate::input::SqsReceiveMessagesBody,
) -> Result<Value, String> {
    let queue_url = resolve_queue_url(client, body.queue_url, body.queue_name).await?;
    let mut req = client.receive_message().queue_url(&queue_url);
    if let Some(n) = body.max_number_of_messages {
        req = req.max_number_of_messages(n);
    }
    if let Some(w) = body.wait_time_seconds {
        req = req.wait_time_seconds(w);
    }
    if let Some(v) = body.visibility_timeout {
        req = req.visibility_timeout(v);
    }

    let out = req
        .send()
        .await
        .map_err(|e| format!("SQS receive_messages error: {:?}", e))?;
    let mut messages = Vec::new();
    for m in out.messages() {
        messages.push(json!({
            "message_id": m.message_id(),
            "receipt_handle": m.receipt_handle(),
            "md5_of_body": m.md5_of_body(),
            "body": m.body()
        }));
    }
    Ok(json!({
        "queue_url": queue_url,
        "messages": messages
    }))
}

async fn handle_sqs_delete_message(
    client: &SqsClient,
    body: crate::input::SqsDeleteMessageBody,
) -> Result<Value, String> {
    let queue_url = resolve_queue_url(client, body.queue_url, body.queue_name).await?;
    client
        .delete_message()
        .queue_url(&queue_url)
        .receipt_handle(body.receipt_handle)
        .send()
        .await
        .map_err(|e| format!("SQS delete_message error: {}", e))?;
    Ok(json!({ "queue_url": queue_url, "deleted": true }))
}

use aws_sdk_sqs::types::QueueAttributeName;

fn parse_queue_attribute_name(name: &str) -> Option<QueueAttributeName> {
    match name {
        // Common settable attributes
        "DelaySeconds" => Some(QueueAttributeName::DelaySeconds),
        "MaximumMessageSize" => Some(QueueAttributeName::MaximumMessageSize),
        "MessageRetentionPeriod" => Some(QueueAttributeName::MessageRetentionPeriod),
        "Policy" => Some(QueueAttributeName::Policy),
        "ReceiveMessageWaitTimeSeconds" => Some(QueueAttributeName::ReceiveMessageWaitTimeSeconds),
        "VisibilityTimeout" => Some(QueueAttributeName::VisibilityTimeout),
        "RedrivePolicy" => Some(QueueAttributeName::RedrivePolicy),
        "RedriveAllowPolicy" => Some(QueueAttributeName::RedriveAllowPolicy),
        "FifoQueue" => Some(QueueAttributeName::FifoQueue),
        "ContentBasedDeduplication" => Some(QueueAttributeName::ContentBasedDeduplication),
        "KmsMasterKeyId" => Some(QueueAttributeName::KmsMasterKeyId),
        "KmsDataKeyReusePeriodSeconds" => Some(QueueAttributeName::KmsDataKeyReusePeriodSeconds),
        "SqsManagedSseEnabled" => Some(QueueAttributeName::SqsManagedSseEnabled),
        "DeduplicationScope" => Some(QueueAttributeName::DeduplicationScope),
        "FifoThroughputLimit" => Some(QueueAttributeName::FifoThroughputLimit),
        _ => None,
    }
}

async fn handle_sqs_create_queue(
    client: &SqsClient,
    body: crate::input::SqsCreateQueueBody,
) -> Result<Value, String> {
    let mut req = client.create_queue().queue_name(&body.queue_name);
    if let Some(attrs) = body.attributes {
        if let Some(map) = attrs.as_object() {
            let mut built: std::collections::HashMap<QueueAttributeName, String> =
                std::collections::HashMap::new();
            for (k, v) in map.iter() {
                if let Some(attr) = parse_queue_attribute_name(&k.to_string()) {
                    built.insert(attr, v.to_string());
                }
            }
            req = req.set_attributes(Some(built));
        }
    }
    let out = req
        .send()
        .await
        .map_err(|e| format!("SQS create_queue error: {}", e))?;
    Ok(json!({
        "queue_url": out.queue_url()
    }))
}

async fn handle_sqs_delete_queue(
    client: &SqsClient,
    body: crate::input::SqsDeleteQueueBody,
) -> Result<Value, String> {
    let queue_url = resolve_queue_url(client, body.queue_url, body.queue_name).await?;
    client
        .delete_queue()
        .queue_url(&queue_url)
        .send()
        .await
        .map_err(|e| format!("SQS delete_queue error: {:?}", e))?;
    Ok(json!({ "queue_url": queue_url, "deleted": true }))
}

async fn handle_sqs_get_queue_attributes(
    client: &SqsClient,
    body: crate::input::SqsGetQueueAttributesBody,
) -> Result<Value, String> {
    let queue_url = resolve_queue_url(client, body.queue_url, body.queue_name).await?;
    let out = client
        .get_queue_attributes()
        .queue_url(&queue_url)
        .attribute_names(QueueAttributeName::All)
        .send()
        .await
        .map_err(|e| format!("SQS get_queue_attributes error: {:?}", e))?;
    let mut attrs_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if let Some(map) = out.attributes() {
        for (k, v) in map.iter() {
            attrs_map.insert(k.as_str().to_string(), v.clone());
        }
    }
    Ok(json!({ "queue_url": queue_url, "attributes": attrs_map }))
}

async fn handle_sqs_set_queue_attributes(
    client: &SqsClient,
    body: crate::input::SqsSetQueueAttributesBody,
) -> Result<Value, String> {
    let queue_url = resolve_queue_url(client, body.queue_url, body.queue_name).await?;
    let mut attrs: std::collections::HashMap<QueueAttributeName, String> =
        std::collections::HashMap::new();
    if let Some(map) = body.attributes.as_object() {
        for (k, v) in map.iter() {
            if let Some(attr) = parse_queue_attribute_name(&k.to_string()) {
                attrs.insert(attr, v.to_string());
            }
        }
    } else {
        return Err("'attributes' must be an object of key->value".to_string());
    }
    client
        .set_queue_attributes()
        .queue_url(&queue_url)
        .set_attributes(Some(attrs))
        .send()
        .await
        .map_err(|e| format!("SQS set_queue_attributes error: {}", e))?;
    Ok(json!({ "queue_url": queue_url, "updated": true }))
}

// ----------------------
// S3 Bucket Handlers
// ----------------------

async fn handle_s3_create_bucket(
    client: &S3Client,
    body: crate::input::S3CreateBucketBody,
) -> Result<Value, String> {
    // Keep it simple and compatible with LocalStack/us-east-1: omit location constraint here.
    client
        .create_bucket()
        .bucket(&body.bucket)
        .send()
        .await
        .map_err(|e| format!("S3 create_bucket error: {}", e))?;
    Ok(json!({ "bucket": body.bucket, "created": true }))
}

async fn handle_s3_delete_bucket(
    client: &S3Client,
    body: crate::input::S3DeleteBucketBody,
) -> Result<Value, String> {
    client
        .delete_bucket()
        .bucket(&body.bucket)
        .send()
        .await
        .map_err(|e| format!("S3 delete_bucket error: {}", e))?;
    Ok(json!({ "bucket": body.bucket, "deleted": true }))
}

async fn handle_s3_get_bucket_location(
    client: &S3Client,
    body: crate::input::S3GetBucketLocationBody,
) -> Result<Value, String> {
    let out = client
        .get_bucket_location()
        .bucket(&body.bucket)
        .send()
        .await
        .map_err(|e| format!("S3 get_bucket_location error: {}", e))?;
    Ok(json!({ "bucket": body.bucket, "location": out.location_constraint().map(|v| v.as_str()) }))
}

async fn handle_s3_put_bucket_versioning(
    client: &S3Client,
    body: crate::input::S3PutBucketVersioningBody,
) -> Result<Value, String> {
    use aws_sdk_s3::types::{BucketVersioningStatus, VersioningConfiguration};
    let status = if body.enabled {
        BucketVersioningStatus::Enabled
    } else {
        BucketVersioningStatus::Suspended
    };
    let cfg = VersioningConfiguration::builder().status(status).build();
    client
        .put_bucket_versioning()
        .bucket(&body.bucket)
        .versioning_configuration(cfg)
        .send()
        .await
        .map_err(|e| format!("S3 put_bucket_versioning error: {}", e))?;
    Ok(json!({ "bucket": body.bucket, "versioning_enabled": body.enabled }))
}

async fn handle_s3_list_buckets(client: &S3Client) -> Result<Value, String> {
    let out = client
        .list_buckets()
        .send()
        .await
        .map_err(|e| format!("S3 list_buckets error: {}", e))?;
    let items: Vec<_> = out
        .buckets()
        .iter()
        .map(|b| json!({ "name": b.name(), "creation_date": b.creation_date().map(|d| d.to_string()) }))
        .collect();
    Ok(json!({ "buckets": items }))
}
