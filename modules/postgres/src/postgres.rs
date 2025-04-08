use deadpool_postgres::{Manager, Pool, Runtime, Timeouts};
use phlow_sdk::{prelude::*, tokio};
use tokio_postgres::{config::SslMode, Client, Error, NoTls};

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
        let pg_config = tokio_postgres::Config::new()
            .host(&self.host)
            .port(self.port)
            .user(&self.user)
            .password(&self.password)
            .dbname(&self.dbname)
            .ssl_mode(SslMode::Prefer)
            .to_owned();
        let tls = NoTls;

        let mgr = Manager::new(pg_config, tls);

        let pool = Pool::builder(mgr)
            .max_size(16)
            .timeouts(Timeouts {
                wait: Some(std::time::Duration::from_secs(30)),
                create: Some(std::time::Duration::from_secs(30)),
                recycle: Some(std::time::Duration::from_secs(30)),
            })
            .runtime(Runtime::Tokio1)
            .build()?;
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
