mod loader;
use clap::{Arg, Command};
use libloading::{Library, Symbol};
use loader::Loader;
use phlow_rule_engine::{build_engine_async, Context, Phlow};
use sdk::prelude::*;

#[tokio::main]
async fn main() {
    let matches = Command::new("Phlow Runtime")
        .version("0.1.0")
        .arg(
            Arg::new("main_file")
                .help("Main file to load")
                .required(true)
                .index(1),
        )
        .get_matches();

    let config = match matches.get_one::<String>("main_file") {
        Some(file) => {
            let file = std::fs::read_to_string(file).unwrap();
            match Value::json_to_value(&file) {
                Ok(value) => value,
                Err(err) => {
                    println!("Error: {:?}", err);
                    return;
                }
            }
        }
        None => {
            println!("Error: No main file provided");
            return;
        }
    };

    let loader = match Loader::try_from(config) {
        Ok(main) => main,
        Err(err) => {
            println!("Error: {:?}", err);
            return;
        }
    };

    let steps: Value = loader.get_steps();
    let engine = build_engine_async(None);

    let flow = match Phlow::try_from_value(&engine, &steps, None, None) {
        Ok(flow) => flow,
        Err(err) => {
            println!("Error: {:?}", err);
            return;
        }
    };

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
            let result = match flow.execute_with_context(&mut context) {
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
