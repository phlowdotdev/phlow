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
    Arc,
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

    let engine = build_engine_async(None);
    let steps: Value = loader.get_steps();

    let (trace_step_sender, trace_step_receiver) = channel::<Step>();
    let (main_sender_package, main_receiver_package) = channel::<Package>();

    let mut modules = Modules::default();

    for (id, module) in loader.modules.into_iter().enumerate() {
        let (setup_sender, setup_receive) = oneshot::channel::<Option<Sender<ModulePackage>>>();

        let main_sender = if loader.main == id as i32 {
            Some(main_sender_package.clone())
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

        debug!("Module {} loaded", module.name);

        match setup_receive.await {
            Ok(Some(sender)) => {
                debug!("Module {} registered", module.name);
                modules.register(&module.name, sender);
            }
            Ok(None) => {
                debug!("Module {} did not register", module.name);
            }
            Err(err) => {
                error!("Runtime Error: {:?}", err);
                return;
            }
        }
    }

    debug!("Starting Phlow");

    let flow = {
        match Phlow::try_from_value(
            &engine,
            &steps,
            None,
            Some(Arc::new(modules)),
            Some(trace_step_sender),
        ) {
            Ok(flow) => flow,
            Err(err) => {
                error!("Runtime Error: {:?}", err);
                return;
            }
        }
    };

    tokio::task::spawn(async move {
        for step in trace_step_receiver {
            processes::step(step);
        }
    });

    for mut package in main_receiver_package {
        processes::execute_steps(&flow, &mut package).await;
    }
}
