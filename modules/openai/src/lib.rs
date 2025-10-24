mod config;
use openai_api_rust::audio::AudioApi;
use openai_api_rust::completions::CompletionsApi;
use openai_api_rust::embeddings::EmbeddingsApi;
use openai_api_rust::*;
use openai_api_rust::{chat::*, images::ImagesApi};
use phlow_sdk::{prelude::*, valu3};

use crate::config::{OpenaiApi, OpenaiConfig};

create_step!(openapi(rx));

pub async fn openapi(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    listen!(rx, move |package: ModulePackage| {
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
        };

        let auth = Auth::from_env().unwrap();
        let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

        let response: Value = match config.with {
            OpenaiApi::Chat(body) => match openai.chat_completion_create(&body.into()) {
                Ok(completion) => valu3::serde_value::to_value(&completion)
                    .expect("Failed to convert completion to Value"),
                Err(e) => Value::from(format!("Error creating chat completion: {}", e)),
            },
            OpenaiApi::Completions(body) => match openai.completion_create(&body.into()) {
                Ok(completion) => valu3::serde_value::to_value(&completion)
                    .expect("Failed to convert completion to Value"),
                Err(e) => Value::from(format!("Error creating completion: {}", e)),
            },
            OpenaiApi::AudioTranslate(body) => match openai.audio_translation_create(body.into()) {
                Ok(transcription) => valu3::serde_value::to_value(&transcription)
                    .expect("Failed to convert transcription to Value"),
                Err(e) => Value::from(format!("Error creating audio transcription: {}", e)),
            },
            OpenaiApi::AudioTranscribe(body) => {
                match openai.audio_transcription_create(body.into()) {
                    Ok(transcription) => valu3::serde_value::to_value(&transcription)
                        .expect("Failed to convert transcription to Value"),
                    Err(e) => Value::from(format!("Error creating audio transcription: {}", e)),
                }
            }
            OpenaiApi::ImagesEdit(body) => match openai.image_edit(body.into()) {
                Ok(images) => valu3::serde_value::to_value(&images)
                    .expect("Failed to convert images to Value"),
                Err(e) => Value::from(format!("Error creating image edit: {}", e)),
            },
            OpenaiApi::ImagesCreate(body) => match openai.image_create(&body.into()) {
                Ok(images) => valu3::serde_value::to_value(&images)
                    .expect("Failed to convert images to Value"),
                Err(e) => Value::from(format!("Error creating images: {}", e)),
            },
            OpenaiApi::Embeddings(body) => match openai.embeddings_create(&body.into()) {
                Ok(embeddings) => valu3::serde_value::to_value(&embeddings)
                    .expect("Failed to convert embeddings to Value"),
                Err(e) => Value::from(format!("Error creating embeddings: {}", e)),
            },
        };

        sender_safe!(package.sender, response.into());
    });

    Ok(())
}
