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
            .and_then(|v| {
                if let Value::Object(obj) = v {
                    let mut result = Vec::new();
                    for (_, v) in obj.iter() {
                        let param: Result<Box<dyn ToSql + Sync + Send>, String> = if let Some(n) = v.to_i64() {
                            if n >= i32::MIN as i64 && n <= i32::MAX as i64 {
                                Ok(Box::new(n as i32) as Box<dyn ToSql + Sync + Send>)
                            } else {
                                Ok(Box::new(n) as Box<dyn ToSql + Sync + Send>)
                            }
                        } else if let Some(f) = v.to_f64() {
                            Ok(Box::new(f) as Box<dyn ToSql + Sync + Send>)
                        } else if let Some(b) = v.as_bool() {
                            Ok(Box::new(*b) as Box<dyn ToSql + Sync + Send>)
                        } else {
                            Ok(Box::new(v.to_string()) as Box<dyn ToSql + Sync + Send>)
                        };
                        match param {
                            Ok(p) => result.push(p),
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    Some(Ok(result))
                } else {
                    None
                }
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
