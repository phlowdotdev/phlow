mod input;
mod setup;

use crate::input::{AwsApi, AwsInput};
use crate::setup::Setup;
use aws_sdk_s3::types::ObjectCannedAcl;
use aws_sdk_s3::Client as S3Client;
use phlow_sdk::prelude::*;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

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

    for package in rx {
        let input_value = package.input().unwrap_or(Value::Null);

        let parsed = match AwsInput::try_from(input_value) {
            Ok(p) => p,
            Err(e) => {
                sender_safe!(package.sender, error_response!(format!("Invalid input: {}", e)).into());
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
            _ => error_response!("Action not implemented yet for AWS module"),
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

    let mut req = client
        .put_object()
        .bucket(&body.bucket)
        .key(&body.key);

    // Content
    let bytes: Vec<u8> = if let Some(b64) = body.content_base64 {
        BASE64.decode(b64).map_err(|e| format!("invalid base64 content: {}", e))?
    } else if let Some(text) = body.content {
        text.into_bytes()
    } else {
        return Err("Missing 'content' or 'content_base64' for s3_put_object".to_string());
    };

    let bs = ByteStream::from(bytes::Bytes::from(bytes));
    req = req.body(bs);

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
    let mut req = client
        .list_objects_v2()
        .bucket(&body.bucket);

    if let Some(prefix) = body.prefix { req = req.prefix(prefix); }
    if let Some(mk) = body.max_keys { req = req.max_keys(mk); }
    if let Some(token) = body.continuation_token { req = req.continuation_token(token); }

    let resp = req.send().await.map_err(|e| format!("S3 list_objects error: {}", e))?;

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

