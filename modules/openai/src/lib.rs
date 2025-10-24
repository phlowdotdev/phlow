use base64::{engine::general_purpose, Engine as _};
use phlow_sdk::prelude::*;
use reqwest::multipart::{Form, Part};
use serde_json::json;
use std::env;

create_step!(openai(setup));

#[derive(Debug, Clone)]
struct OpenAIConfig {
    api_key: String,
}

impl TryFrom<Value> for OpenAIConfig {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // Try to get api_key from 'with' value
        if let Some(key_val) = value.get("api_key") {
            let key = key_val.to_string();
            if key.is_empty() {
                return Err("api_key is empty".to_string());
            }

            return Ok(OpenAIConfig { api_key: key });
        }

        // Fallback to env
        if let Ok(k) = env::var("OPENAI_API_KEY") {
            if k.is_empty() {
                return Err("OPENAI_API_KEY is empty".to_string());
            }
            return Ok(OpenAIConfig { api_key: k });
        }

        Err("Missing OpenAI API key (with.api_key or OPENAI_API_KEY)".to_string())
    }
}

#[derive(Debug, Clone)]
enum OpenAIInput {
    Chat {
        messages: Value,
        model: Option<String>,
    },
    Completion {
        prompt: String,
        model: Option<String>,
        temperature: Option<f64>,
    },
    Embeddings {
        input_text: String,
        model: Option<String>,
    },
    AudioTranscribe {
        audio_base64: String,
        model: Option<String>,
        language: Option<String>,
    },
    AudioTranslate {
        audio_base64: String,
        model: Option<String>,
    },
    ImageGenerate {
        prompt: String,
        size: Option<String>,
        model: Option<String>,
    },
    ImageEdit {
        image_base64: String,
        prompt: String,
        size: Option<String>,
    },
}

impl TryFrom<Option<Value>> for OpenAIInput {
    type Error = String;

    fn try_from(value: Option<Value>) -> Result<Self, Self::Error> {
        let v = value.ok_or("Missing input")?;

        let action = match v.get("action") {
            Some(av) => av.to_string().to_lowercase(),
            None => return Err("Missing required field 'action'".to_string()),
        };

        match action.as_str() {
            "chat" => {
                let messages = v.get("messages").cloned().unwrap_or(Value::Null);
                let model = v.get("model").and_then(|m| Some(m.to_string()));
                Ok(OpenAIInput::Chat { messages, model })
            }
            "completion" => {
                let prompt = v
                    .get("prompt")
                    .and_then(|p| Some(p.to_string()))
                    .ok_or("Missing 'prompt' for completion")?;
                let model = v.get("model").and_then(|m| Some(m.to_string()));
                let temperature = v.get("temperature").and_then(Value::to_f64);
                Ok(OpenAIInput::Completion {
                    prompt,
                    model,
                    temperature,
                })
            }
            "embeddings" => {
                let input_text = v
                    .get("input_text")
                    .and_then(|p| Some(p.to_string()))
                    .ok_or("Missing 'input_text' for embeddings")?;
                let model = v.get("model").and_then(|m| Some(m.to_string()));
                Ok(OpenAIInput::Embeddings { input_text, model })
            }
            "audio_transcribe" => {
                let audio_base64 = v
                    .get("audio_base64")
                    .and_then(|p| Some(p.to_string()))
                    .ok_or("Missing 'audio_base64' for audio_transcribe")?;
                let model = v.get("model").and_then(|m| Some(m.to_string()));
                let language = v.get("language").and_then(|l| Some(l.to_string()));
                Ok(OpenAIInput::AudioTranscribe {
                    audio_base64,
                    model,
                    language,
                })
            }
            "audio_translate" => {
                let audio_base64 = v
                    .get("audio_base64")
                    .and_then(|p| Some(p.to_string()))
                    .ok_or("Missing 'audio_base64' for audio_translate")?;
                let model = v.get("model").and_then(|m| Some(m.to_string()));
                Ok(OpenAIInput::AudioTranslate {
                    audio_base64,
                    model,
                })
            }
            "image_generate" => {
                let prompt = v
                    .get("image_prompt")
                    .and_then(|p| Some(p.to_string()))
                    .ok_or("Missing 'image_prompt' for image_generate")?;
                let size = v.get("size").and_then(|s| Some(s.to_string()));
                let model = v.get("model").and_then(|m| Some(m.to_string()));
                Ok(OpenAIInput::ImageGenerate {
                    prompt,
                    size,
                    model,
                })
            }
            "image_edit" => {
                let image_base64 = v
                    .get("image_base64")
                    .and_then(|p| Some(p.to_string()))
                    .ok_or("Missing 'image_base64' for image_edit")?;
                let prompt = v
                    .get("prompt")
                    .and_then(|p| Some(p.to_string()))
                    .ok_or("Missing 'prompt' for image_edit")?;
                let size = v.get("size").and_then(|s| Some(s.to_string()));
                Ok(OpenAIInput::ImageEdit {
                    image_base64,
                    prompt,
                    size,
                })
            }
            other => Err(format!("Unsupported action: {}", other)),
        }
    }
}

