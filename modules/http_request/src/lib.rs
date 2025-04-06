mod input;
mod request;
use input::Input;
use phlow_sdk::{listen, prelude::*, sender_safe};
use std::collections::HashMap;

create_step!(http_request(rx));

pub async fn http_request(
    rx: ModuleReceiver,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let default_user_agent = {
        let string_bool = std::env::var("PHLOW_HTTP_REQUEST_USER_AGENT_DISABLE")
            .unwrap_or_else(|_| "false".to_string());

        match string_bool.as_str() {
            "true" => None,
            _ => {
                let default_user_agent = std::env::var("PHLOW_HTTP_REQUEST_USER_AGENT")
                    .unwrap_or_else(|_| "Phlow HTTP Request".to_string())
                    .to_string();

                Some(default_user_agent)
            }
        }
    };

    listen!(rx, resolve, default_user_agent);

    Ok(())
}

pub async fn resolve(package: ModulePackage, default_user_agent: Option<String>) {
    let response = match package.context.input {
        Some(value) => {
            let input = Input::new(value, &default_user_agent);
            match request::request(input) {
                Ok(value) => HashMap::from([
                    ("response", value),
                    ("is_success", true.to_value()),
                    ("is_error", false.to_value()),
                    ("message", "Request successful".to_value()),
                ]),
                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                    HashMap::from([
                        ("response", Value::Undefined),
                        ("is_success", false.to_value()),
                        ("is_error", true.to_value()),
                        ("message", format!("{:?}", e).to_value()),
                    ])
                }
            }
        }
        _ => HashMap::from([
            ("response", Value::Undefined),
            ("is_success", false.to_value()),
            ("is_error", true.to_value()),
            ("message", "No input provided".to_value()),
        ]),
    }
    .to_value();

    sender_safe!(package.sender, response);
}
