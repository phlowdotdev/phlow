use phlow_sdk::prelude::*;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    RoutingKey,
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
            Self::RoutingKey => write!(f, "routing_key is required"),
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
}

impl Config {
    pub fn to_connection_string(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}",
            self.username, self.password, self.host, &self.port,
        )
    }
}

impl TryFrom<&Value> for Config {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Error> {
        let username = value
            .get("username")
            .map(|v| v.to_string())
            .unwrap_or("guest".to_string());

        let password = value
            .get("password")
            .map(|v| v.to_string())
            .unwrap_or("guest".to_string());

        let port = value.get("port").map(|v| v.to_i64().unwrap_or(5672) as u16);

        let host = value
            .get("host")
            .map(|v| v.to_string())
            .unwrap_or("localhost".to_string());

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
                .ok_or(Error::RoutingKey)?
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

        // Parse RabbitMQ definition if available and import
        if let Some(definition) = value.get("definition") {
            let management_port = value.get("management_port")
                .map(|v| v.to_i64().unwrap_or(15672) as u16)
                .unwrap_or(15672);
            
            import_definition(&host, management_port, &username, &password, definition)?;
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
    if let Some(obj) = definition.as_object() {
        let client = reqwest::blocking::Client::new();
        
        // Import vhosts
        if let Some(vhosts) = obj.get("vhosts") {
            if let Some(vhosts_array) = vhosts.as_array() {
                for vhost in vhosts_array.values.iter() {
                    if let Some(vhost_obj) = vhost.as_object() {
                        let vhost_name = vhost_obj.get("name")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());
                        
                        let url = format!("http://{}:{}/api/vhosts/{}", host, management_port, vhost_name);
                        let response = client.put(&url)
                            .basic_auth(username, Some(password))
                            .header("Content-Type", "application/json")
                            .json(&serde_json::json!({}))
                            .send()?;
                        
                        debug!("Created vhost '{}': {}", vhost_name, response.status());
                    }
                }
            }
        }

        // Import exchanges
        if let Some(exchanges) = obj.get("exchanges") {
            if let Some(exchanges_array) = exchanges.as_array() {
                for exchange in exchanges_array.values.iter() {
                    if let Some(exchange_obj) = exchange.as_object() {
                        let exchange_name = exchange_obj.get("name")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());
                        
                        let vhost = exchange_obj.get("vhost")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("/".to_string());
                        
                        let exchange_type = exchange_obj.get("type")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("direct".to_string());
                        
                        let durable = exchange_obj.get("durable")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(true);
                        
                        let auto_delete = exchange_obj.get("auto_delete")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(false);
                        
                        let internal = exchange_obj.get("internal")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(false);
                        
                        let url = format!("http://{}:{}/api/exchanges/{}/{}", host, management_port, 
                            urlencoding::encode(&vhost), urlencoding::encode(&exchange_name));
                        
                        let arguments = exchange_obj.get("arguments")
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
                        
                        let response = client.put(&url)
                            .basic_auth(username, Some(password))
                            .header("Content-Type", "application/json")
                            .json(&body)
                            .send()?;
                        
                        debug!("Created exchange '{}': {}", exchange_name, response.status());
                    }
                }
            }
        }

        // Import queues
        if let Some(queues) = obj.get("queues") {
            if let Some(queues_array) = queues.as_array() {
                for queue in queues_array.values.iter() {
                    if let Some(queue_obj) = queue.as_object() {
                        let queue_name = queue_obj.get("name")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());
                        
                        let vhost = queue_obj.get("vhost")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("/".to_string());
                        
                        let durable = queue_obj.get("durable")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(true);
                        
                        let auto_delete = queue_obj.get("auto_delete")
                            .and_then(|v| v.as_bool())
                            .copied()
                            .unwrap_or(false);
                        
                        let url = format!("http://{}:{}/api/queues/{}/{}", host, management_port, 
                            urlencoding::encode(&vhost), urlencoding::encode(&queue_name));
                        
                        let arguments = queue_obj.get("arguments")
                            .and_then(|v| v.as_object())
                            .map(|_| serde_json::json!({}))
                            .unwrap_or(serde_json::json!({}));
                        
                        let body = serde_json::json!({
                            "durable": durable,
                            "auto_delete": auto_delete,
                            "arguments": arguments
                        });
                        
                        let response = client.put(&url)
                            .basic_auth(username, Some(password))
                            .header("Content-Type", "application/json")
                            .json(&body)
                            .send()?;
                        
                        debug!("Created queue '{}': {}", queue_name, response.status());
                    }
                }
            }
        }

        // Import bindings
        if let Some(bindings) = obj.get("bindings") {
            if let Some(bindings_array) = bindings.as_array() {
                for binding in bindings_array.values.iter() {
                    if let Some(binding_obj) = binding.as_object() {
                        let source = binding_obj.get("source")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());
                        
                        let destination = binding_obj.get("destination")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());
                        
                        let destination_type = binding_obj.get("destination_type")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("queue".to_string());
                        
                        let vhost = binding_obj.get("vhost")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("/".to_string());
                        
                        let routing_key = binding_obj.get("routing_key")
                            .and_then(|v| v.as_string_b())
                            .map(|s| s.as_string())
                            .unwrap_or("".to_string());
                        
                        let url = format!("http://{}:{}/api/bindings/{}/e/{}/{}/{}", 
                            host, management_port, 
                            urlencoding::encode(&vhost), 
                            urlencoding::encode(&source), 
                            destination_type, 
                            urlencoding::encode(&destination));
                        
                        let arguments = binding_obj.get("arguments")
                            .and_then(|v| v.as_object())
                            .map(|_| serde_json::json!({}))
                            .unwrap_or(serde_json::json!({}));
                        
                        let body = serde_json::json!({
                            "routing_key": routing_key,
                            "arguments": arguments
                        });
                        
                        let response = client.post(&url)
                            .basic_auth(username, Some(password))
                            .header("Content-Type", "application/json")
                            .json(&body)
                            .send()?;
                        
                        debug!("Created binding '{}' -> '{}': {}", source, destination, response.status());
                    }
                }
            }
        }
    }

    Ok(())
}
