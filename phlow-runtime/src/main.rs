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
use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
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
    let modules_len = loader.modules.len();

    let (setup_sender_main_package, setup_receiver_main_package) = channel::<Package>();

    let (complete_module_sender, complete_module_receiver) = channel::<usize>();

    let modules = Arc::new(Mutex::new(Modules::default()));

    for (id, module) in loader.modules.into_iter().enumerate() {
        let (setup_sender, setup_receive) = oneshot::channel::<Sender<ModulePackage>>();

        let main_sender = if loader.main == id as i32 {
            Some(setup_sender_main_package.clone())
        } else {
            None
        };

        let setup = ModuleSetup {
            id,
            setup_sender,
            main_sender,
            with: module.with.clone(),
        };

        let module_name = module.name.clone();

        tokio::task::spawn(async move {
            if let Err(err) = load_module(setup, &module_name) {
                error!("Runtime Error: {:?}", err)
            }
        });

        let setup_data = setup_receive.await.unwrap();
        modules.lock().unwrap().register(&module.name, setup_data);
        complete_module_sender.send(id).unwrap();
    }

    {
        let mut total_modules_loaded = 0;
        for module_id in complete_module_receiver {
            debug!("Module {} loaded", module_id);
            total_modules_loaded += 1;

            if total_modules_loaded == modules_len {
                break;
            }
        }
    }

    let modules = {
        let modules = match modules.lock() {
            Ok(modules) => modules.extract(),
            Err(err) => {
                error!("Runtime Error: {:?}", err);
                return;
            }
        };

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

    for mut package in setup_receiver_main_package {
        processes::execute_steps(&flow, &mut package).await;
    }
}
