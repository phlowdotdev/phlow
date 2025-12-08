use deadpool_postgres::{Pool, Runtime};
use phlow_sdk::prelude::*;
use rustls::ClientConfig;
use rustls::client::danger::{ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use std::sync::Arc;
use tokio_postgres::NoTls;
use tokio_postgres_rustls::MakeRustlsConnect;

#[derive(Debug)]
struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

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
    pub database: String,
    pub ssl_mode: String,
    pub batch: bool,
    pub cache_query: bool,
    pub max_size: usize,
}

impl PostgresConfig {
    pub fn create_pool(&self) -> Result<Pool, PostgresConfigError> {
        let mut cfg = deadpool_postgres::Config::new();

        cfg.host = Some(self.host.clone());
        cfg.port = Some(self.port);
        cfg.user = Some(self.user.clone());
        cfg.password = Some(self.password.clone());
        cfg.dbname = Some(self.database.clone());
        cfg.pool = Some(deadpool_postgres::PoolConfig {
            max_size: self.max_size,
            ..Default::default()
        });

        let ssl_mode = match self.ssl_mode.as_str() {
            "prefer" => deadpool_postgres::SslMode::Prefer,
            "require" => deadpool_postgres::SslMode::Require,
            "disable" => deadpool_postgres::SslMode::Disable,
            _ => return Err(PostgresConfigError::SslMode(self.ssl_mode.clone())),
        };

        cfg.ssl_mode = Some(ssl_mode);

        // Use TLS when ssl_mode is not "disable"
        if self.ssl_mode == "disable" {
            cfg.create_pool(Some(Runtime::Tokio1), NoTls)
                .map_err(PostgresConfigError::PoolError)
        } else {
            // Create rustls config that accepts any certificate
            // This is necessary for some cloud providers like DigitalOcean
            let tls_config = ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoVerifier))
                .with_no_client_auth();

            let tls = MakeRustlsConnect::new(tls_config);

            cfg.create_pool(Some(Runtime::Tokio1), tls)
                .map_err(PostgresConfigError::PoolError)
        }
    }
}

impl TryFrom<Value> for PostgresConfig {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let host = value
            .get("host")
            .map(Value::to_string)
            .unwrap_or_else(|| "localhost".to_string());
        let port = match value.get("port") {
            Some(port) => match port.to_string() {
                s if s.is_empty() => 5432,
                s => s.parse().unwrap_or(5432),
            },
            None => 5432,
        };
        let user = value
            .get("user")
            .map(Value::to_string)
            .unwrap_or_else(|| "postgres".to_string());
        let password = value
            .get("password")
            .map(Value::to_string)
            .unwrap_or_else(|| "postgres".to_string());
        let database = value
            .get("database")
            .map(Value::to_string)
            .unwrap_or_else(|| "postgres".to_string());

        let prepare_statements = *value.get("batch").and_then(Value::as_bool).unwrap_or(&true);

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

        let max_size = value
            .get("max_pool_size")
            .and_then(Value::to_i64)
            .unwrap_or(10) as usize;

        Ok(PostgresConfig {
            host,
            port,
            user,
            password,
            database,
            batch: prepare_statements,
            cache_query,
            ssl_mode,
            max_size,
        })
    }
}
