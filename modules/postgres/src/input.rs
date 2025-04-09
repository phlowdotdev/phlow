use phlow_sdk::prelude::*;
use tokio_postgres::types::ToSql;

// input.rs
pub struct Input {
    pub query: String,
    pub params: Vec<Box<dyn ToSql + Sync + Send>>, // <- adicione o `Send` aqui
}

impl TryFrom<Option<Value>> for Input {
    type Error = String;

    fn try_from(value: Option<Value>) -> Result<Self, Self::Error> {
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
                            Ok::<_, String>(Box::new(n) as Box<dyn ToSql + Sync + Send>)
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

        Ok(Input { query, params })
    }
}
