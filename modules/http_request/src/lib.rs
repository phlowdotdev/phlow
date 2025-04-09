mod config;
mod input;
mod request;
use config::Config;
use input::Input;
use phlow_sdk::prelude::*;
use reqwest::Client;
use std::collections::HashMap;

create_step!(http_request(setup));

pub async fn http_request(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);
    let config = Config::from(setup.with);

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

    let client = match reqwest::Client::builder()
        .danger_accept_invalid_certs(!config.verify_ssl)
        .timeout(std::time::Duration::from_secs(config.timeout))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Error creating client: {:?}", e);
            return Err(Box::new(e));
        }
    };

    listen!(rx, resolve, default_user_agent, client);

    Ok(())
}

pub async fn resolve(package: ModulePackage, default_user_agent: Option<String>, client: Client) {
    let response = match package.input() {
        Some(value) => {
            let input = Input::new(value, &default_user_agent);
            match request::request(input, client).await {
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
