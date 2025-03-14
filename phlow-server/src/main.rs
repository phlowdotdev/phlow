mod anymain;
use anymain::Main;
use libloading::{Library, Symbol};
use phlow::{build_engine_async, Context, Phlow};
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
                "return": {
                    "status_code": 201,
                    "body": "main",
                    "headers": {
                        "Content-Type": r#""application/json""#
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

    let steps: Value = main.get_steps();
    let engine = build_engine_async(None);

    let phlow = Phlow::try_from_value(&engine, &steps, None, None).unwrap();

    let (sender, receiver) = std::sync::mpsc::channel::<Package>();

    tokio::task::spawn(async move {
        unsafe {
            println!("Loading module: {}", main.module);
            let lib = Library::new(format!("phlow_modules/{}.so", main.module).as_str()).unwrap();
            let func: Symbol<unsafe extern "C" fn(Broker, Value)> = lib.get(b"plugin").unwrap();

            func(sender, main.with);
        }
    });

    println!("Server started");

    for mut package in receiver {
        if let Some(data) = package.get_data() {
            let mut context = Context::from_main(data.clone());
            let result = phlow.execute_with_context(&mut context).unwrap();

            package.send(result.unwrap_or(Value::Null));
        }
    }
}
