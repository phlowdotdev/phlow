mod postgres;
mod response;
use phlow_sdk::prelude::*;
use postgres::Config;
use response::{QueryResult, Response};

create_step!(postgres(setup));

pub async fn postgres(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);
    let config = Config::try_from(setup.with.clone())?;
    let pool = config.create_pool()?;

    listen!(rx, move |package: ModulePackage| async {
        match pool.get().await {
            Ok(client) => {
                let query = match package.context.input {
                    Some(input) => match input.get("query") {
                        Some(query) => {
                            if query.is_string() {
                                query.to_string()
                            } else {
                                let response = Response {
                                    status: "error".to_string(),
                                    message: "Invalid query format".to_string(),
                                    data: Value::Undefined,
                                }
                                .to_value();

                                sender_safe!(package.sender, response);
                            }
                        }
                        None => {
                            let response = Response {
                                status: "error".to_string(),
                                message: "No query provided".to_string(),
                                data: Value::Undefined,
                            }
                            .to_value();

                            sender_safe!(package.sender, response);
                        }
                    },
                    None => {
                        let response = Response {
                            status: "error".to_string(),
                            message: "No input provided".to_string(),
                            data: Value::Undefined,
                        }
                        .to_value();

                        sender_safe!(package.sender, response);
                        return;
                    }
                };

                match client.query(query.as_str(), &[]).await {
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
            }
            Err(e) => {
                let response = Response {
                    status: "error".to_string(),
                    message: format!("Failed to connect to the database: {}", e),
                    data: Value::Undefined,
                }
                .to_value();

                sender_safe!(package.sender, response);
            }
        }
    });

    Ok(())
}
