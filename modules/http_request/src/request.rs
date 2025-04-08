use crate::input::Input;
use phlow_sdk::{timer::Timer, valu3::prelude::*};
use reqwest::{header, Client, Method};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    RequestError(reqwest::Error),
    HeaderError(header::InvalidHeaderName),
    HeaderValueError(header::InvalidHeaderValue),
    ValueError(phlow_sdk::valu3::Error),
}

impl From<Error> for Value {
    fn from(error: Error) -> Self {
        match error {
            Error::RequestError(e) => format!("Request error: {}", e).to_value(),
            Error::HeaderError(e) => format!("Header error: {}", e).to_value(),
            Error::HeaderValueError(e) => format!("Header value error: {}", e).to_value(),
            Error::ValueError(e) => format!(
                "Value error: {}",
                match e {
                    phlow_sdk::valu3::Error::NonParsebleMsg(msg) => msg,
                    phlow_sdk::valu3::Error::NonParseble => "Non parseable".to_string(),
                    phlow_sdk::valu3::Error::NotNumber => "Not a number".to_string(),
                }
            )
            .to_value(),
        }
    }
}

pub async fn request(input: Input, client: Client) -> Result<Value, Error> {
    let request_builder = match input.method {
        Method::GET => client.get(&input.url),
        Method::POST => client.post(&input.url),
        Method::PUT => client.put(&input.url),
        Method::PATCH => client.patch(&input.url),
        Method::DELETE => client.delete(&input.url),
        _ => client.get(&input.url),
    }
    .headers(input.headers)
    .body(input.body.unwrap_or_default());

    let response = request_builder.send().await.map_err(Error::RequestError)?;

    let status_code = response.status().as_u16();

    let mut headers_map: HashMap<String, String> = HashMap::new();

    for (key, value) in response.headers().iter() {
        headers_map.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
    }

    let body = response.text().await.map_err(Error::RequestError)?;
    let body_value = Value::json_to_value(&body).map_err(Error::ValueError)?;

    let response = HashMap::from([
        ("headers", headers_map.to_value()),
        ("body", body_value),
        ("status_code", status_code.to_value()),
    ])
    .to_value();

    Ok(response)
}
