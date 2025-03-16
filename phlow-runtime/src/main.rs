mod loader;
mod opentelemetry;
mod processes;

use loader::{load_module, Loader};
use opentelemetry::init_tracing_subscriber;
use phlow_engine::{build_engine_async, collector::Step, modules::Modules, Phlow};
use sdk::prelude::*;
use std::sync::mpsc::channel;
use tracing::{debug, error};

#[tokio::main]
async fn main() {
    let _guard = init_tracing_subscriber();

    let loader = match Loader::load() {
        Ok(main) => main,
        Err(err) => {
            error!("Runtime Error: {:?}", err);
            return;
        }
    };

    let (sender_step, receiver_step) = channel::<Step>();
    let engine = build_engine_async(None);
    let steps: Value = loader.get_steps();

    let (sender_main_package, receiver_main_package) = channel::<Package>();
    let (sender_package, receiver_package) = channel::<Package>();
    let mut modules = Modules::default();

    for (id, module) in loader.modules.into_iter().enumerate() {
        modules.register(&module.name);

        if loader.main == id as i32 {
            let sender = sender_main_package.clone();

            tokio::task::spawn(async move {
                match load_module(id, sender, &module) {
                    Ok(_) => debug!("Main module {} loaded", module.name),
                    Err(err) => error!("Runtime Error: {:?}", err),
                }
            });
        } else {
            let sender = sender_package.clone();

            tokio::task::spawn(async move {
                match load_module(id, sender, &module) {
                    Ok(_) => debug!("Module {} loaded", module.name),
                    Err(err) => error!("Runtime Error: {:?}", err),
                }
            });
        }
    }

    let flow = {
        match Phlow::try_from_value(&engine, &steps, None, Some(modules), Some(sender_step)) {
            Ok(flow) => flow,
            Err(err) => {
                error!("Runtime Error: {:?}", err);
                return;
            }
        }
    };

    tokio::task::spawn(async move {
        for step in receiver_step {
            processes::step(step);
        }
    });

    tokio::task::spawn(async move {
        for package in receiver_package {
            println!("{:?}", package);
        }
    });

    for mut package in receiver_main_package {
        processes::execute_steps(&flow, &mut package);
    }
}
