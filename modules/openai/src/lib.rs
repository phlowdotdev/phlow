mod input;
mod setup;
use crate::input::{OpenaiApi, OpenaiInput};
use crate::setup::Setup;
use openai_api_rust::audio::AudioApi;
use openai_api_rust::completions::CompletionsApi;
use openai_api_rust::embeddings::EmbeddingsApi;
use openai_api_rust::*;
use openai_api_rust::{chat::*, images::ImagesApi};
use phlow_sdk::{prelude::*, valu3};

create_step!(openai(setup));

macro_rules! success_response {
    ($data:expr) => {
        Value::from(
            json!({
                "success": true,
                "data": $data
            })
        )
    };
}

macro_rules! error_response {
    ($message:expr) => {
        Value::from(
            json!({
                "success": false,
                "error": $message
            })
        )
    };
}

pub async fn openai(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    let setup = Setup::try_from(setup.with)?;
    let openai = OpenAI::new(setup.auth.clone(), "https://api.openai.com/v1/");

    println!("OpenAI module initialized with model: {}", setup.model);

    for package in rx {
        let mut input = package.input().unwrap_or(Value::Null);

        // Ensure the model is set in the input if it was specified in setup
        if input.get("model").is_none() {
            input.insert("model", setup.model.clone());
        }

        let config = match OpenaiInput::try_from(input) {
            Ok(cfg) => cfg,
            Err(e) => {
                println!("Error parsing OpenAI config: {}", e);
                sender_safe!(
                    package.sender,
                    Value::from(format!("Error parsing OpenAI config: {}", e)).into()
                );
                continue;
            }
        };

        let response: Value = match config.with {
            OpenaiApi::Chat(body) => match openai.chat_completion_create(&body.into()) {
                Ok(completion) => success_response!(
                    valu3::serde_value::to_value(&completion)
                        .expect("Failed to convert completion to Value")
                ),
                Err(e) => error_response!(format!("Error creating chat completion: {}", e)),
            },
            OpenaiApi::Completions(body) => match openai.completion_create(&body.into()) {
                Ok(completion) => success_response!(
                    valu3::serde_value::to_value(&completion)
                        .expect("Failed to convert completion to Value")
                ),
                Err(e) => error_response!(format!("Error creating completion: {}", e)),
            },
            OpenaiApi::AudioTranslate(body) => match openai.audio_translation_create(body.into()) {
                Ok(transcription) => success_response!(
                    valu3::serde_value::to_value(&transcription)
                        .expect("Failed to convert transcription to Value")
                ),
                Err(e) => error_response!(format!("Error creating audio transcription: {}", e)),
            },
            OpenaiApi::AudioTranscribe(body) => {
                match openai.audio_transcription_create(body.into()) {
                    Ok(transcription) => success_response!(
                        valu3::serde_value::to_value(&transcription)
                            .expect("Failed to convert transcription to Value")
                    ),
                    Err(e) => error_response!(format!("Error creating audio transcription: {}", e)),
                }
            }
            OpenaiApi::ImagesEdit(body) => match openai.image_edit(body.into()) {
                Ok(images) => success_response!(
                    valu3::serde_value::to_value(&images)
                        .expect("Failed to convert images to Value")
                ),
                Err(e) => error_response!(format!("Error creating image edit: {}", e)),
            },
            OpenaiApi::ImagesCreate(body) => match openai.image_create(&body.into()) {
                Ok(images) => success_response!(
                    valu3::serde_value::to_value(&images)
                        .expect("Failed to convert images to Value")
                ),
                Err(e) => error_response!(format!("Error creating images: {}", e)),
            },
            OpenaiApi::Embeddings(body) => match openai.embeddings_create(&body.into()) {
                Ok(embeddings) => success_response!(
                    valu3::serde_value::to_value(&embeddings)
                        .expect("Failed to convert embeddings to Value")
                ),
                Err(e) => error_response!(format!("Error creating embeddings: {}", e)),
            },
        };

        sender_safe!(package.sender, response.into());
    }

    println!("OpenAI module shutting down...");

    Ok(())
}
