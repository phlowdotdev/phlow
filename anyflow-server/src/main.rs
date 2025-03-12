use libloading::{Library, Symbol};
use sdk::prelude::*;
use std::collections::HashMap;

extern "C" fn callback(input: *const Value) -> *const Value {
    unsafe {
        if input.is_null() {
            return std::ptr::null();
        }

        let input_ref = &*input;
        println!("Callback chamado com: {:?}", input_ref);

        Box::into_raw(Box::new(Value::Null))
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
