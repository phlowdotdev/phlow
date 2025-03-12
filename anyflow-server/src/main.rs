use libloading::{Library, Symbol};
use sdk::prelude::*;
use std::collections::HashMap;

// Função de callback que será passada para o plugin
extern "C" fn callback(input: *const Value) -> *const Value {
    unsafe {
        if input.is_null() {
            return std::ptr::null();
        }

        let input_ref = &*input;
        println!("Callback chamado com: {:?}", input_ref);

        let response = Value::from("Resposta do callback");
        let boxed_response = Box::new(response);

        Box::into_raw(boxed_response)
    }
}

fn main() {
    let mut data = HashMap::new();
    data.insert("name", Value::from("Anyflow"));
    data.insert("version", Value::from("0.1.0"));

    let value = data.to_value();

    unsafe {
        let lib =
            Library::new("target/release/libhttp.so").expect("Falha ao carregar a biblioteca");

        let func: Symbol<unsafe extern "C" fn(*const Value, CallbackFn)> =
            lib.get(b"process_data").unwrap();

        func(&value, callback);
    }
}
