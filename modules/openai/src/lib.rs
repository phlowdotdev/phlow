mod config;
use openai_api_rust::chat::*;
use openai_api_rust::*;
use phlow_sdk::prelude::*;

use crate::config::{OpenaiAction, OpenaiApi, OpenaiConfig};

create_step!(openapi(rx));

pub async fn openapi(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    listen!(rx, move |package: ModulePackage| async {
        let input = package.input().unwrap_or(Value::Null);
        let config = match OpenaiConfig::try_from(input) {
            Ok(cfg) => cfg,
            Err(e) => {
                sender_safe!(
                    package.sender,
                    Value::from(format!("Error parsing OpenAI config: {}", e)).into()
                );
                return;
            }
        }

        let auth = Auth::from_env().unwrap();
        let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

        let response = match config.with {
            OpenaiApi::Completions(body) => openai.chat_completion_create(&body),
            // Additional actions can be handled here
        }

        let rs = openai.chat_completion_create(&body);
        let choice = rs.unwrap().choices;
        let message = &choice[0].message.as_ref().unwrap();
        assert!(message.content.contains("Hello"));

        sender_safe!(package.sender, input.into());
    });

    Ok(())
}
