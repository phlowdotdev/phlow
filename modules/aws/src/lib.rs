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
    log::debug!("aws module: start");
    let rx = module_channel!(setup);
    log::debug!("aws module: channel created");

    let setup_cfg = Setup::try_from(setup.with)?;
    log::debug!("aws module: setup parsed");

    // Build S3 client once; reused for all S3 actions
    let s3_client = setup_cfg.build_s3_client().await?;
    log::debug!("aws module: s3 client built");
    // Build SQS client once; reused for all SQS actions
    let sqs_client = setup_cfg.build_sqs_client().await?;
    log::debug!("aws module: sqs client built");

    for package in rx {
        log::debug!("aws module: received package");
        let input_value = package.input().unwrap_or(Value::Null);
        log::debug!("aws module: raw input = {}", input_value);

        let parsed = match AwsInput::try_from(input_value) {
            Ok(p) => {
                log::debug!("aws module: input parsed: action={:?}", p.action);
                p
            }
            Err(e) => {
                log::debug!("aws module: input parse error: {}", e);
                sender_safe!(
                    package.sender,
                    error_response!(format!("Invalid input: {}", e)).into()
                );
                continue;
            }
        };

        let response: Value = match parsed.with {
            AwsApi::S3PutObject(body) => {
                log::debug!("aws module: handling S3PutObject");
                match handlers::handle_s3_put_object(&s3_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: S3PutObject success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3PutObject error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::S3GetObject(body) => {
                log::debug!("aws module: handling S3GetObject");
                match handlers::handle_s3_get_object(&s3_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: S3GetObject success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3GetObject error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::S3DeleteObject(body) => {
                log::debug!("aws module: handling S3DeleteObject");
                match handlers::handle_s3_delete_object(&s3_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: S3DeleteObject success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3DeleteObject error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::S3ListObjects(body) => {
                log::debug!("aws module: handling S3ListObjects");
                match handlers::handle_s3_list_objects(&s3_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: S3ListObjects success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3ListObjects error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::S3CreateBucket(body) => {
                log::debug!("aws module: handling S3CreateBucket");
                match handlers::handle_s3_create_bucket(&s3_client, &setup_cfg, body).await {
                    Ok(data) => {
                        log::debug!("aws module: S3CreateBucket success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3CreateBucket error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::S3DeleteBucket(body) => {
                log::debug!("aws module: handling S3DeleteBucket");
                match handlers::handle_s3_delete_bucket(&s3_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: S3DeleteBucket success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3DeleteBucket error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::S3GetBucketLocation(body) => {
                log::debug!("aws module: handling S3GetBucketLocation");
                match handlers::handle_s3_get_bucket_location(&s3_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: S3GetBucketLocation success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3GetBucketLocation error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::S3PutBucketVersioning(body) => {
                log::debug!("aws module: handling S3PutBucketVersioning");
                match handlers::handle_s3_put_bucket_versioning(&s3_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: S3PutBucketVersioning success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3PutBucketVersioning error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::S3ListBuckets => {
                log::debug!("aws module: handling S3ListBuckets");
                match handlers::handle_s3_list_buckets(&s3_client).await {
                    Ok(data) => {
                        log::debug!("aws module: S3ListBuckets success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3ListBuckets error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::S3GetObjectAttributes(body) => {
                log::debug!("aws module: handling S3GetObjectAttributes");
                match handlers::handle_s3_get_object_attributes(&s3_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: S3GetObjectAttributes success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: S3GetObjectAttributes error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::SqsSendMessage(body) => {
                log::debug!("aws module: handling SqsSendMessage");
                match handlers::handle_sqs_send_message(&sqs_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: SqsSendMessage success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: SqsSendMessage error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::SqsReceiveMessages(body) => {
                log::debug!("aws module: handling SqsReceiveMessages");
                match handlers::handle_sqs_receive_messages(&sqs_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: SqsReceiveMessages success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: SqsReceiveMessages error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::SqsDeleteMessage(body) => {
                log::debug!("aws module: handling SqsDeleteMessage");
                match handlers::handle_sqs_delete_message(&sqs_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: SqsDeleteMessage success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: SqsDeleteMessage error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::SqsCreateQueue(body) => {
                log::debug!("aws module: handling SqsCreateQueue");
                match handlers::handle_sqs_create_queue(&sqs_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: SqsCreateQueue success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: SqsCreateQueue error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::SqsDeleteQueue(body) => {
                log::debug!("aws module: handling SqsDeleteQueue");
                match handlers::handle_sqs_delete_queue(&sqs_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: SqsDeleteQueue success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: SqsDeleteQueue error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::SqsGetQueueAttributes(body) => {
                log::debug!("aws module: handling SqsGetQueueAttributes");
                match handlers::handle_sqs_get_queue_attributes(&sqs_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: SqsGetQueueAttributes success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: SqsGetQueueAttributes error: {}", e);
                        error_response!(e)
                    }
                }
            }
            AwsApi::SqsSetQueueAttributes(body) => {
                log::debug!("aws module: handling SqsSetQueueAttributes");
                match handlers::handle_sqs_set_queue_attributes(&sqs_client, body).await {
                    Ok(data) => {
                        log::debug!("aws module: SqsSetQueueAttributes success");
                        success_response!(data)
                    }
                    Err(e) => {
                        log::debug!("aws module: SqsSetQueueAttributes error: {}", e);
                        error_response!(e)
                    }
                }
            }
        };

        sender_safe!(package.sender, response.into());
        log::debug!("aws module: response sent");
    }

    log::debug!("aws module: end");
    Ok(())
}

// handlers moved to handlers::s3_handler and handlers::sqs_handler
