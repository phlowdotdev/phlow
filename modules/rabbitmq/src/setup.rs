use sdk::prelude::*;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    UriRequired,
    QueueRequired,
    ModeRequired,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UriRequired => write!(f, "uri is required"),
            Self::QueueRequired => write!(f, "queue is required"),
            Self::ModeRequired => write!(f, "mode is required"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mode {
    Producer,
    Consumer,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub uri: String,
    pub queue: String,
    pub mode: Mode,
    pub exchange: String,
    pub consumer_tag: String,
    pub declare: bool,
}

impl Config {
    pub fn is_producer(&self) -> bool {
        matches!(self.mode, Mode::Producer)
    }

    pub fn is_consumer(&self) -> bool {
        matches!(self.mode, Mode::Consumer)
    }
}

impl TryFrom<&Value> for Config {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Error> {
        let uri = value.get("uri").ok_or(Error::UriRequired)?.to_string();

        let routing_key = value
            .get("routing_key")
            .ok_or(Error::QueueRequired)?
            .to_string();

        let mode = value
            .get("mode")
            .and_then(|mode| match mode.to_string().as_str() {
                "producer" => Some(Mode::Producer),
                "consumer" => Some(Mode::Consumer),
                _ => None,
            })
            .ok_or(Error::ModeRequired)?;

        let exchange = value
            .get("exchange")
            .map_or("".to_string(), |v| v.to_string());

        let consumer_tag = value
            .get("consumer_tag")
            .map_or("".to_string(), |v| v.to_string());

        let declare = value
            .get("declare")
            .map_or(false, |v| v.as_bool().unwrap_or(&false).clone());

        Ok(Self {
            uri,
            queue: routing_key,
            exchange,
            mode,
            consumer_tag,
            declare,
        })
    }
}
