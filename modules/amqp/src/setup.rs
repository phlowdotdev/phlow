use phlow_sdk::prelude::*;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    GenericError(String),
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Error::GenericError(error.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenericError(msg) => write!(f, "Generic error: {}", msg),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub routing_key: String,
    pub username: String,
    pub password: String,
    pub port: u16,
    pub exchange: String,
    pub exchange_type: String,
    pub consumer_tag: String,
    pub queue_name: String,
    pub uri: Option<String>,
    pub vhost: String,
    pub max_retry: i64,
    pub dlq_enable: bool,
    pub max_concurrency: u16,
}

impl Config {
    pub fn to_connection_string(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, &self.port, &self.vhost
        )
    }
}

impl TryFrom<&Value> for Config {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Error> {
        let exchange = value
            .get("exchange")
            .map(|v| v.to_string())
            .unwrap_or("".to_string());

        let exchange_type = value
            .get("exchange_type")
            .map(|v| v.to_string())
            .unwrap_or("direct".to_string());

        let routing_key = if exchange_type == "fanout" || exchange_type == "headers" {
            "".to_string()
        } else {
            value
                .get("routing_key")
                .map(|v| v.to_string())
                .unwrap_or("".to_string())
        };

        let consumer_tag = value
            .get("consumer_tag")
            .map(|v| v.to_string())
            .unwrap_or("".to_string());

        let queue_name: String = value
            .get("queue_name")
            .map(|v| v.to_string())
            .unwrap_or_else(|| {
                // Use routing_key as default only if queue_name is not provided
                routing_key.clone()
            });

        let uri: Option<String> = value
            .get("uri")
            .and_then(|v| v.as_string_b())
            .map(|s| s.as_string());

        let (username, password, host, port, vhost) = if let Some(uri_str) = &uri {
            log::debug!("Using URI from config: {}", uri_str);

            if uri_str.starts_with("amqp://") {
                match url::Url::parse(uri_str) {
                    Ok(parsed) => {
                        let host = parsed.host_str().unwrap_or("localhost").to_string();
                        let port = parsed.port().map(|p| p as u16);
                        let username = if !parsed.username().is_empty() {
                            parsed.username().to_string()
                        } else {
                            "guest".to_string()
                        };
                        let password = parsed.password().unwrap_or("guest").to_string();
                        let vhost = parsed.path().trim_start_matches('/').to_string();
                        (username, password, host, port, vhost)
                    }
                    Err(e) => {
                        log::warn!("Failed to parse AMQP URI: {} ({})", uri_str, e);
                        (
                            "guest".to_string(),
                            "guest".to_string(),
                            "localhost".to_string(),
                            None,
                            "/".to_string(),
                        )
                    }
                }
            } else {
                (
                    "guest".to_string(),
                    "guest".to_string(),
                    "localhost".to_string(),
                    None,
                    "/".to_string(),
                )
            }
        } else {
            let username = value
                .get("username")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "guest".to_string());

            let password = value
                .get("password")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "guest".to_string());

            let host = value
                .get("host")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "localhost".to_string());

            let port = value.get("port").map(|v| v.to_i64().unwrap_or(5672) as u16);

            let vhost = value
                .get("vhost")
                .and_then(|v| v.as_string_b())
                .map(|s| s.as_string())
                .unwrap_or_else(|| "/".to_string());

            (username, password, host, port, vhost)
        };

        // Parse new retry and DLQ configuration
        let max_retry = value
            .get("max_retry")
            .map(|v| v.to_i64().unwrap_or(3))
            .unwrap_or(3);

        let dlq_enable = value
            .get("dlq_enable")
            .and_then(|v| v.as_bool())
            .copied()
            .unwrap_or(true);

        // Limite de concorrência (0 = sem limites)
        let max_concurrency = value
            .get("max_concurrency")
            .map(|v| v.to_i64().unwrap_or(10))
            .unwrap_or(10)
            .clamp(0, u16::MAX as i64) as u16;

        // Parse RabbitMQ definition if available and import
        if let Some(definition) = value.get("definition") {
            log::debug!("Found definition in config, importing...");
            let management_port = value
                .get("management_port")
                .map(|v| v.to_i64().unwrap_or(15672) as u16)
                .unwrap_or(15672);

            // Execute import synchronously to ensure it completes before module use
            match import_definition(&host, management_port, &username, &password, definition) {
                Ok(_) => log::debug!("Definition import completed successfully"),
                Err(e) => {
                    log::debug!("Definition import failed: {}", e);
                    panic!("Error importing definition: {}", e);
                }
            }
        } else {
            log::debug!("No definition found in config");
        }

        Ok(Self {
            host,
            username,
            password,
            port: port.unwrap_or(5672),
            routing_key,
            exchange,
            exchange_type,
            consumer_tag,
            queue_name,
            vhost,
            uri,
            max_retry,
            dlq_enable,
            max_concurrency,
        })
    }
}

