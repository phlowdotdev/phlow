mod anymain;
use anymain::Main;
use libloading::{Library, Symbol};
use sdk::prelude::*;
use valu3::json;

#[tokio::main]
async fn main() {
    let config = json!({
        "main" : {
            "module": "restapi"
        },
        "steps": [
            {
                "output": {
                    "status_code": 200,
                    "body": {
                        "message": "Hello, World!"
                    }
                }
            }
        ]
    });

    let main = match Main::try_from(config) {
        Ok(main) => main,
        Err(err) => {
            println!("Error: {:?}", err);
            return;
        }
    };

    let (sender, receiver) = std::sync::mpsc::channel::<Package>();

    tokio::task::spawn(async move {
        unsafe {
            let lib = Library::new(format!("anyflow_modules/{}.so", main.module).as_str()).unwrap();
            let func: Symbol<unsafe extern "C" fn(Broker, Value)> = lib.get(b"plugin").unwrap();

            func(sender, main.with);
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
