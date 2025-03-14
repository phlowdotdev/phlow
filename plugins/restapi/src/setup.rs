use valu3::prelude::*;

#[derive(Clone, Debug)]
pub struct Setup {
    pub port: Option<u16>,
    pub host: Option<String>,
}

impl From<Value> for Setup {
    fn from(value: Value) -> Self {
        let port = match value.get("port") {
            Some(port) => Some(port.to_u64().unwrap() as u16),
            None => None,
        };

        let host = match value.get("host") {
            Some(host) => Some(host.as_string()),
            None => None,
        };

        Setup { port, host }
    }
}
