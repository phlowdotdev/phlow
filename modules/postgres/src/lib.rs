mod input;
mod postgres;
mod response;
use std::sync::Arc;

use input::Input;
use phlow_sdk::prelude::*;
use postgres::PostgresConfig;
use response::QueryResult;
use tokio_postgres::types::ToSql;

create_step!(postgres(setup));

pub async fn postgres(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);
    let config = PostgresConfig::try_from(setup.with.clone())?;
    let pool = Arc::new(config.create_pool()?);

    for package in rx {
        let pool = pool.clone();
        let config = config.clone();

        tokio::spawn(async move {
            let input = match Input::try_from((package.context.input, &config)) {
                Ok(input) => input,
                Err(e) => {
                    let response =
                        ModuleResponse::from_error(format!("Failed to parse input: {}", e));

                    sender_safe!(package.sender, response.into());
                    return;
                }
            };

            let client = match pool.get().await {
                Ok(client) => client,
                Err(e) => {
                    let response = ModuleResponse::from_error(format!(
                        "Failed to get client from pool: {}",
                        e
                    ));

                    sender_safe!(package.sender, response.into());
                    return;
                }
            };

            if input.multiple_query {
                let stmt = if input.cache_query {
                    match client.prepare_cached(input.query.as_str()).await {
                        Ok(stmt) => stmt,
                        Err(e) => {
                            let response = ModuleResponse::from_error(format!(
                                "Failed to prepare statement: {}",
                                e
                            ));

                            sender_safe!(package.sender, response.into());
                            return;
                        }
                    }
                } else {
                    match client.prepare(input.query.as_str()).await {
                        Ok(stmt) => stmt,
                        Err(e) => {
                            let response = ModuleResponse::from_error(format!(
                                "Failed to prepare statement: {}",
                                e
                            ));

                            sender_safe!(package.sender, response.into());
                            return;
                        }
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

                        sender_safe!(package.sender, result.to_value().into());
                    }
                    Err(e) => {
                        let response =
                            ModuleResponse::from_error(format!("Query execution failed: {}", e));

                        sender_safe!(package.sender, response.into());
                        return;
                    }
                };
            } else {
                match client.batch_execute(&input.query).await {
                    Ok(_) => {
                        let response = "OK".to_value().into();
                        sender_safe!(package.sender, response);
                    }
                    Err(e) => {
                        let response =
                            ModuleResponse::from_error(format!("Batch execution failed: {}", e));
                        sender_safe!(package.sender, response.into());
                    }
                }
            }
        });
    }

    Ok(())
}
