use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::types::{BucketLocationConstraint, CreateBucketConfiguration, ObjectCannedAcl};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use phlow_sdk::prelude::*;
use std::collections::HashMap;

use crate::setup::Setup;

pub async fn handle_s3_put_object(
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
        Err(e) => Err(e.to_string()),
    }
}

pub async fn handle_s3_get_object(
    client: &S3Client,
    body: crate::input::S3GetObjectBody,
) -> Result<Value, String> {
    let resp = match client
        .get_object()
        .bucket(&body.bucket)
        .key(&body.key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };

    let data = match resp.body.collect().await {
        Ok(d) => d,
        Err(e) => return Err(e.to_string()),
    };
    let bytes = data.into_bytes();

    if body.as_base64.unwrap_or(false) {
        let b64 = BASE64.encode(&bytes);
        let first_line = b64.lines().next().unwrap_or("");

        log::debug!(
            "Note: Retrieved S3 object as base64. First line:\n{}",
            first_line
        );

        Ok(json!({
            "bucket": body.bucket,
            "key": body.key,
            "encoding": "base64",
            "content": b64
        }))
    } else {
        match String::from_utf8(bytes.to_vec()) {
            Ok(text) => {
                let fist_line = text.lines().next().unwrap_or("");

                log::debug!(
                    "Note: Retrieved S3 object as UTF-8 text. First line:\n{}",
                    fist_line
                );

                Ok(json!({
                    "bucket": body.bucket,
                    "key": body.key,
                    "encoding": "utf-8",
                    "content": text
                }))
            }
            Err(_) => {
                // Fallback to base64 if not UTF-8
                let b64 = BASE64.encode(bytes);

                log::debug!(
                    "Note: Retrieved S3 object is not valid UTF-8; returning as base64. First line:\n{}",
                    b64.lines().next().unwrap_or("")
                );

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

pub async fn handle_s3_delete_object(
    client: &S3Client,
    body: crate::input::S3DeleteObjectBody,
) -> Result<Value, String> {
    match client
        .delete_object()
        .bucket(&body.bucket)
        .key(&body.key)
        .send()
        .await
    {
        Ok(_) => Ok(json!({ "bucket": body.bucket, "key": body.key, "deleted": true })),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn handle_s3_list_objects(
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

    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };

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

fn parse_location_constraint(s: &str) -> Option<BucketLocationConstraint> {
    match s {
        // Special case handled by caller: us-east-1 => None (no LocationConstraint)
        "us-east-2" => Some(BucketLocationConstraint::UsEast2),
        "us-west-1" => Some(BucketLocationConstraint::UsWest1),
        "us-west-2" => Some(BucketLocationConstraint::UsWest2),
        "af-south-1" => Some(BucketLocationConstraint::AfSouth1),
        "ap-east-1" => Some(BucketLocationConstraint::ApEast1),
        "ap-south-1" => Some(BucketLocationConstraint::ApSouth1),
        "ap-south-2" => Some(BucketLocationConstraint::ApSouth2),
        "ap-northeast-1" => Some(BucketLocationConstraint::ApNortheast1),
        "ap-northeast-2" => Some(BucketLocationConstraint::ApNortheast2),
        "ap-northeast-3" => Some(BucketLocationConstraint::ApNortheast3),
        "ap-southeast-1" => Some(BucketLocationConstraint::ApSoutheast1),
        "ap-southeast-2" => Some(BucketLocationConstraint::ApSoutheast2),
        "ap-southeast-3" => Some(BucketLocationConstraint::ApSoutheast3),
        "ap-southeast-4" => Some(BucketLocationConstraint::ApSoutheast4),
        "ca-central-1" => Some(BucketLocationConstraint::CaCentral1),
        "eu-central-1" => Some(BucketLocationConstraint::EuCentral1),
        "eu-central-2" => Some(BucketLocationConstraint::EuCentral2),
        "eu-north-1" => Some(BucketLocationConstraint::EuNorth1),
        "eu-south-1" => Some(BucketLocationConstraint::EuSouth1),
        "eu-south-2" => Some(BucketLocationConstraint::EuSouth2),
        "eu-west-1" => Some(BucketLocationConstraint::EuWest1),
        "eu-west-2" => Some(BucketLocationConstraint::EuWest2),
        "eu-west-3" => Some(BucketLocationConstraint::EuWest3),
        "me-central-1" => Some(BucketLocationConstraint::MeCentral1),
        "me-south-1" => Some(BucketLocationConstraint::MeSouth1),
        "sa-east-1" => Some(BucketLocationConstraint::SaEast1),
        // China/GovCloud partitions (supported in enum)
        "cn-north-1" => Some(BucketLocationConstraint::CnNorth1),
        "cn-northwest-1" => Some(BucketLocationConstraint::CnNorthwest1),
        "us-gov-east-1" => Some(BucketLocationConstraint::UsGovEast1),
        "us-gov-west-1" => Some(BucketLocationConstraint::UsGovWest1),
        _ => None,
    }
}

pub async fn handle_s3_create_bucket(
    client: &S3Client,
    setup: &Setup,
    body: crate::input::S3CreateBucketBody,
) -> Result<Value, String> {
    // In us-east-1 the API requires no LocationConstraint; otherwise set it.
    let mut req = client.create_bucket().bucket(&body.bucket);

    // Effective location: explicit input, else setup.with.region (required by module policy)
    let effective_loc = body.location.clone().or_else(|| setup.region.clone());
    if let Some(loc) = effective_loc {
        if loc != "us-east-1" {
            if let Some(lc) = parse_location_constraint(&loc) {
                let cfg = CreateBucketConfiguration::builder()
                    .location_constraint(lc)
                    .build();
                req = req.create_bucket_configuration(cfg);
            } else {
                return Err(format!("invalid 'location' for create_bucket: {}", loc));
            }
        }
    } else {
        return Err("missing region: set with.region or provide 'location'".to_string());
    }

    match req.send().await {
        Ok(_) => Ok(json!({ "bucket": body.bucket, "created": true })),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn handle_s3_delete_bucket(
    client: &S3Client,
    body: crate::input::S3DeleteBucketBody,
) -> Result<Value, String> {
    if body.force.unwrap_or(false) {
        // Empty bucket (versions and objects) before deletion
        empty_bucket_versions(client, &body.bucket).await?;
        empty_bucket_objects(client, &body.bucket).await?;
    } else {
        let resp = match client
            .list_objects_v2()
            .bucket(&body.bucket)
            .max_keys(1)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };

        let contents = resp.contents();
        if !contents.is_empty() {
            return Err(format!(
                "bucket '{}' is not empty; use force=true to delete",
                body.bucket
            ));
        }
    }
    match client.delete_bucket().bucket(&body.bucket).send().await {
        Ok(_) => Ok(json!({ "bucket": body.bucket, "deleted": true })),
        Err(e) => Err(e.to_string()),
    }
}

async fn empty_bucket_objects(client: &S3Client, bucket: &str) -> Result<(), String> {
    use aws_sdk_s3::types::Delete;
    loop {
        let resp = match client
            .list_objects_v2()
            .bucket(bucket)
            .max_keys(1000)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };
        let contents = resp.contents();
        if contents.is_empty() {
            break;
        }

        let mut ids: Vec<aws_sdk_s3::types::ObjectIdentifier> = Vec::new();
        for o in contents.iter() {
            if let Some(k) = o.key() {
                let obj_id = aws_sdk_s3::types::ObjectIdentifier::builder()
                    .key(k)
                    .build()
                    .map_err(|e| format!("build ObjectIdentifier error: {}", e))?;
                ids.push(obj_id);
            }
        }

        let del = match Delete::builder().set_objects(Some(ids)).build() {
            Ok(d) => d,
            Err(e) => return Err(e.to_string()),
        };
        match client
            .delete_objects()
            .bucket(bucket)
            .delete(del)
            .send()
            .await
        {
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

async fn empty_bucket_versions(client: &S3Client, bucket: &str) -> Result<(), String> {
    use aws_sdk_s3::types::Delete;
    loop {
        let resp = match client
            .list_object_versions()
            .bucket(bucket)
            .max_keys(1000)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };

        let mut ids: Vec<aws_sdk_s3::types::ObjectIdentifier> = Vec::new();

        for v in resp.versions().iter() {
            if let Some(key) = v.key() {
                let mut b = aws_sdk_s3::types::ObjectIdentifier::builder().key(key);
                if let Some(vid) = v.version_id() {
                    b = b.version_id(vid);
                }
                let obj_id = b
                    .build()
                    .map_err(|e| format!("build ObjectIdentifier error: {}", e))?;
                ids.push(obj_id);
            }
        }
        for m in resp.delete_markers().iter() {
            if let Some(key) = m.key() {
                let mut b = aws_sdk_s3::types::ObjectIdentifier::builder().key(key);
                if let Some(vid) = m.version_id() {
                    b = b.version_id(vid);
                }
                let obj_id = b
                    .build()
                    .map_err(|e| format!("build ObjectIdentifier error: {}", e))?;
                ids.push(obj_id);
            }
        }

        if ids.is_empty() {
            break;
        }

        let del = match Delete::builder().set_objects(Some(ids)).build() {
            Ok(d) => d,
            Err(e) => return Err(e.to_string()),
        };
        match client
            .delete_objects()
            .bucket(bucket)
            .delete(del)
            .send()
            .await
        {
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

pub async fn handle_s3_get_bucket_location(
    client: &S3Client,
    body: crate::input::S3GetBucketLocationBody,
) -> Result<Value, String> {
    let out = match client
        .get_bucket_location()
        .bucket(&body.bucket)
        .send()
        .await
    {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    Ok(json!({ "bucket": body.bucket, "location": out.location_constraint().map(|v| v.as_str()) }))
}

pub async fn handle_s3_put_bucket_versioning(
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
    match client
        .put_bucket_versioning()
        .bucket(&body.bucket)
        .versioning_configuration(cfg)
        .send()
        .await
    {
        Ok(_) => Ok(json!({ "bucket": body.bucket, "versioning_enabled": body.enabled })),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn handle_s3_list_buckets(client: &S3Client) -> Result<Value, String> {
    let out = match client.list_buckets().send().await {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    let items: Vec<_> = out
        .buckets()
        .iter()
        .map(|b| json!({ "name": b.name(), "creation_date": b.creation_date().map(|d| d.to_string()) }))
        .collect();
    Ok(json!({ "buckets": items }))
}

pub async fn handle_s3_get_object_attributes(
    client: &S3Client,
    body: crate::input::S3GetObjectAttributesBody,
) -> Result<Value, String> {
    // Use HEAD object to retrieve basic attributes (size, etag, last_modified, content_type, metadata)
    let mut req = client.head_object().bucket(&body.bucket).key(&body.key);

    if let Some(vid) = body.version_id {
        req = req.version_id(vid);
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };

    let metadata = resp
        .metadata()
        .map(|m| {
            let mut map: HashMap<String, Value> = HashMap::new();
            for (k, v) in m.iter() {
                map.insert(k.clone(), Value::from(v.clone()));
            }
            map.to_value()
        })
        .unwrap_or(json!(null));

    let result = json!({
        "bucket": body.bucket,
        "key": body.key,
        "content_length": resp.content_length(),
        "etag": resp.e_tag().map(|s| s.to_string()),
        "last_modified": resp.last_modified().map(|d| d.to_string()),
        "content_type": resp.content_type().map(|s| s.to_string()),
        "metadata": metadata
    });

    Ok(result)
}
