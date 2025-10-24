use phlow_sdk::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    System,
    Assistant,
    User,
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

impl FromValueBehavior for Role {
    type Item = Role;

    fn from_value(value: Value) -> Option<Self::Item> {
        match value.as_str() {
            "system" => Some(Role::System),
            "assistant" => Some(Role::Assistant),
            "user" => Some(Role::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl FromValueBehavior for Message {
    type Item = Message;

    fn from_value(value: Value) -> Option<Self::Item> {
        let role = Role::from_value(value.get("role")?.clone())?;
        let content = value.get("content")?.to_string();

        Some(Message { role, content })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompletionsBody {
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

impl FromValueBehavior for CompletionsBody {
    type Item = CompletionsBody;

    fn from_value(value: Value) -> Option<Self::Item> {
        let model = value.get("model")?.to_string();

        let prompt = match value.get("prompt") {
            Some(p) => {
                if p.is_array() {
                    Some(
                        p.as_array()?
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
                        s.as_array()?
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
                    None => return None,
                }

                Some(bias_map)
            }
            None => None,
        };

        let user = value.get("user").map(|u| u.to_string());

        Some(CompletionsBody {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ChatBody {
    pub model: String,
    pub messages: Vec<Message>,
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

impl FromValueBehavior for ChatBody {
    type Item = ChatBody;

    fn from_value(value: Value) -> Option<Self::Item> {
        let model = value.get("model")?.to_string();

        let messages_value = value.get("messages")?;
        let messages_array = messages_value.as_array()?;

        let mut messages = Vec::new();
        for msg_value in messages_array {
            let msg = Message::from_value(msg_value.clone())?;
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
                        s.as_array()?
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
                    None => return None,
                }

                Some(bias_map)
            }
            None => None,
        };

        let user = value.get("user").map(|u| u.to_string());

        Some(ChatBody {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ImagesBody {
    pub prompt: String,
    pub n: Option<i32>,
    pub size: Option<String>,
    pub response_format: Option<String>,
    pub user: Option<String>,
}

impl FromValueBehavior for ImagesBody {
    type Item = ImagesBody;

    fn from_value(value: Value) -> Option<Self::Item> {
        let prompt = value.get("prompt")?.to_string();

        let n = value.get("n").map(|n| n.to_i64().unwrap() as i32);

        let size = value.get("size").map(|s| s.to_string());

        let response_format = value.get("response_format").map(|r| r.to_string());

        let user = value.get("user").map(|u| u.to_string());

        Some(ImagesBody {
            prompt,
            n,
            size,
            response_format,
            user,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImagesEditBody {
    pub image: String,
    pub mask: Option<String>,
    pub images_body: ImagesBody,
}

impl FromValueBehavior for ImagesEditBody {
    type Item = ImagesEditBody;

    fn from_value(value: Value) -> Option<Self::Item> {
        let image = value.get("image")?.to_string();

        let mask = value.get("mask").map(|m| m.to_string());

        let images_body_value = value.get("images_body")?;
        let images_body = ImagesBody::from_value(images_body_value.clone())?;

        Some(ImagesEditBody {
            image,
            mask,
            images_body,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddingsBody {
    pub model: String,
    pub input: Vec<String>,
    pub user: Option<String>,
}

impl FromValueBehavior for EmbeddingsBody {
    type Item = EmbeddingsBody;

    fn from_value(value: Value) -> Option<Self::Item> {
        let model = value.get("model")?.to_string();

        let input_value = value.get("input")?;
        let input_array = input_value.as_array()?;

        let mut input = Vec::new();
        for inp in input_array {
            input.push(inp.to_string());
        }

        let user = value.get("user").map(|u| u.to_string());

        Some(EmbeddingsBody { model, input, user })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioBody {
    pub file: String,
    pub model: String,
    pub prompt: Option<String>,
    pub response_format: Option<String>,
    pub temperature: Option<f32>,
    pub language: Option<String>,
}

impl FromValueBehavior for AudioBody {
    type Item = AudioBody;

    fn from_value(value: Value) -> Option<Self::Item> {
        let file = value.get("file")?.to_string();

        let model = value.get("model")?.to_string();

        let prompt = value.get("prompt").map(|p| p.to_string());

        let response_format = value.get("response_format").map(|r| r.to_string());

        let temperature = value.get("temperature").map(|t| t.to_f64().unwrap() as f32);

        let language = value.get("language").map(|l| l.to_string());

        Some(AudioBody {
            file,
            model,
            prompt,
            response_format,
            temperature,
            language,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpenaiApi {
    Completions(CompletionsBody),
    Chat(ChatBody),
    Images(ImagesBody),
    ImagesEdit(ImagesEditBody),
    Embeddings(EmbeddingsBody),
    Audio(AudioBody),
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

impl FromValueBehavior for OpenaiConfig {
    type Item = OpenaiConfig;

    fn from_value(value: Value) -> Option<Self::Item> {
        let action_str = value.get("action")?.as_str();

        let action = match action_str {
            "completions" => OpenaiAction::Completions,
            "chat" => OpenaiAction::Chat,
            "images" => OpenaiAction::Images,
            "images_edit" => OpenaiAction::ImagesEdit,
            "embeddings" => OpenaiAction::Embeddings,
            "audio" => OpenaiAction::Audio,
            _ => return None,
        };

        let with_value = value.get("with")?;

        let with = match action {
            OpenaiAction::Completions => {
                let body = CompletionsBody::from_value(with_value.clone())?;
                OpenaiApi::Completions(body)
            }
            OpenaiAction::Chat => {
                let body = ChatBody::from_value(with_value.clone())?;
                OpenaiApi::Chat(body)
            }
            OpenaiAction::Images => {
                let body = ImagesBody::from_value(with_value.clone())?;
                OpenaiApi::Images(body)
            }
            OpenaiAction::ImagesEdit => {
                let body = ImagesEditBody::from_value(with_value.clone())?;
                OpenaiApi::ImagesEdit(body)
            }
            OpenaiAction::Embeddings => {
                let body = EmbeddingsBody::from_value(with_value.clone())?;
                OpenaiApi::Embeddings(body)
            }
            OpenaiAction::Audio => {
                let body = AudioBody::from_value(with_value.clone())?;
                OpenaiApi::Audio(body)
            }
        };

        Some(OpenaiConfig { action, with })
    }
}
