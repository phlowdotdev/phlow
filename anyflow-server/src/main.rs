use std::collections::HashMap;

use libloading::{Library, Symbol};
use valu3::prelude::*;

fn main() {
    let mut data = HashMap::new();
    data.insert("name", Value::from("Anyflow"));
    data.insert("version", Value::from("0.1.0"));

    let value = data.to_value();

    unsafe {
        let lib = Library::new("target/release/libhttp.so") // Windows
            .expect("Falha ao carregar a biblioteca");

        let func: Symbol<unsafe extern "C" fn(*const Value)> = lib.get(b"process_data").unwrap();

        func(&value);
    }
}
