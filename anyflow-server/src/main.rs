use libloading::{Library, Symbol};
use sdk::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

// Função de callback chamada pelo plugin
extern "C" fn my_callback(input: *const Value) -> *const Value {
    unsafe {
        if input.is_null() {
            return std::ptr::null();
        }

        let input_ref = &*input;
        println!("🔔 Callback chamado com: {:?}", input_ref);

        let response = Value::from("🎉 Resposta do callback!");
        let boxed_response = Box::new(response);
        Box::into_raw(boxed_response)
    }
}

fn main() {
    let mut config = HashMap::new();
    config.insert("server_port", Value::from(3000));

    let value = config.to_value();

    unsafe {
        let lib = Library::new("target/release/libhttp.so").expect("Falha ao carregar o plugin");

        let func: Symbol<unsafe extern "C" fn(*const Value, CallbackFn)> =
            lib.get(b"process_data").unwrap();

        func(&value, my_callback);
    }

    // Mantém o programa rodando para o servidor funcionar
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}
