use phlow_engine::{Context, Phlow};
use sdk::tracing_opentelemetry::OpenTelemetrySpanExt;
use sdk::{
    prelude::*,
    tracing::{dispatcher, span, warn, Level},
};
use tracing::debug;

pub fn execute_steps(flow: &Phlow, package: &mut Package) {
    debug!("Processing package: {:?}", package);

    let dispatch = package
        .dispatch
        .clone()
        .unwrap_or(dispatcher::get_default(|d| d.clone()));

    let rt = tokio::runtime::Runtime::new().unwrap();

    dispatcher::with_default(&dispatch, || {
        let parent = package.span.clone().unwrap();
        let span = span!(Level::INFO, "execute_steps");
        span.set_parent(parent.context());
        let _enter = span.enter();

        rt.block_on(async {
            if let Some(data) = package.get_data() {
                let mut context = Context::from_main(data.clone());
                let result = match flow.execute(&mut context).await {
                    Ok(result) => result,
                    Err(err) => {
                        warn!("Runtime Error Execute Steps: {:?}", err);
                        return;
                    }
                };

                package.send(result.unwrap_or(Value::Null));
            }
        });
    });
}
