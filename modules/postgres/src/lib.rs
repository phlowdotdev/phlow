mod input;
mod postgres;
mod response;
use std::sync::Arc;

use input::Input;
use phlow_sdk::prelude::*;
use postgres::PostgresConfig;
use response::{QueryResult, Response};
use tokio_postgres::types::ToSql;

create_step!(postgres(setup));

pub async fn postgres(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);
    let config = PostgresConfig::try_from(setup.with.clone())?;
    let pool = Arc::new(config.create_pool()?);

    for package in rx {
        let pool = pool.clone();

        tokio::spawn(async move {
            let input = match Input::try_from(package.context.input) {
                Ok(input) => input,
                Err(e) => {
                    let response = Response {
                        status: "error".to_string(),
                        message: format!("Failed to parse input: {}", e),
                        data: Value::Undefined,
                    }
                    .to_value();

                    sender_safe!(package.sender, response);
                    return;
                }
            };

            let client = match pool.get().await {
                Ok(client) => client,
                Err(e) => {
                    let response = Response {
                        status: "error".to_string(),
                        message: format!("Failed to get client from pool: {}", e),
                        data: Value::Undefined,
                    }
                    .to_value();

                    sender_safe!(package.sender, response);
                    return;
                }
            };

            let stmt = match client.prepare_cached(input.query.as_str()).await {
                Ok(stmt) => stmt,
                Err(e) => {
                    let response = Response {
                        status: "error".to_string(),
                        message: format!("Failed to prepare statement: {}", e),
                        data: Value::Undefined,
                    }
                    .to_value();

                    sender_safe!(package.sender, response);
                    return;
                }
            };

            let param_refs: Vec<&(dyn ToSql + Sync)> = input
                .params
                .iter()
                .map(|p| p.as_ref() as &(dyn ToSql + Sync))
                .collect();

            match client.query(&stmt, &param_refs[..]).await {
                Ok(rows) => {
                    let result = QueryResult::from(rows);
                    let response = Response {
                        status: "success".to_string(),
                        message: "Query executed successfully".to_string(),
                        data: result.to_value(),
                    }
                    .to_value();

                    sender_safe!(package.sender, response);
                }
                Err(e) => {
                    let response = Response {
                        status: "error".to_string(),
                        message: format!("Query execution failed: {}", e),
                        data: Value::Undefined,
                    }
                    .to_value();

                    sender_safe!(package.sender, response);
                    return;
                }
            };
        });
    }

    Ok(())
}
