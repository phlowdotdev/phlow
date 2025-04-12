use crate::args::Args;
use phlow_sdk::{
    prelude::*,
    structs::{MainRuntimeSender, ModuleId},
    tracing::{Dispatch, Span},
};
use std::convert::Infallible;

pub struct RequestContext {
    pub args: Args,
    pub span: Span,
    pub dispatch: Dispatch,
    pub id: ModuleId,
    pub sender: MainRuntimeSender,
}

pub async fn resolve(context: RequestContext) -> Result<Value, Infallible> {
    let response_value = sender_package!(
        context.span.clone(),
        context.dispatch.clone(),
        context.id,
        context.sender,
        Some(context.args.args.to_value())
    )
    .await
    .unwrap_or(Value::Null);

    Ok(response_value)
}
