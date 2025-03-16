mod loader;
mod opentelemetry;

use loader::{load_module, Loader};
use opentelemetry::init_tracing_subscriber;
use phlow_rule_engine::{build_engine_async, collector::Step, Context, Phlow};
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

    let (flow_sender, flow_receiver) = channel::<Step>();
    let engine = build_engine_async(None);

    let flow = {
        let steps: Value = loader.get_steps();

        match Phlow::try_from_value(&engine, &steps, None, Some(flow_sender)) {
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
        for step in flow_receiver {
            process_step(step);
        }
    });

    for mut package in receiver_package {
        process_package(&flow, &mut package);
    }
}

#[tracing::instrument]
fn process_step(step: Step) {
    debug!("Processing step: {:?}", step);
}

#[tracing::instrument]
fn process_package(flow: &Phlow, package: &mut Package) {
    debug!("Processing package: {:?}", package);

    if let Some(data) = package.get_data() {
        let mut context = Context::from_main(data.clone());
        let result = match flow.execute_with_context(&mut context) {
            Ok(result) => result,
            Err(err) => {
                error!("Runtime Error: {:?}", err);
                return;
            }
        };

        package.send(result.unwrap_or(Value::Null));
    }
}
