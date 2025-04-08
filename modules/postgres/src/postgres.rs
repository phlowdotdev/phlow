use deadpool_postgres::{Config as PoolConfig, ManagerConfig, Pool, RecyclingMethod, Runtime};
use phlow_sdk::{prelude::*, tokio};
use tokio_postgres::{Client, Error, NoTls};

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

impl Config {
    pub async fn connect(&self) -> Result<Client, Error> {
        let conn_str = self.to_conn_string();
        let (client, connection) = tokio_postgres::connect(&conn_str, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Erro na conexão com o banco: {}", e);
            }
        });

        Ok(client)
    }

    pub fn to_conn_string(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.host, self.port, self.user, self.password, self.dbname
        )
    }

    pub fn create_pool(&self) -> Result<Pool, Box<dyn std::error::Error + Send + Sync>> {
        let mut cfg = PoolConfig::new();

        cfg.host = Some(self.host.clone());
        cfg.port = Some(self.port);
        cfg.user = Some(self.user.clone());
        cfg.password = Some(self.password.clone());
        cfg.dbname = Some(self.dbname.clone());

        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
        Ok(pool)
    }
}

impl TryFrom<Value> for Config {
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

        Ok(Config {
            host,
            port,
            user,
            password,
            dbname,
        })
    }
}
