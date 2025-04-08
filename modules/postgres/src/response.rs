use std::collections::HashMap;

use phlow_sdk::prelude::*;
use tokio_postgres::Row;

#[derive(Debug, Clone, ToValue)]
pub struct QueryResult {
    pub rows: Vec<HashMap<String, Value>>,
    pub count: usize,
}

impl From<Vec<Row>> for QueryResult {
    fn from(rows: Vec<Row>) -> Self {
        let mut result = QueryResult {
            rows: Vec::new(),
            count: rows.len(),
        };

        for row in rows {
            let mut map = HashMap::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value = row.get::<_, String>(i);

                match Value::json_to_value(&value) {
                    Ok(v) => map.insert(column.name().to_string(), v),
                    Err(_) => map.insert(column.name().to_string(), Value::Undefined),
                };
            }
            result.rows.push(map);
        }

        result
    }
}

#[derive(Debug, Clone, ToValue)]
pub struct Response {
    pub status: String,
    pub message: String,
    pub data: Value,
}
