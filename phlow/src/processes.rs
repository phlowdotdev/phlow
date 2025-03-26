use phlow_engine::{collector::Step, Context, Phlow};
use sdk::{prelude::*, tracing::warn};
use tracing::debug;

#[tracing::instrument]
pub fn step(step: Step) {
    debug!("Processing step: {:?}", step.to_value());
}

#[tracing::instrument(skip(flow))]
pub async fn execute_steps(flow: &Phlow, package: &mut Package) {
    debug!("Processing package: {:?}", package);

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
}
