mod loader;
mod opentelemetry;
mod processes;

use loader::{load_module, Loader};
use opentelemetry::init_tracing_subscriber;
use phlow_rule_engine::{build_engine_async, collector::Step, Phlow};
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

    let flow = {
        let steps: Value = loader.get_steps();

        match Phlow::try_from_value(&engine, &steps, None, Some(sender_step)) {
            Ok(flow) => flow,
            Err(err) => {
                error!("Runtime Error: {:?}", err);
                return;
            }
        }
    };

    let (sender_package, receiver_package) = channel::<Package>();

    for (id, module) in loader.modules.into_iter().enumerate() {
        let sender = sender_package.clone();

        tokio::task::spawn(async move {
            match load_module(id, sender, &module) {
                Ok(_) => debug!("Module {} loaded", module.name),
                Err(err) => error!("Runtime Error: {:?}", err),
            }
        });
    }

    tokio::task::spawn(async move {
        for step in receiver_step {
            processes::step(step);
        }
    });

    for mut package in receiver_package {
        processes::module(&flow, &mut package);
    }
}
