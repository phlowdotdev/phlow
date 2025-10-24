use openai_api_rust::completions::CompletionsBody;
use openai_api_rust::images::ImagesBody;
use openai_api_rust::{Message, Role};
use phlow_sdk::prelude::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;

#[derive(Debug, Clone, PartialEq)]
pub enum LocalRole {
    System,
    Assistant,
    User,
}

impl Default for LocalRole {
    fn default() -> Self {
        LocalRole::User
    }
}

impl TryFrom<Value> for LocalRole {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.as_str() {
            "system" => Ok(LocalRole::System),
            "assistant" => Ok(LocalRole::Assistant),
            "user" => Ok(LocalRole::User),
            _ => Err(format!("invalid role: {:?}", value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LocalMessage {
    pub role: LocalRole,
    pub content: String,
}

impl TryFrom<Value> for LocalMessage {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let role_val = value
            .get("role")
            .ok_or_else(|| "missing field 'role' for Message".to_string())?
            .clone();

        let role = LocalRole::try_from(role_val)?;

        let content = value
            .get("content")
            .ok_or_else(|| "missing field 'content' for Message".to_string())?
            .to_string();

        Ok(LocalMessage { role, content })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LocalCompletionsBody {
    pub model: String,
    pub prompt: Option<Vec<String>>,
    pub suffix: Option<String>,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub n: Option<i32>,
    pub stream: Option<bool>,
    pub logprobs: Option<i32>,
    pub echo: Option<bool>,
    pub stop: Option<Vec<String>>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub best_of: Option<i32>,
    pub logit_bias: Option<HashMap<String, String>>,
    pub user: Option<String>,
}

impl TryFrom<Value> for LocalCompletionsBody {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let model = value
            .get("model")
            .ok_or_else(|| "missing field 'model' for CompletionsBody".to_string())?
            .to_string();

        let prompt = match value.get("prompt") {
            Some(p) => {
                if p.is_array() {
                    Some(
                        p.as_array()
                            .ok_or_else(|| "invalid 'prompt' array".to_string())?
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>(),
                    )
                } else {
                    Some(vec![p.to_string()])
                }
            }
            None => None,
        };

        let suffix = value.get("suffix").map(|s| s.to_string());

        let max_tokens = value.get("max_tokens").map(|m| m.to_i64().unwrap() as i32);

        let temperature = value.get("temperature").map(|t| t.to_f64().unwrap() as f32);

        let top_p = value.get("top_p").map(|t| t.to_f64().unwrap() as f32);

        let n = value.get("n").map(|n| n.to_i64().unwrap() as i32);

        let stream = value.get("stream").and_then(|s| s.as_bool().cloned());

        let logprobs = value.get("logprobs").map(|l| l.to_i64().unwrap() as i32);

        let echo = value.get("echo").and_then(|e| e.as_bool().cloned());

        let stop = match value.get("stop") {
            Some(s) => {
                if s.is_array() {
                    Some(
                        s.as_array()
                            .ok_or_else(|| "invalid 'stop' array".to_string())?
                            .into_iter()
                            .map(|st| st.to_string())
                            .collect::<Vec<String>>(),
                    )
                } else {
                    Some(vec![s.to_string()])
                }
            }
            None => None,
        };

        let presence_penalty = value
            .get("presence_penalty")
            .map(|p| p.to_f64().unwrap() as f32);

        let frequency_penalty = value
            .get("frequency_penalty")
            .map(|f| f.to_f64().unwrap() as f32);

        let best_of = value.get("best_of").map(|b| b.to_i64().unwrap() as i32);

        let logit_bias = match value.get("logit_bias") {
            Some(lb) => {
                let mut bias_map = HashMap::new();

                match lb.as_object() {
                    Some(obj) => {
                        for (k, v) in obj.iter() {
                            bias_map.insert(k.to_string(), v.to_string());
                        }
                    }
                    None => return Err("invalid 'logit_bias' object".to_string()),
                }

                Some(bias_map)
            }
            None => None,
        };

        let user = value.get("user").map(|u| u.to_string());

        Ok(LocalCompletionsBody {
            model,
            prompt,
            suffix,
            max_tokens,
            temperature,
            top_p,
            n,
            stream,
            logprobs,
            echo,
            stop,
            presence_penalty,
            frequency_penalty,
            best_of,
            logit_bias,
            user,
        })
    }
}

impl Into<CompletionsBody> for LocalCompletionsBody {
    fn into(self) -> CompletionsBody {
        CompletionsBody {
            model: self.model,
            prompt: self.prompt,
            suffix: self.suffix,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            n: self.n,
            stream: self.stream,
            logprobs: self.logprobs,
            echo: self.echo,
            stop: self.stop,
            presence_penalty: self.presence_penalty,
            frequency_penalty: self.frequency_penalty,
            best_of: self.best_of,
            logit_bias: self.logit_bias,
            user: self.user,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalChatBody {
    pub model: String,
    pub messages: Vec<LocalMessage>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub n: Option<i32>,
    pub stream: Option<bool>,
    pub stop: Option<Vec<String>>,
    pub max_tokens: Option<i32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub logit_bias: Option<HashMap<String, String>>,
    pub user: Option<String>,
}

impl TryFrom<Value> for LocalChatBody {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let model = value
            .get("model")
            .ok_or_else(|| "missing field 'model' for ChatBody".to_string())?
            .to_string();

        let messages_value = value
            .get("messages")
            .ok_or_else(|| "missing field 'messages' for ChatBody".to_string())?;
        let messages_array = messages_value
            .as_array()
            .ok_or_else(|| "invalid 'messages' array".to_string())?;

        let mut messages = Vec::new();
        for msg_value in messages_array {
            let msg = LocalMessage::try_from(msg_value.clone())?;
            messages.push(msg);
        }

        let temperature = value.get("temperature").map(|t| t.to_f64().unwrap() as f32);

        let top_p = value.get("top_p").map(|t| t.to_f64().unwrap() as f32);

        let n = value.get("n").map(|n| n.to_i64().unwrap() as i32);

        let stream = value.get("stream").and_then(|s| s.as_bool().cloned());

        let stop = match value.get("stop") {
            Some(s) => {
                if s.is_array() {
                    Some(
                        s.as_array()
                            .ok_or_else(|| "invalid 'stop' array".to_string())?
                            .into_iter()
                            .map(|st| st.to_string())
                            .collect::<Vec<String>>(),
                    )
                } else {
                    Some(vec![s.to_string()])
                }
            }
            None => None,
        };

        let max_tokens = value.get("max_tokens").map(|m| m.to_i64().unwrap() as i32);

        let presence_penalty = value
            .get("presence_penalty")
            .map(|p| p.to_f64().unwrap() as f32);

        let frequency_penalty = value
            .get("frequency_penalty")
            .map(|f| f.to_f64().unwrap() as f32);

        let logit_bias = match value.get("logit_bias") {
            Some(lb) => {
                let mut bias_map = HashMap::new();

                match lb.as_object() {
                    Some(obj) => {
                        for (k, v) in obj.iter() {
                            bias_map.insert(k.to_string(), v.to_string());
                        }
                    }
                    None => return Err("invalid 'logit_bias' object".to_string()),
                }

                Some(bias_map)
            }
            None => None,
        };

        let user = value.get("user").map(|u| u.to_string());

        Ok(LocalChatBody {
            model,
            messages,
            temperature,
            top_p,
            n,
            stream,
            stop,
            max_tokens,
            presence_penalty,
            frequency_penalty,
            logit_bias,
            user,
        })
    }
}

impl Into<openai_api_rust::chat::ChatBody> for LocalChatBody {
    fn into(self) -> openai_api_rust::chat::ChatBody {
        let messages_converted = self
            .messages
            .into_iter()
            .map(|m| Message {
                role: match m.role {
                    LocalRole::System => Role::System,
                    LocalRole::Assistant => Role::Assistant,
                    LocalRole::User => Role::User,
                },
                content: m.content,
            })
            .collect();

        openai_api_rust::chat::ChatBody {
            model: self.model,
            messages: messages_converted,
            temperature: self.temperature,
            top_p: self.top_p,
            n: self.n,
            stream: self.stream,
            stop: self.stop,
            max_tokens: self.max_tokens,
            presence_penalty: self.presence_penalty,
            frequency_penalty: self.frequency_penalty,
            logit_bias: self.logit_bias,
            user: self.user,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalImagesBody {
    pub prompt: String,
    pub n: Option<i32>,
    pub size: Option<String>,
    pub response_format: Option<String>,
    pub user: Option<String>,
}

impl TryFrom<Value> for LocalImagesBody {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let prompt = value
            .get("prompt")
            .ok_or_else(|| "missing field 'prompt' for ImagesBody".to_string())?
            .to_string();

        let n = value.get("n").map(|n| n.to_i64().unwrap() as i32);

        let size = value.get("size").map(|s| s.to_string());

        let response_format = value.get("response_format").map(|r| r.to_string());

        let user = value.get("user").map(|u| u.to_string());

        Ok(LocalImagesBody {
            prompt,
            n,
            size,
            response_format,
            user,
        })
    }
}

impl Into<openai_api_rust::images::ImagesBody> for LocalImagesBody {
    fn into(self) -> openai_api_rust::images::ImagesBody {
        openai_api_rust::images::ImagesBody {
            prompt: self.prompt,
            n: self.n,
            size: self.size,
            response_format: self.response_format,
            user: self.user,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalImagesEditBody {
    pub image: String,
    pub mask: Option<String>,
    pub images_body: LocalImagesBody,
}

impl TryFrom<Value> for LocalImagesEditBody {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let image = value
            .get("image")
            .ok_or_else(|| "missing field 'image' for ImagesEditBody".to_string())?
            .to_string();

        let mask = value.get("mask").map(|m| m.to_string());

        let images_body_value = value
            .get("images_body")
            .ok_or_else(|| "missing field 'images_body' for ImagesEditBody".to_string())?;
        let images_body = LocalImagesBody::try_from(images_body_value.clone())?;

        Ok(LocalImagesEditBody {
            image,
            mask,
            images_body,
        })
    }
}

impl Into<openai_api_rust::images::ImagesEditBody> for LocalImagesEditBody {
    fn into(self) -> openai_api_rust::images::ImagesEditBody {
        let image = File::open(self.image).unwrap();
        let mask = self.mask.map(|m| File::open(m).unwrap());

        openai_api_rust::images::ImagesEditBody {
            image,
            mask,
            images_body: self.images_body.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalEmbeddingsBody {
    pub model: String,
    pub input: Vec<String>,
    pub user: Option<String>,
}

impl TryFrom<Value> for LocalEmbeddingsBody {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let model = value
            .get("model")
            .ok_or_else(|| "missing field 'model' for EmbeddingsBody".to_string())?
            .to_string();

        let input_value = value
            .get("input")
            .ok_or_else(|| "missing field 'input' for EmbeddingsBody".to_string())?;
        let input_array = input_value
            .as_array()
            .ok_or_else(|| "invalid 'input' array".to_string())?;

        let mut input = Vec::new();
        for inp in input_array {
            input.push(inp.to_string());
        }

        let user = value.get("user").map(|u| u.to_string());

        Ok(LocalEmbeddingsBody { model, input, user })
    }
}

impl Into<openai_api_rust::embeddings::EmbeddingsBody> for LocalEmbeddingsBody {
    fn into(self) -> openai_api_rust::embeddings::EmbeddingsBody {
        openai_api_rust::embeddings::EmbeddingsBody {
            model: self.model,
            input: self.input,
            user: self.user,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalAudioBody {
    pub file: String,
    pub model: String,
    pub prompt: Option<String>,
    pub response_format: Option<String>,
    pub temperature: Option<f32>,
    pub language: Option<String>,
}

impl TryFrom<Value> for LocalAudioBody {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let file = value
            .get("file")
            .ok_or_else(|| "missing field 'file' for AudioBody".to_string())?
            .to_string();

        let model = value
            .get("model")
            .ok_or_else(|| "missing field 'model' for AudioBody".to_string())?
            .to_string();

        let prompt = value.get("prompt").map(|p| p.to_string());

        let response_format = value.get("response_format").map(|r| r.to_string());

        let temperature = value.get("temperature").map(|t| t.to_f64().unwrap() as f32);

        let language = value.get("language").map(|l| l.to_string());

        Ok(LocalAudioBody {
            file,
            model,
            prompt,
            response_format,
            temperature,
            language,
        })
    }
}

impl Into<openai_api_rust::audio::AudioBody> for LocalAudioBody {
    fn into(self) -> openai_api_rust::audio::AudioBody {
        let file = File::open(self.file).unwrap();

        openai_api_rust::audio::AudioBody {
            file,
            model: self.model,
            prompt: self.prompt,
            response_format: self.response_format,
            temperature: self.temperature,
            language: self.language,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpenaiApi {
    Completions(LocalCompletionsBody),
    Chat(LocalChatBody),
    Images(LocalImagesBody),
    ImagesEdit(LocalImagesEditBody),
    Embeddings(LocalEmbeddingsBody),
    Audio(LocalAudioBody),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpenaiAction {
    Completions,
    Chat,
    Images,
    ImagesEdit,
    Embeddings,
    Audio,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenaiConfig {
    pub action: OpenaiAction,
    pub with: OpenaiApi,
}

impl TryFrom<Value> for OpenaiConfig {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let action_str = value
            .get("action")
            .ok_or_else(|| "missing field 'action' for OpenaiConfig".to_string())?
            .as_str();

        let action = match action_str {
            "completions" => OpenaiAction::Completions,
            "chat" => OpenaiAction::Chat,
            "images" => OpenaiAction::Images,
            "images_edit" => OpenaiAction::ImagesEdit,
            "embeddings" => OpenaiAction::Embeddings,
            "audio" => OpenaiAction::Audio,
            _ => return Err(format!("invalid action: {}", action_str)),
        };

        let with_value = value
            .get("with")
            .ok_or_else(|| "missing field 'with' for OpenaiConfig".to_string())?;

        let with = match action {
            OpenaiAction::Completions => {
                let body = LocalCompletionsBody::try_from(with_value.clone())?;
                OpenaiApi::Completions(body)
            }
            OpenaiAction::Chat => {
                let body = LocalChatBody::try_from(with_value.clone())?;
                OpenaiApi::Chat(body)
            }
            OpenaiAction::Images => {
                let body = LocalImagesBody::try_from(with_value.clone())?;
                OpenaiApi::Images(body)
            }
            OpenaiAction::ImagesEdit => {
                let body = LocalImagesEditBody::try_from(with_value.clone())?;
                OpenaiApi::ImagesEdit(body)
            }
            OpenaiAction::Embeddings => {
                let body = LocalEmbeddingsBody::try_from(with_value.clone())?;
                OpenaiApi::Embeddings(body)
            }
            OpenaiAction::Audio => {
                let body = LocalAudioBody::try_from(with_value.clone())?;
                OpenaiApi::Audio(body)
            }
        };

        Ok(OpenaiConfig { action, with })
    }
}
