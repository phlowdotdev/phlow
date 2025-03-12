use libloading::{Library, Symbol};
use sdk::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

fn main() {
    let mut config = HashMap::new();
    config.insert("server_port", Value::from(3000));

    let value = config.to_value();

    unsafe {
        let lib = Library::new("target/release/libhttp.so").expect("Falha ao carregar o plugin");

        let func: Symbol<unsafe extern "C" fn(*const Value)> = lib.get(b"process_data").unwrap();

        func(&value);
    }

    // Mantém o programa rodando para o servidor funcionar
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}
