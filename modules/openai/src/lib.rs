mod input;
mod setup;
use openai_api_rust::audio::AudioApi;
use openai_api_rust::completions::CompletionsApi;
use openai_api_rust::embeddings::EmbeddingsApi;
use openai_api_rust::*;
use openai_api_rust::{chat::*, images::ImagesApi};
use phlow_sdk::{prelude::*, valu3};

use crate::input::{OpenaiApi, OpenaiInput};
use crate::setup::Setup;
use std::sync::Arc;

create_step!(openapi(setup));

pub async fn openapi(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    println!("Initializing OpenAI module...");

    let setup = Arc::new(Setup::try_from(setup.with)?);

    println!("OpenAI module initialized with model: {}", setup.model);

    listen!(
        rx,
        move |package: ModulePackage, setup: Arc<Setup>| async move {
            println!("Received package: {:?}", package);
            let mut input = package.input().unwrap_or(Value::Null);

            // Ensure the model is set in the input if it was specified in setup
            if let Some(with) = input.get_mut("with") {
                if let Some(input_model) = with.get("model") {
                    if input_model.is_null() {
                        with.as_object_mut().unwrap().remove(&"model");
                        // clone the setup.model to avoid moving it out of `setup`
                        // which would make the closure non-reusable across iterations
                        with.as_object_mut()
                            .unwrap()
                            .insert("model", setup.as_ref().model.clone());
                    }
                }
            }

            let config = match OpenaiInput::try_from(input) {
                Ok(cfg) => cfg,
                Err(e) => {
                    sender_safe!(
                        package.sender,
                        Value::from(format!("Error parsing OpenAI config: {}", e)).into()
                    );
                    return;
                }
            };

            let openai = OpenAI::new(setup.auth.clone(), "https://api.openai.com/v1/");

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
                OpenaiApi::AudioTranslate(body) => {
                    match openai.audio_translation_create(body.into()) {
                        Ok(transcription) => valu3::serde_value::to_value(&transcription)
                            .expect("Failed to convert transcription to Value"),
                        Err(e) => Value::from(format!("Error creating audio transcription: {}", e)),
                    }
                }
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
        },
        setup
    );

    Ok(())
}
