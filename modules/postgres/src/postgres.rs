use deadpool_postgres::{Pool, Runtime};
use phlow_sdk::prelude::*;
use tokio_postgres::NoTls;

#[derive(Debug)]
pub enum PostgresConfigError {
    PoolError(deadpool_postgres::CreatePoolError),
    SslMode(String),
}

impl std::fmt::Display for PostgresConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostgresConfigError::PoolError(err) => write!(f, "Pool error: {}", err),
            PostgresConfigError::SslMode(mode) => write!(f, "Invalid SSL mode: {}", mode),
        }
    }
}

impl std::error::Error for PostgresConfigError {}

#[derive(Clone, Debug)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
    pub ssl_mode: String,
    pub multiple_query: bool,
    pub cache_query: bool,
}

impl PostgresConfig {
    pub fn create_pool(&self) -> Result<Pool, PostgresConfigError> {
        let mut cfg = deadpool_postgres::Config::new();

        cfg.host = Some(self.host.clone());
        cfg.port = Some(self.port);
        cfg.user = Some(self.user.clone());
        cfg.password = Some(self.password.clone());
        cfg.dbname = Some(self.dbname.clone());

        let ssl_mode = match self.ssl_mode.as_str() {
            "prefer" => deadpool_postgres::SslMode::Prefer,
            "require" => deadpool_postgres::SslMode::Require,
            "disable" => deadpool_postgres::SslMode::Disable,
            _ => return Err(PostgresConfigError::SslMode(self.ssl_mode.clone())),
        };

        cfg.ssl_mode = Some(ssl_mode);

        cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(PostgresConfigError::PoolError)
    }
}

impl TryFrom<Value> for PostgresConfig {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let host = value
            .get("host")
            .map(Value::to_string)
            .unwrap_or_else(|| "localhost".to_string());
        let port = value.get("port").and_then(Value::to_i64).unwrap_or(5432) as u16;
        let user = value
            .get("user")
            .map(Value::to_string)
            .unwrap_or_else(|| "postgres".to_string());
        let password = value
            .get("password")
            .map(Value::to_string)
            .unwrap_or_else(|| "postgres".to_string());
        let dbname = value
            .get("dbname")
            .map(Value::to_string)
            .unwrap_or_else(|| "postgres".to_string());

        let prepare_statements = *value
            .get("multiple_query")
            .and_then(Value::as_bool)
            .unwrap_or(&true);

        let cache_query = *value
            .get("cache_query")
            .and_then(Value::as_bool)
            .unwrap_or(&true);

        let ssl_mode = value
            .get("ssl_mode")
            .map(Value::to_string)
            .unwrap_or_else(|| "prefer".to_string());

        if ssl_mode != "prefer" && ssl_mode != "require" && ssl_mode != "disable" {
            return Err("Invalid SSL mode. Use 'disable', 'prefer' or 'require'.".to_string());
        }

        Ok(PostgresConfig {
            host,
            port,
            user,
            password,
            dbname,
            multiple_query: prepare_statements,
            cache_query,
            ssl_mode,
        })
    }
}
