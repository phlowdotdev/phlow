use phlow_rule_engine::{collector::Step, Context, Phlow};
use sdk::prelude::*;
use tracing::{debug, error};

#[tracing::instrument]
pub fn step(step: Step) {
    debug!("Processing step: {:?}", step);
}

#[tracing::instrument]
pub fn module(flow: &Phlow, package: &mut Package) {
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
