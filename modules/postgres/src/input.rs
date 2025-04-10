use phlow_sdk::prelude::*;
use tokio_postgres::types::ToSql;

use crate::postgres::PostgresConfig;

#[derive(Debug)]
pub struct Input {
    pub query: String,
    pub params: Vec<Box<dyn ToSql + Sync + Send>>,
    pub batch: bool,
    pub cache_query: bool,
}

impl TryFrom<(Option<Value>, &PostgresConfig)> for Input {
    type Error = String;

    fn try_from((value, config): (Option<Value>, &PostgresConfig)) -> Result<Self, Self::Error> {
        let value = value.ok_or_else(|| "Input value is None".to_string())?;

        let query = value
            .get("query")
            .clone()
            .and_then(|v| Some(v.to_string()))
            .ok_or_else(|| "Query not found or not a string".to_string())?
            .to_string();

        let params = value
            .get("params")
            .clone()
            .and_then(|v| v.as_array().cloned()) // <- clona os valores do array
            .map(|arr| {
                arr.into_iter()
                    .map(|v| {
                        if let Some(n) = v.to_i64() {
                            if n >= i32::MIN as i64 && n <= i32::MAX as i64 {
                                Ok::<_, String>(Box::new(n as i32) as Box<dyn ToSql + Sync + Send>)
                            } else {
                                Ok::<_, String>(Box::new(n) as Box<dyn ToSql + Sync + Send>)
                            }
                        } else if let Some(f) = v.to_f64() {
                            Ok::<_, String>(Box::new(f) as Box<dyn ToSql + Sync + Send>)
                        } else if let Some(b) = v.as_bool() {
                            Ok::<_, String>(Box::new(b.clone()) as Box<dyn ToSql + Sync + Send>)
                        } else {
                            Ok::<_, String>(Box::new(v.to_string()) as Box<dyn ToSql + Sync + Send>)
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_else(|| Ok(vec![]))?;

        let prepare_statements = *value
            .get("batch")
            .and_then(Value::as_bool)
            .unwrap_or(&config.batch);

        let cache_query = *value
            .get("cache_query")
            .and_then(Value::as_bool)
            .unwrap_or(&config.cache_query);

        Ok(Input {
            query,
            params,
            batch: prepare_statements,
            cache_query,
        })
    }
}
