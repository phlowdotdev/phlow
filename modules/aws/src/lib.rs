mod handlers;
mod input;
mod setup;

use crate::input::{AwsApi, AwsInput};
use crate::setup::Setup;
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
            AwsApi::S3PutObject(body) => {
                match handlers::handle_s3_put_object(&s3_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3GetObject(body) => {
                match handlers::handle_s3_get_object(&s3_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3DeleteObject(body) => {
                match handlers::handle_s3_delete_object(&s3_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3ListObjects(body) => {
                match handlers::handle_s3_list_objects(&s3_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3CreateBucket(body) => {
                match handlers::handle_s3_create_bucket(&s3_client, &setup_cfg, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3DeleteBucket(body) => {
                match handlers::handle_s3_delete_bucket(&s3_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3GetBucketLocation(body) => {
                match handlers::handle_s3_get_bucket_location(&s3_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3PutBucketVersioning(body) => {
                match handlers::handle_s3_put_bucket_versioning(&s3_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::S3ListBuckets => match handlers::handle_s3_list_buckets(&s3_client).await {
                Ok(data) => success_response!(data),
                Err(e) => error_response!(e),
            },
            AwsApi::SqsSendMessage(body) => {
                match handlers::handle_sqs_send_message(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsReceiveMessages(body) => {
                match handlers::handle_sqs_receive_messages(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsDeleteMessage(body) => {
                match handlers::handle_sqs_delete_message(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsCreateQueue(body) => {
                match handlers::handle_sqs_create_queue(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsDeleteQueue(body) => {
                match handlers::handle_sqs_delete_queue(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsGetQueueAttributes(body) => {
                match handlers::handle_sqs_get_queue_attributes(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
            AwsApi::SqsSetQueueAttributes(body) => {
                match handlers::handle_sqs_set_queue_attributes(&sqs_client, body).await {
                    Ok(data) => success_response!(data),
                    Err(e) => error_response!(e),
                }
            }
        };

        sender_safe!(package.sender, response.into());
    }

    Ok(())
}

// handlers moved to handlers::s3_handler and handlers::sqs_handler
