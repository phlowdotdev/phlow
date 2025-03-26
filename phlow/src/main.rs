mod envs;
mod loader;
mod processes;
mod yaml;
use crossbeam::channel;
use envs::Envs;
use loader::{load_module, Loader};
use phlow_engine::{
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
    let envs = Envs::load();
    let _guard = init_tracing_subscriber();

    debug!("STEP_CONSUMERS = {}", envs.step_consumer_count);
    debug!("PACKAGE_CONSUMERS = {}", envs.package_consumer_count);

    // -------------------------
    // Load the main file
    // -------------------------
    let loader = match Loader::load() {
        Ok(main) => main,
        Err(err) => {
            error!("Runtime Error Main File: {:?}", err);
            return;
        }
    };

    let steps: Value = loader.get_steps();

    // -------------------------
    // Create the channels
    // -------------------------
    let (tx_trace_step, rx_trace_step) = channel::unbounded::<Step>();
    let (tx_main_package, rx_main_package) = channel::unbounded::<Package>();

    let mut modules = Modules::default();

    // -------------------------
    // Load the modules
    // -------------------------
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

    // -------------------------
    // Create the flow
    // -------------------------

    let flow = {
        match Phlow::try_from_value(&steps, Some(Arc::new(modules)), Some(tx_trace_step)) {
            Ok(flow) => flow,
            Err(err) => {
                error!("Runtime Error To Value: {:?}", err);
                return;
            }
        }
    };

    // Opcional: se você quer compartilhar 'flow' facilmente entre tasks
    let flow_arc = Arc::new(flow);

    for i in 0..envs.step_consumer_count {
        let rx_clone = rx_trace_step.clone();

        tokio::task::spawn_blocking(move || {
            // Esse loop bloqueia a thread enquanto espera mensagens
            for step in rx_clone {
                processes::step(step);
            }
            debug!("Step consumer #{} terminou (canal fechado).", i);
        });
    }

    if loader.main >= 0 {
        debug!("Main module exist");

        for i in 0..envs.package_consumer_count {
            let rx_pkg = rx_main_package.clone();
            let flow_ref = Arc::clone(&flow_arc);

            tokio::task::spawn_blocking(move || {
                for mut package in rx_pkg {
                    // processes::execute_steps é assíncrona => usamos block_in_place
                    // para chamá-la dentro de um ambiente bloqueante:
                    tokio::task::block_in_place(|| {
                        // Obter handle da runtime e rodar a future
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async {
                            processes::execute_steps(&flow_ref, &mut package).await;
                        });
                    });
                }
                debug!("Package consumer #{} terminou (canal fechado).", i);
            });
        }
    }

    drop(tx_main_package);

    debug!("Phlow finished.");
}
