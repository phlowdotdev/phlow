use aws_sdk_sqs::Client as SqsClient;
use aws_sdk_sqs::types::QueueAttributeName;
use phlow_sdk::prelude::*;
use phlow_sdk::tracing::debug;

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

pub async fn handle_sqs_send_message(
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

pub async fn handle_sqs_receive_messages(
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

pub async fn handle_sqs_delete_message(
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

pub async fn handle_sqs_create_queue(
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

pub async fn handle_sqs_delete_queue(
    client: &SqsClient,
    body: crate::input::SqsDeleteQueueBody,
) -> Result<Value, String> {
    let queue_url = resolve_queue_url(client, body.queue_url, body.queue_name).await?;

    if body.force.unwrap_or(false) {
        debug!("Force deleting SQS queue: {}", &queue_url);
        // Best-effort purge before deletion; ignore errors like PurgeQueueInProgress
        let _ = client.purge_queue().queue_url(&queue_url).send().await;
    } else {
        debug!(
            "Checking if SQS queue is empty before deletion: {}",
            &queue_url
        );
        // queue is not empty
        let resp = client
            .receive_message()
            .queue_url(&queue_url)
            .max_number_of_messages(1)
            .send()
            .await
            .map_err(|e| format!("SQS receive_message error: {:?}", e))?;
        let msgs = resp.messages();
        if !msgs.is_empty() {
            return Err("SQS queue is not empty; use 'force' to delete".to_string());
        }
    }

    client
        .delete_queue()
        .queue_url(&queue_url)
        .send()
        .await
        .map_err(|e| format!("SQS delete_queue error: {:?}", e))?;
    Ok(json!({ "queue_url": queue_url, "deleted": true }))
}

pub async fn handle_sqs_get_queue_attributes(
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

pub async fn handle_sqs_set_queue_attributes(
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
