mod loader;
use libloading::{Library, Symbol};
use loader::Loader;
use phlow_rule_engine::{build_engine_async, Context, Phlow};
use sdk::prelude::*;
use valu3::json;

#[tokio::main]
async fn main() {
    let config = json!({
        "main" : "restapi",
        "modules":[
            {
                "name": "restapi"
            },
            {
                "name": "rabbitmq",
                "with": {
                    "type": "producer",
                    "host": "localhost",
                    "port": 5672,
                    "username": "guest",
                    "password": "guest",
                    "exchange": "phlow",
                    "routing_key": "phlow"
                }
            }
        ],
        "steps": [
            {
                "module": "rabbitmq",
                "params": { "message": "main.body"}
            },
            {
                "return": {
                    "status_code": 201,
                    "body": "main.body",
                    "headers": {
                        "Content-Type": r#""application/json""#
                    }
                }
            }
        ]
    });

    let loader = match Loader::try_from(config) {
        Ok(main) => main,
        Err(err) => {
            println!("Error: {:?}", err);
            return;
        }
    };

    let steps: Value = loader.get_steps();
    let engine = build_engine_async(None);

    let phlow = Phlow::try_from_value(&engine, &steps, None, None).unwrap();

    let (sender, receiver) = std::sync::mpsc::channel::<Package>();

    {
        for module in loader.modules.iter() {
            let path = format!("phlow_modules/{}.so", module.name);

            if !std::path::Path::new(&path).exists() {
                println!("Error: Module {} does not exist", module.name);
                return;
            }
        }
    }

    for (id, module) in loader.modules.into_iter().enumerate() {
        let sender = sender.clone();

        tokio::task::spawn(async move {
            unsafe {
                println!("Loading module: {}", module.name);
                let lib = match Library::new(format!("phlow_modules/{}.so", module.name).as_str()) {
                    Ok(lib) => lib,
                    Err(err) => {
                        println!("Error: {:?}", err);
                        return;
                    }
                };
                let func: Symbol<unsafe extern "C" fn(ModuleId, RuntimeSender, Value)> =
                    match lib.get(b"plugin") {
                        Ok(func) => func,
                        Err(err) => {
                            println!("Error: {:?}", err);
                            return;
                        }
                    };

                func(id, sender, module.with.clone());
                print!("Module {} loaded", module.name);
            }
        });
    }

    for mut package in receiver {
        if let Some(data) = package.get_data() {
            let mut context = Context::from_main(data.clone());
            let result = match phlow.execute_with_context(&mut context) {
                Ok(result) => result,
                Err(err) => {
                    println!("Error: {:?}", err);
                    continue;
                }
            };

            package.send(result.unwrap_or(Value::Null));
        }
    }
}
