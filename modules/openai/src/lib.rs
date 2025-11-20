mod input;
mod setup;
use crate::input::{OpenaiAction, OpenaiInput};
use crate::setup::Setup;
use phlow_sdk::prelude::*;
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::multipart::{Form, Part};
use serde_json::Value as JsonValue;

create_step!(openai(setup));

macro_rules! success_response {
    ($data:expr) => {
        json!({
            "success": true,
            "data": $data
        }).into()
    };
}

macro_rules! error_response {
    ($message:expr) => {
        json!({
            "success": false,
            "error": $message
        }).into()
    };
}

pub async fn openai(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    let setup = Setup::try_from(setup.with)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", setup.api_key))?,
    );

    let mut client_builder = Client::builder().default_headers(headers);

    if let Some(proxy) = &setup.proxy {
        client_builder = client_builder.proxy(reqwest::Proxy::all(proxy)?);
    }

    let client = client_builder.build()?;

    let base = setup.api_url.trim_end_matches('/').to_string();

    log::debug!("OpenAI module initialized with model: {}", setup.model);

    for package in rx {
        let mut input = package.input().unwrap_or(Value::Null);

        // Garante model default no nível raiz, sem validar payloads
        if input.get("model").is_none() {
            input.insert("model", setup.model.clone());
        }

        let openai_input = match OpenaiInput::try_from(input) {
            Ok(cfg) => cfg,
            Err(e) => {
                log::error!("Error parsing OpenAI config: {}", e);
                sender_safe!(
                    package.sender,
                    Value::from(format!("Error parsing OpenAI config: {}", e)).into()
                );
                continue;
            }
        };
        let url: String = match openai_input.action {
            OpenaiAction::Chat => format!("{}/chat/completions", base),
            OpenaiAction::Completions => format!("{}/completions", base),
            OpenaiAction::AudioTranscribe => format!("{}/audio/transcriptions", base),
            OpenaiAction::AudioTranslate => format!("{}/audio/translations", base),
            OpenaiAction::Embeddings => format!("{}/embeddings", base),
            OpenaiAction::ImagesEdit => format!("{}/images/edits", base),
            OpenaiAction::ImagesCreate => format!("{}/images/generations", base),
            OpenaiAction::Responses => format!("{}/responses", base),
        };

        // Corpo bruto como JSON (sem validação)
        let body_json: JsonValue =
            match serde_json::from_str::<JsonValue>(&openai_input.with.to_string()) {
                Ok(v) => v,
                Err(err) => {
                    let msg = format!("Invalid JSON body: {}", err);
                    log::error!("{}", msg);
                    sender_safe!(package.sender, error_response!(msg));
                    continue;
                }
            };

        // Envia conforme o tipo de ação (JSON vs multipart)
        let result = match openai_input.action {
            OpenaiAction::AudioTranscribe
            | OpenaiAction::AudioTranslate
            | OpenaiAction::ImagesEdit => {
                // Constrói multipart dinamicamente: arquivos em chaves [file|image|mask] se apontarem para paths existentes
                let form = match build_multipart_form(&body_json) {
                    Ok(f) => f,
                    Err(e) => {
                        let msg = format!("Erro ao montar multipart: {}", e);
                        log::error!("{}", msg);
                        sender_safe!(package.sender, error_response!(msg));
                        continue;
                    }
                };
                client.post(&url).multipart(form).send().await
            }
            _ => client.post(&url).json(&body_json).send().await,
        };

        match result {
            Ok(resp) => {
                let status = resp.status();
                let bytes = match resp.bytes().await {
                    Ok(b) => b,
                    Err(e) => {
                        let msg = format!("Erro lendo resposta: {}", e);
                        log::error!("{}", msg);
                        sender_safe!(package.sender, error_response!(msg));
                        continue;
                    }
                };
                let text = String::from_utf8_lossy(&bytes).to_string();
                if status.is_success() {
                    // Tenta converter JSON; caso contrário, retorna texto cru
                    let data_val: Value = match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(v) => serde_to_value(&v),
                        Err(_) => Value::from(text.clone()),
                    };

                    sender_safe!(package.sender, success_response!(data_val));
                } else {
                    let msg = format!("HTTP {}: {}", status.as_u16(), text);
                    log::error!("{}", msg);
                    sender_safe!(package.sender, error_response!(msg));
                }
            }
            Err(e) => {
                let msg = format!("Erro na requisição: {}", e);
                log::error!("{}", msg);
                sender_safe!(package.sender, error_response!(msg));
            }
        }
    }

    Ok(())
}

fn file_part(path: &str) -> Result<Part, Box<dyn std::error::Error + Send + Sync>> {
    let data = std::fs::read(path)?;
    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    Ok(Part::bytes(data).file_name(filename))
}

fn build_multipart_form(
    body: &JsonValue,
) -> Result<Form, Box<dyn std::error::Error + Send + Sync>> {
    let mut form = Form::new();
    if let Some(obj) = body.as_object() {
        for (k, v) in obj.iter() {
            let is_file_key = matches!(k.as_str(), "file" | "image" | "mask");
            if is_file_key {
                if let Some(s) = v.as_str() {
                    let path = std::path::Path::new(s);
                    if path.exists() {
                        form = form.part(k.clone(), file_part(s)?);
                    } else {
                        form = form.text(k.clone(), s.to_string());
                    }
                } else {
                    form = form.text(k.clone(), v.to_string());
                }
            } else {
                if let Some(s) = v.as_str() {
                    form = form.text(k.clone(), s.to_string());
                } else {
                    // Serializa valores não-string como JSON
                    form = form.text(k.clone(), v.to_string());
                }
            }
        }
    }
    Ok(form)
}

fn serde_to_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::from(i)
            } else if let Some(u) = n.as_u64() {
                // `Value::from(u64)` pode não existir; converte para i64 se possível
                if u <= i64::MAX as u64 {
                    Value::from(u as i64)
                } else if let Some(f) = n.as_f64() {
                    Value::from(f)
                } else {
                    Value::from(u as i64)
                }
            } else if let Some(f) = n.as_f64() {
                Value::from(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::from(s.as_str()),
        serde_json::Value::Array(arr) => {
            let vec: Vec<Value> = arr.iter().map(serde_to_value).collect();
            Value::from(vec)
        }
        serde_json::Value::Object(map) => {
            let mut hm = std::collections::HashMap::<String, Value>::new();
            for (k, v2) in map.iter() {
                hm.insert(k.clone(), serde_to_value(v2));
            }
            Value::from(hm)
        }
    }
}