pub async fn openai(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    // Parse config
    let config = match OpenAIConfig::try_from(setup.with.clone()) {
        Ok(c) => c,
        Err(e) => {
            log::error!("OpenAI module configuration error: {}", e);
            return Err(e.into());
        }
    };

    let client = reqwest::Client::new();

    for package in rx {
        let client = client.clone();
        let config = config.clone();

        // Spawn task per request to avoid blocking others
        tokio::spawn(async move {
            let input = match OpenAIInput::try_from(package.input.clone()) {
                Ok(i) => i,
                Err(e) => {
                    let response = ModuleResponse::from_error(format!("Invalid input: {}", e));
                    sender_safe!(package.sender, response.into());
                    return;
                }
            };

            let api_key = config.api_key;

            // Default model names
            let default_chat_model = "gpt-5-mini";
            let default_completion_model = "text-davinci-003";
            let default_embeddings_model = "text-embedding-3-small";

            let base_url = "https://api.openai.com/v1";

            let result = match input {
                OpenAIInput::Chat { messages, model } => {
                    let model = model.unwrap_or(default_chat_model.to_string());

                    // If feature `use_openai_crate` is enabled, prefer the openai_api_rust client
                    #[cfg(feature = "use_openai_crate")]
                    {
                        // Use closure to unify error handling

                        let call = || -> Result<Value, String> {
                            use openai_api_rust::apis::{Message, Role};
                            use openai_api_rust::chat::{ChatApi, ChatBody};
                            use openai_api_rust::{Auth, OpenAI};

                            // Convert messages (phlow Value) to Vec<Message>
                            let mut vec_msgs: Vec<Message> = Vec::new();
                            if let Value::Array(arr) = messages {
                                for m in arr.into_iter() {
                                    let role_str = m
                                        .get("role")
                                        .map(|r| r.to_string())
                                        .unwrap_or("user".to_string());
                                    let role = match role_str.as_str() {
                                        "system" => Role::System,
                                        "assistant" => Role::Assistant,
                                        _ => Role::User,
                                    };
                                    let content =
                                        m.get("content").map(|c| c.to_string()).unwrap_or_default();
                                    vec_msgs.push(Message { role, content });
                                }
                            }

                            let auth = Auth::new(&api_key);
                            let openai = OpenAI::new(auth, base_url);

                            let body = ChatBody {
                                model: model.clone(),
                                max_tokens: None,
                                temperature: None,
                                top_p: None,
                                n: None,
                                stream: None,
                                stop: None,
                                presence_penalty: None,
                                frequency_penalty: None,
                                logit_bias: None,
                                user: None,
                                messages: vec_msgs,
                            };

                            let rs = openai
                                .chat_completion_create(&body)
                                .map_err(|e| format!("openai_api_rust error: {}", e.to_string()))?;

                            serde_json::to_string(&rs)
                                .map(|s| Value::from(s))
                                .map_err(|e| e.to_string())
                        };

                        match call() {
                            Ok(v) => Ok(v),
                            Err(e) => Err(e),
                        }
                    }

                    #[cfg(not(feature = "use_openai_crate"))]
                    {
                        let body = json!({
                            "model": model,
                            "messages": messages
                        });

                        let res = client
                            .post(format!("{}/chat/completions", base_url))
                            .bearer_auth(api_key)
                            .json(&body)
                            .send()
                            .await;

                        match res {
                            Ok(resp) => match resp.json::<serde_json::Value>().await {
                                Ok(json) => match serde_json::to_string(&json) {
                                    Ok(s) => Ok(Value::from(s)),
                                    Err(e) => Err(format!("Failed to serialize response: {}", e)),
                                },
                                Err(e) => Err(format!("Failed to parse response JSON: {}", e)),
                            },
                            Err(e) => Err(format!("Request error: {}", e)),
                        }
                    }
                }
                OpenAIInput::Completion {
                    prompt,
                    model,
                    temperature,
                } => {
                    let model = model.unwrap_or(default_completion_model.to_string());
                    let mut body = json!({
                        "model": model,
                        "prompt": prompt,
                    });

                    if let Some(t) = temperature {
                        body["temperature"] = json!(t);
                    }

                    let res = client
                        .post(format!("{}/completions", base_url))
                        .bearer_auth(api_key)
                        .json(&body)
                        .send()
                        .await;

                    match res {
                        Ok(resp) => match resp.json::<serde_json::Value>().await {
                            Ok(json) => match serde_json::to_string(&json) {
                                Ok(s) => Ok(Value::from(s)),
                                Err(e) => Err(format!("Failed to serialize response: {}", e)),
                            },
                            Err(e) => Err(format!("Failed to parse response JSON: {}", e)),
                        },
                        Err(e) => Err(format!("Request error: {}", e)),
                    }
                }
                OpenAIInput::Embeddings { input_text, model } => {
                    let model = model.unwrap_or(default_embeddings_model.to_string());
                    let body = json!({
                        "model": model,
                        "input": input_text,
                    });

                    let res = client
                        .post(format!("{}/embeddings", base_url))
                        .bearer_auth(api_key)
                        .json(&body)
                        .send()
                        .await;

                    match res {
                        Ok(resp) => match resp.json::<serde_json::Value>().await {
                            Ok(json) => match serde_json::to_string(&json) {
                                Ok(s) => Ok(Value::from(s)),
                                Err(e) => Err(format!("Failed to serialize response: {}", e)),
                            },
                            Err(e) => Err(format!("Failed to parse response JSON: {}", e)),
                        },
                        Err(e) => Err(format!("Request error: {}", e)),
                    }
                }
                OpenAIInput::AudioTranscribe {
                    audio_base64,
                    model,
                    language,
                } => {
                    let model = model.unwrap_or_else(|| "whisper-1".to_string());
                    let b64 = audio_base64.trim_matches('"');
                    let bytes = match general_purpose::STANDARD.decode(b64) {
                        Ok(b) => b,
                        Err(e) => {
                            return sender_safe!(
                                package.sender,
                                std::collections::HashMap::from([
                                    ("success", false.to_value()),
                                    (
                                        "error",
                                        format!("Invalid base64 audio: {}", e)
                                            .to_string()
                                            .to_value()
                                    ),
                                ])
                                .to_value()
                                .into()
                            )
                        }
                    };

                    let mut form = Form::new()
                        .part("file", Part::bytes(bytes).file_name("audio.wav"))
                        .text("model", model.clone());

                    if let Some(lang) = language {
                        form = form.text("language", lang);
                    }

                    let res = client
                        .post(format!("{}/audio/transcriptions", base_url))
                        .bearer_auth(api_key)
                        .multipart(form)
                        .send()
                        .await;

                    match res {
                        Ok(resp) => match resp.json::<serde_json::Value>().await {
                            Ok(json) => match serde_json::to_string(&json) {
                                Ok(s) => Ok(Value::from(s)),
                                Err(e) => Err(format!("Failed to serialize response: {}", e)),
                            },
                            Err(e) => Err(format!("Failed to parse response JSON: {}", e)),
                        },
                        Err(e) => Err(format!("Request error: {}", e)),
                    }
                }
                OpenAIInput::AudioTranslate {
                    audio_base64,
                    model,
                } => {
                    let model = model.unwrap_or_else(|| "whisper-1".to_string());
                    let b64 = audio_base64.trim_matches('"');
                    let bytes = match general_purpose::STANDARD.decode(b64) {
                        Ok(b) => b,
                        Err(e) => {
                            return sender_safe!(
                                package.sender,
                                std::collections::HashMap::from([
                                    ("success", false.to_value()),
                                    (
                                        "error",
                                        format!("Invalid base64 audio: {}", e)
                                            .to_string()
                                            .to_value()
                                    ),
                                ])
                                .to_value()
                                .into()
                            )
                        }
                    };

                    let form = Form::new()
                        .part("file", Part::bytes(bytes).file_name("audio.wav"))
                        .text("model", model.clone());

                    let res = client
                        .post(format!("{}/audio/translations", base_url))
                        .bearer_auth(api_key)
                        .multipart(form)
                        .send()
                        .await;

                    match res {
                        Ok(resp) => match resp.json::<serde_json::Value>().await {
                            Ok(json) => match serde_json::to_string(&json) {
                                Ok(s) => Ok(Value::from(s)),
                                Err(e) => Err(format!("Failed to serialize response: {}", e)),
                            },
                            Err(e) => Err(format!("Failed to parse response JSON: {}", e)),
                        },
                        Err(e) => Err(format!("Request error: {}", e)),
                    }
                }
                OpenAIInput::ImageGenerate {
                    prompt,
                    size,
                    model,
                } => {
                    // Image generation endpoint
                    let mut body = json!({
                        "prompt": prompt,
                    });
                    if let Some(s) = size {
                        body["size"] = json!(s);
                    }
                    if let Some(m) = model {
                        body["model"] = json!(m);
                    }

                    let res = client
                        .post(format!("{}/images/generations", base_url))
                        .bearer_auth(api_key)
                        .json(&body)
                        .send()
                        .await;

                    match res {
                        Ok(resp) => match resp.json::<serde_json::Value>().await {
                            Ok(json) => match serde_json::to_string(&json) {
                                Ok(s) => Ok(Value::from(s)),
                                Err(e) => Err(format!("Failed to serialize response: {}", e)),
                            },
                            Err(e) => Err(format!("Failed to parse response JSON: {}", e)),
                        },
                        Err(e) => Err(format!("Request error: {}", e)),
                    }
                }
                OpenAIInput::ImageEdit {
                    image_base64,
                    prompt,
                    size,
                } => {
                    let b64 = image_base64.trim_matches('"');
                    let bytes = match general_purpose::STANDARD.decode(b64) {
                        Ok(b) => b,
                        Err(e) => {
                            return sender_safe!(
                                package.sender,
                                std::collections::HashMap::from([
                                    ("success", false.to_value()),
                                    (
                                        "error",
                                        format!("Invalid base64 image: {}", e)
                                            .to_string()
                                            .to_value()
                                    ),
                                ])
                                .to_value()
                                .into()
                            )
                        }
                    };

                    let mut form = Form::new()
                        .part("image", Part::bytes(bytes).file_name("image.png"))
                        .text("prompt", prompt);

                    if let Some(s) = size {
                        form = form.text("size", s);
                    }

                    let res = client
                        .post(format!("{}/images/edits", base_url))
                        .bearer_auth(api_key)
                        .multipart(form)
                        .send()
                        .await;

                    match res {
                        Ok(resp) => match resp.json::<serde_json::Value>().await {
                            Ok(json) => match serde_json::to_string(&json) {
                                Ok(s) => Ok(Value::from(s)),
                                Err(e) => Err(format!("Failed to serialize response: {}", e)),
                            },
                            Err(e) => Err(format!("Failed to parse response JSON: {}", e)),
                        },
                        Err(e) => Err(format!("Request error: {}", e)),
                    }
                }
            };

            match result {
                Ok(data) => {
                    // data is a phlow-sdk Value (stringified JSON)
                    let response = std::collections::HashMap::from([
                        ("success", true.to_value()),
                        ("data", data),
                    ])
                    .to_value();

                    sender_safe!(package.sender, response.into());
                }
                Err(e) => {
                    log::error!("OpenAI request failed: {}", e);
                    let response = std::collections::HashMap::from([
                        ("success", false.to_value()),
                        ("error", e.to_string().to_value()),
                    ])
                    .to_value();

                    sender_safe!(package.sender, response.into());
                }
            }
        });
    }

    Ok(())
}
