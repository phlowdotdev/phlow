mod loader;
mod processes;
mod yaml;
use crossbeam::channel;
use loader::{load_module, Loader};
use phlow_engine::{
    build_engine_async,
    collector::Step,
    modules::{ModulePackage, Modules},
    Phlow,
};
use sdk::{opentelemetry::init_tracing_subscriber, prelude::*};
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::{debug, error};

#[tokio::main]
async fn main() {
    let _guard = init_tracing_subscriber();

    let loader = match Loader::load() {
        Ok(main) => main,
        Err(err) => {
            error!("Runtime Error Main File: {:?}", err);
            return;
        }
    };

    let engine = build_engine_async(None);
    let steps: Value = loader.get_steps();

    let (tx_trace_step, rx_trace_step) = channel::unbounded::<Step>();
    let (tx_main_package, rx_main_package) = channel::unbounded::<Package>();

    let mut modules = Modules::default();

    for (id, module) in loader.modules.into_iter().enumerate() {
        let (setup_sender, setup_receive) =
            oneshot::channel::<Option<channel::Sender<ModulePackage>>>();

        let main_sender = if loader.main == id as i32 {
            Some(tx_main_package.clone())
        } else {
            None
        };

        let setup = ModuleSetup {
            id,
            setup_sender,
            main_sender,
            with: module.with.clone(),
        };

        let module_target = module.module.clone();

        tokio::task::spawn(async move {
            if let Err(err) = load_module(setup, &module_target) {
                error!("Runtime Error Load Module: {:?}", err)
            }
        });

        debug!(
            "Module {} loaded with name \"{}\"",
            module.module, module.name
        );

        match setup_receive.await {
            Ok(Some(sender)) => {
                debug!("Module {} registered", module.name);
                modules.register(&module.name, sender);
            }
            Ok(None) => {
                debug!("Module {} did not register", module.name);
            }
            Err(err) => {
                error!("Runtime Error Setup Receive: {:?}", err);
                return;
            }
        }
    }

    debug!("Starting Phlow");

    let flow = {
        match Phlow::try_from_value(
            &engine,
            &steps,
            Some(Arc::new(modules)),
            Some(tx_trace_step),
        ) {
            Ok(flow) => flow,
            Err(err) => {
                error!("Runtime Error To Value: {:?}", err);
                return;
            }
        }
    };

    tokio::task::spawn(async move {
        for step in rx_trace_step {
            processes::step(step);
        }
    });

    if loader.main >= 0 {
        debug!("Main module exist");
        for mut package in rx_main_package {
            processes::execute_steps(&flow, &mut package).await;
        }
    }

    debug!("Phlow finished");
}