fn import_definition(
    host: &str,
    management_port: u16,
    username: &str,
    password: &str,
    definition: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Starting import_definition function");

    if let Some(obj) = definition.as_object() {
        log::debug!("Definition parsed as object successfully");
        let client = reqwest::blocking::Client::new();

        // Import vhosts
        if let Some(vhosts) = obj.get("vhosts") {
            if let Some(vhosts_array) = vhosts.as_array() {
                for vhost in vhosts_array.values.iter() {
                    if let Some(vhost_obj) = vhost.as_object() {
                        let vhost_name = vhost_obj
                            .get("name")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());

                        let url = format!(
                            "http://{}:{}/api/vhosts/{}",
                            host, management_port, vhost_name
                        );
                        let response = client
                            .put(&url)
                            .basic_auth(username, Some(password))
                            .header("Content-Type", "application/json")
                            .json(&serde_json::json!({}))
                            .send()?;

                        log::debug!("Created vhost '{}': {}", vhost_name, response.status());
                    }
                }
            }
        }

        // Import exchanges
        if let Some(exchanges) = obj.get("exchanges") {
            if let Some(exchanges_array) = exchanges.as_array() {
                for exchange in exchanges_array.values.iter() {
                    if let Some(exchange_obj) = exchange.as_object() {
                        let exchange_name = exchange_obj
                            .get("name")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());

                        let vhost = exchange_obj
                            .get("vhost")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("/".to_string());

                        let exchange_type = exchange_obj
                            .get("type")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("direct".to_string());

                        let durable = exchange_obj
                            .get("durable")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(true);

                        let auto_delete = exchange_obj
                            .get("auto_delete")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(false);

                        let internal = exchange_obj
                            .get("internal")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(false);

                        let url = format!(
                            "http://{}:{}/api/exchanges/{}/{}",
                            host,
                            management_port,
                            urlencoding::encode(&vhost),
                            urlencoding::encode(&exchange_name)
                        );

                        let arguments = exchange_obj
                            .get("arguments")
                            .and_then(|v| v.as_object())
                            .map(|_| serde_json::json!({}))
                            .unwrap_or(serde_json::json!({}));

                        let body = serde_json::json!({
                            "type": exchange_type,
                            "durable": durable,
                            "auto_delete": auto_delete,
                            "internal": internal,
                            "arguments": arguments
                        });

                        let response = client
                            .put(&url)
                            .basic_auth(username, Some(password))
                            .header("Content-Type", "application/json")
                            .json(&body)
                            .send()?;

                        log::debug!(
                            "Created exchange '{}': {}",
                            exchange_name,
                            response.status()
                        );
                    }
                }
            }
        }

        // Import queues
        if let Some(queues) = obj.get("queues") {
            if let Some(queues_array) = queues.as_array() {
                for queue in queues_array.values.iter() {
                    if let Some(queue_obj) = queue.as_object() {
                        let queue_name = queue_obj
                            .get("name")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());

                        let vhost = queue_obj
                            .get("vhost")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("/".to_string());

                        let durable = queue_obj
                            .get("durable")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(true);

                        let auto_delete = queue_obj
                            .get("auto_delete")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(false);

                        let url = format!(
                            "http://{}:{}/api/queues/{}/{}",
                            host,
                            management_port,
                            urlencoding::encode(&vhost),
                            urlencoding::encode(&queue_name)
                        );

                        let arguments = queue_obj
                            .get("arguments")
                            .and_then(|v| v.as_object())
                            .map(|obj| {
                                let mut args_map = serde_json::Map::new();
                                for (key, value) in obj.iter() {
                                    let key_str = key.to_string();
                                    match value {
                                        Value::String(s) => {
                                            args_map.insert(
                                                key_str,
                                                serde_json::Value::String(s.as_string()),
                                            );
                                        }
                                        Value::Number(n) => {
                                            if let Some(i) = n.get_i64() {
                                                args_map.insert(
                                                    key_str,
                                                    serde_json::Value::Number(
                                                        serde_json::Number::from(i),
                                                    ),
                                                );
                                            } else if let Some(f) = n.get_f64() {
                                                args_map.insert(
                                                    key_str,
                                                    serde_json::Value::Number(
                                                        serde_json::Number::from_f64(f)
                                                            .unwrap_or(serde_json::Number::from(0)),
                                                    ),
                                                );
                                            } else {
                                                // fallback para outros tipos numéricos
                                                args_map.insert(
                                                    key_str,
                                                    serde_json::Value::String(n.to_string()),
                                                );
                                            }
                                        }
                                        Value::Boolean(b) => {
                                            args_map.insert(key_str, serde_json::Value::Bool(*b));
                                        }
                                        _ => {
                                            args_map.insert(
                                                key_str,
                                                serde_json::Value::String(value.to_string()),
                                            );
                                        }
                                    }
                                }
                                serde_json::Value::Object(args_map)
                            })
                            .unwrap_or(serde_json::json!({}));

                        let body = serde_json::json!({
                            "durable": durable,
                            "auto_delete": auto_delete,
                            "arguments": arguments
                        });

                        let response = client
                            .put(&url)
                            .basic_auth(username, Some(password))
                            .header("Content-Type", "application/json")
                            .json(&body)
                            .send()?;

                        log::debug!("Created queue '{}': {}", queue_name, response.status());
                    }
                }
            }
        }

        // Import bindings
        if let Some(bindings) = obj.get("bindings") {
            if let Some(bindings_array) = bindings.as_array() {
                for binding in bindings_array.values.iter() {
                    if let Some(binding_obj) = binding.as_object() {
                        let source = binding_obj
                            .get("source")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());

                        let destination = binding_obj
                            .get("destination")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());

                        let vhost = binding_obj
                            .get("vhost")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("/".to_string());

                        let routing_key = binding_obj
                            .get("routing_key")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());

                        let url = format!(
                            "http://{}:{}/api/bindings/{}/e/{}/q/{}",
                            host,
                            management_port,
                            urlencoding::encode(&vhost),
                            urlencoding::encode(&source),
                            urlencoding::encode(&destination)
                        );

                        log::debug!("Creating binding with URL: {}", url);

                        let arguments = binding_obj
                            .get("arguments")
                            .and_then(|v| v.as_object())
                            .map(|_| serde_json::json!({}))
                            .unwrap_or(serde_json::json!({}));

                        let body = serde_json::json!({
                            "routing_key": routing_key,
                            "arguments": arguments
                        });

                        log::debug!("Binding request body: {}", body);

                        let response = client
                            .post(&url)
                            .basic_auth(username, Some(password))
                            .header("Content-Type", "application/json")
                            .json(&body)
                            .send();

                        match response {
                            Ok(resp) => {
                                let status = resp.status();
                                if !status.is_success() {
                                    let body_text = resp.text().unwrap_or_else(|_| {
                                        "Unable to read response body".to_string()
                                    });
                                    log::debug!(
                                        "Binding creation failed with status {}: {}",
                                        status,
                                        body_text
                                    );
                                } else {
                                    log::debug!(
                                        "Created binding '{}' -> '{}': {}",
                                        source,
                                        destination,
                                        status
                                    );
                                }
                            }
                            Err(e) => {
                                log::debug!("Error creating binding: {}", e);
                                return Err(e.into());
                            }
                        }
                    }
                }
            }
        }
    } else {
        log::debug!("Definition is not an object, skipping import");
    }

    Ok(())
}
