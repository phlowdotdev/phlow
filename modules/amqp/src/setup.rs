use sdk::prelude::*;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    RoutingKey,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoutingKey => write!(f, "routing_key is required"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub routing_key: String,
    pub username: String,
    pub password: String,
    pub port: u16,
    pub exchange: String,
    pub consumer_tag: String,
    pub declare: bool,
}

impl Config {
    pub fn to_connection_string(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}",
            self.username, self.password, self.host, &self.port,
        )
    }
}

impl TryFrom<&Value> for Config {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Error> {
        let username = value
            .get("username")
            .map(|v| v.to_string())
            .unwrap_or("guest".to_string());

        let password = value
            .get("password")
            .map(|v| v.to_string())
            .unwrap_or("guest".to_string());

        let port = value.get("port").map(|v| v.to_i64().unwrap_or(5672) as u16);

        let host = value
            .get("host")
            .map(|v| v.to_string())
            .unwrap_or("localhost".to_string());

        let routing_key = value
            .get("routing_key")
            .map(|v| v.to_string())
            .ok_or(Error::RoutingKey)?;

        let exchange = value
            .get("exchange")
            .map(|v| v.to_string())
            .unwrap_or("".to_string());

        let consumer_tag = value
            .get("consumer_tag")
            .map(|v| v.to_string())
            .unwrap_or("".to_string());

        let declare = *value
            .get("declare")
            .map(|v| v.as_bool().unwrap_or(&false))
            .unwrap_or(&false);

        Ok(Self {
            host,
            username,
            password,
            port: port.unwrap_or(5672),
            routing_key,
            exchange,
            consumer_tag,
            declare,
        })
    }
}
