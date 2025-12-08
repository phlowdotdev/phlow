mod input;
mod postgres;
mod response;
use std::sync::Arc;

use input::Input;
use phlow_sdk::prelude::*;
use postgres::PostgresConfig;
use response::QueryResult;
use std::error::Error;
use tokio_postgres::types::ToSql;

create_step!(postgres(setup));

pub async fn postgres(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);
    let config = PostgresConfig::try_from(setup.with.clone())?;
    let pool = Arc::new(config.create_pool()?);
    println!("Postgres module initialized with config: {:?}", config);
    let mut handles = Vec::new();

    for package in rx {
        let pool = pool.clone();
        let config = config.clone();

        let handle = tokio::spawn(async move {
            let input = match Input::try_from((package.input, &config)) {
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

            if input.batch {
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
                        let response: ModuleResponse = ModuleResponse::from_success(json!({
                            "success": true,
                            "data": result.to_value()
                        }));

                        sender_safe!(package.sender, response.into());
                    }
                    Err(e) => {
                        let code = e.code().map(|c| c.code()).unwrap_or("UNKNOWN");
                        let message = e
                            .as_db_error()
                            .map(|db| db.message().to_string())
                            .unwrap_or_else(|| e.to_string());
                        let cause = e.source().map(|s| s.to_string()).unwrap_or_default();

                        let response = ModuleResponse::from_success(json!({
                            "success": false,
                            "error": {
                                "code": code,
                                "cause": cause,
                                "message": message,
                            }
                        }));

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

        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.await {
            log::error!("Error in task: {:?}", e);
        }
    }

    Ok(())
}
