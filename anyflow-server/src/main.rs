use libloading::{Library, Symbol};
use sdk::prelude::*;
use valu3::json;

#[tokio::main]
async fn main() {
    let (sender, receiver) = std::sync::mpsc::channel::<Package>();

    tokio::task::spawn(async move {
        unsafe {
            let lib = Library::new("target/release/librestapi.so")
                .expect("Falha ao carregar a biblioteca");
            let func: Symbol<unsafe extern "C" fn(Broker, Value)> = lib.get(b"plugin").unwrap();

            func(sender, Value::Null);
        }
    });

    println!("Server started");

    for mut package in receiver {
        if let Some(data) = &package.get_data() {
            package.send(json!({
                "status": "ok",
                "data": data
            }));
        }
    }
}
