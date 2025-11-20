use phlow_sdk::prelude::*;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub enum OpenaiAction {
    Completions,
    Chat,
    ImagesCreate,
    ImagesEdit,
    Embeddings,
    AudioTranslate,
    AudioTranscribe,
    Responses,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenaiInput {
    pub action: OpenaiAction,
    pub with: Value,
}

impl TryFrom<Value> for OpenaiInput {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let action_str = value
            .get("action")
            .ok_or_else(|| "missing field 'action' for OpenaiConfig".to_string())?
            .as_str();

        let action = match action_str {
            "completions" => OpenaiAction::Completions,
            "chat" => OpenaiAction::Chat,
            "images_create" => OpenaiAction::ImagesCreate,
            "images_edit" => OpenaiAction::ImagesEdit,
            "embeddings" => OpenaiAction::Embeddings,
            "audio_translate" => OpenaiAction::AudioTranslate,
            "audio_transcribe" => OpenaiAction::AudioTranscribe,
            "responses" => OpenaiAction::Responses,
            _ => return Err(format!("invalid action: {}", action_str)),
        };

        // Usa todo o input como body, removendo apenas o campo 'action'
        let mut with = value.clone();
        with.remove(&"action");

        Ok(OpenaiInput { action, with })
    }
}
