use libloading::{Library, Symbol};
use sdk::prelude::*;
use tokio::runtime::Runtime;
use valu3::json;

fn main() {
    let config = json!({
        "route": {
            "path": "/",
            "method": "GET"
        }
    });

    let value = config.to_value();

    unsafe {
        let lib = Library::new("target/release/libhttp.so").expect("Falha ao carregar o plugin");

        let func: Symbol<unsafe extern "C" fn(*const Value)> = lib.get(b"plugin").unwrap();

        func(&value);
    }

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}
