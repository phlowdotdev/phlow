use std::collections::HashMap;

use phlow_sdk::prelude::*;
use tokio_postgres::{types::Type, Row};

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
            // ...existing code...
            for (i, column) in row.columns().iter().enumerate() {
                let name = column.name();

                let value = match column.type_() {
                    &Type::INT2 => row.try_get::<_, i16>(i).map_or(Value::Null, Value::from),
                    &Type::INT4 => row.try_get::<_, i32>(i).map_or(Value::Null, Value::from),
                    &Type::INT8 => row.try_get::<_, i64>(i).map_or(Value::Null, Value::from),
                    &Type::FLOAT4 => row.try_get::<_, f32>(i).map_or(Value::Null, Value::from),
                    &Type::FLOAT8 => row.try_get::<_, f64>(i).map_or(Value::Null, Value::from),
                    &Type::NUMERIC => row.try_get::<_, f64>(i).map_or(Value::Null, Value::from),
                    &Type::BOOL => row.try_get::<_, bool>(i).map_or(Value::Null, Value::from),
                    &Type::DATE => row
                        .try_get::<_, chrono::NaiveDate>(i)
                        .map_or(Value::Null, |v| Value::from(v.to_string())),
                    &Type::TIMESTAMP | &Type::TIMESTAMPTZ => row
                        .try_get::<_, chrono::NaiveDateTime>(i)
                        .map_or(Value::Null, |v| Value::from(v.to_string())),
                    _ => row.try_get::<_, String>(i).map_or(Value::Null, Value::from),
                };

                map.insert(name.to_string(), value.to_value());
            }
            // ...existing code...
            // ...existing code...
            result.rows.push(map);
        }

        result
    }
}
