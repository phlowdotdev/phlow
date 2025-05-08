use phlow_sdk::{prelude::*, valu3::types::object};
use std::{collections::HashMap, env, fmt::Display, net::SocketAddr};

pub const DEFAULT_PORT: u16 = 31451;
pub const DEFAULT_SERVER_ADDR: &str = "localhost";

#[derive(Clone, Debug)]
pub struct Server {
    pub port: u16,
    pub server_addr: String,
}

impl Server {
    pub fn get_address(&self) -> SocketAddr {
        format!("{}:{}", self.server_addr, self.port)
            .parse()
            .unwrap_or_else(|_| {
                panic!("Invalid server address: {}:{}", self.server_addr, self.port)
            })
    }
}

impl Default for Server {
    fn default() -> Self {
        Server {
            port: DEFAULT_PORT,
            server_addr: DEFAULT_SERVER_ADDR.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub main_server: Server,
    pub target_servers: HashMap<String, Server>,
    pub parallel_executions: u32,
}

impl From<Value> for Config {
    fn from(value: Value) -> Self {
        let package_consumer_count = env::var("PHLOW_PACKAGE_CONSUMERS_COUNT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(10) as u32;

        if value.is_null() {
            return Config {
                main_server: Server::default(),
                target_servers: HashMap::new(),
                parallel_executions: package_consumer_count,
            };
        }

        let main_server = {
            let port = match value.get("port") {
                Some(port) => port.to_u64().unwrap_or(3000) as u16,
                None => DEFAULT_PORT,
            };

            let server_addr = match value.get("server_addr") {
                Some(addr) => addr.to_string(),
                None => DEFAULT_SERVER_ADDR.to_string(),
            };

            Server { port, server_addr }
        };

        let mut target_servers = HashMap::new();

        if let Some(servers_value) = value.get("servers") {
            let object = servers_value.as_object().unwrap();
            for (key, value) in object.iter() {
                let value_split = value.as_str().split(':').collect::<Vec<_>>();

                let port = value_split[1].parse::<u16>().unwrap_or(DEFAULT_PORT);
                let server_addr = value_split[0].to_string();

                target_servers.insert(key.to_string(), Server { port, server_addr });
            }
        }

        let parallel_executions = match value.get("parallel_executions") {
            Some(port) => port.to_u64().unwrap_or(10) as u32,
            None => package_consumer_count,
        };

        Config {
            main_server,
            target_servers,
            parallel_executions,
        }
    }
}

pub struct StepInput {
    pub server: String,
    pub params: Value,
}

pub enum Error {
    InvalidInput(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl TryFrom<Value> for StepInput {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let server = match value.get("server") {
            Some(server) => server.to_string(),
            None => {
                return Err(Error::InvalidInput(
                    "Server address is required".to_string(),
                ))
            }
        };

        let params = value.get("params").cloned().unwrap_or(Value::Null);

        Ok(StepInput { server, params })
    }
}
