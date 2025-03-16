mod loader;
mod opentelemetry;
mod processes;

use loader::{load_module, Loader};
use opentelemetry::init_tracing_subscriber;
use phlow_engine::{
    build_engine_async,
    collector::Step,
    modules::{ModulePackage, Modules},
    Phlow,
};
use sdk::prelude::*;
use std::{
    hash::Hash,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
};
use tokio::sync::oneshot;
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

    let (setup_sender_main_package, setup_receiver_main_package) = channel::<Package>();

    let modules = {
        let modules = Arc::new(Mutex::new(Modules::default()));

        for (id, module) in loader.modules.into_iter().enumerate() {
            let (setup_sender, setup_receive) = oneshot::channel::<Sender<ModulePackage>>();

            let main_sender = if loader.main == id as i32 {
                Some(setup_sender_main_package.clone())
            } else {
                None
            };

            tokio::task::spawn(async move {
                let setup = ModuleSetup {
                    id,
                    setup_sender,
                    main_sender,
                };

                match load_module(setup, &module) {
                    Ok(sender) => debug!("Main module {} loaded", module.name),
                    Err(err) => error!("Runtime Error: {:?}", err),
                }

                let setup_data = setup_receive.await.unwrap();

                modules.lock().unwrap().register(&module.name, setup_data);
            });
        }

        let modules = modules.lock().unwrap().extract();
        Arc::new(modules)
    };

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
        for package in setup_receiver_package {
            println!("{:?}", package);
        }
    });

    for mut package in setup_receiver_main_package {
        processes::execute_steps(&flow, &mut package).await;
    }
}
