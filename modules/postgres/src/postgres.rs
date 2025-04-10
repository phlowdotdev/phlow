use deadpool_postgres::{Pool, Runtime};
use phlow_sdk::prelude::*;
use tokio_postgres::NoTls;

#[derive(Clone, Debug)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
    pub prepare_statements: bool,
}

impl PostgresConfig {
    pub fn create_pool(&self) -> Result<Pool, deadpool_postgres::CreatePoolError> {
        let mut cfg = deadpool_postgres::Config::new();

        cfg.host = Some(self.host.clone());
        cfg.port = Some(self.port);
        cfg.user = Some(self.user.clone());
        cfg.password = Some(self.password.clone());
        cfg.dbname = Some(self.dbname.clone());
        cfg.ssl_mode = Some(deadpool_postgres::SslMode::Prefer);

        cfg.create_pool(Some(Runtime::Tokio1), NoTls)
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
            .get("prepare_statements")
            .and_then(Value::as_bool)
            .unwrap_or(&true);

        Ok(PostgresConfig {
            host,
            port,
            user,
            password,
            dbname,
            prepare_statements,
        })
    }
}
