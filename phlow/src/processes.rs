use phlow_engine::{collector::Step, Context, Phlow};
use sdk::{
    prelude::*,
    tracing::{dispatcher, span, warn, Level},
};
use tracing::debug;

pub fn step(step: Step) {
    debug!("Processing step: {:?}", step.to_value());
}

pub fn execute_steps(flow: &Phlow, package: &mut Package) {
    debug!("Processing package: {:?}", package);

    let dispatch = package
        .dispatch
        .clone()
        .unwrap_or(dispatcher::get_default(|d| d.clone()));

    dispatcher::with_default(&dispatch, || {
        if let Some(data) = package.get_data() {
            let mut context = Context::from_main(data.clone());

            let rt = tokio::runtime::Handle::current();

            rt.block_on(async {
                let span = span!(Level::INFO, "steps");
                let _enter = span.enter();

                let result = match flow.execute(&mut context).await {
                    Ok(result) => result,
                    Err(err) => {
                        warn!("Runtime Error Execute Steps: {:?}", err);
                        return;
                    }
                };

                package.send(result.unwrap_or(Value::Null));
            });
        }
    });
}
