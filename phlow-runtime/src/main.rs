mod loader;
mod opentelemetry;
use clap::{Arg, Command};
use libloading::{Library, Symbol};
use loader::Loader;
use opentelemetry::init_tracing_subscriber;
use phlow_rule_engine::{build_engine_async, Context, Phlow};
use sdk::prelude::*;
use tracing::{error, info, span};

#[tokio::main]
async fn main() {
    let _guard = init_tracing_subscriber();

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
                    error!("Error: {:?}", err);
                    return;
                }
            }
        }
        None => {
            error!("Error: No main file provided");
            return;
        }
    };

    let loader = match Loader::try_from(config) {
        Ok(main) => main,
        Err(err) => {
            error!("Error: {:?}", err);
            return;
        }
    };

    let steps: Value = loader.get_steps();
    let engine = build_engine_async(None);

    let flow = match Phlow::try_from_value(&engine, &steps, None, None) {
        Ok(flow) => flow,
        Err(err) => {
            error!("Error: {:?}", err);
            return;
        }
    };

    let (sender, receiver) = std::sync::mpsc::channel::<Package>();

    {
        for module in loader.modules.iter() {
            let path = format!("phlow_modules/{}.so", module.name);

            if !std::path::Path::new(&path).exists() {
                error!("Error: Module {} does not exist", module.name);
                return;
            }
        }
    }

    for (id, module) in loader.modules.into_iter().enumerate() {
        let sender = sender.clone();

        tokio::task::spawn(async move {
            unsafe {
                info!("Loading module: {}", module.name);
                let lib = match Library::new(format!("phlow_modules/{}.so", module.name).as_str()) {
                    Ok(lib) => lib,
                    Err(err) => {
                        error!("Error: {:?}", err);
                        return;
                    }
                };
                let func: Symbol<unsafe extern "C" fn(ModuleId, RuntimeSender, Value)> =
                    match lib.get(b"plugin") {
                        Ok(func) => func,
                        Err(err) => {
                            error!("Error: {:?}", err);
                            return;
                        }
                    };

                func(id, sender, module.with.clone());
                info!("Module {} loaded", module.name);
            }
        });
    }

    for mut package in receiver {
        process_package(&flow, &mut package);
    }
}

#[tracing::instrument]
fn process_package(flow: &Phlow, package: &mut Package) {
    if let Some(data) = package.get_data() {
        let mut context = Context::from_main(data.clone());
        let result = match flow.execute_with_context(&mut context) {
            Ok(result) => result,
            Err(err) => {
                error!("Error: {:?}", err);
                return;
            }
        };

        package.send(result.unwrap_or(Value::Null));
    }
}
