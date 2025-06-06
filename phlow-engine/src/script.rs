use crate::context::Context;
use phlow_sdk::prelude::*;
use phs::ScriptError;
use rhai::{plugin::*, serde::to_dynamic, Engine, Scope};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Script {
    pub script: phs::Script,
}

impl Script {
    pub fn try_build(engine: Arc<Engine>, script: &Value) -> Result<Self, ScriptError> {
        let script = phs::Script::try_build(engine, script)?;
        Ok(Self { script })
    }

    pub fn evaluate(&self, context: &Context) -> Result<Value, ScriptError> {
        let mut scope = Scope::new();

        let steps: Dynamic = to_dynamic(context.steps.clone()).map_err(ScriptError::EvalError)?;
        let main: Dynamic = to_dynamic(context.main.clone()).map_err(ScriptError::EvalError)?;
        let payload: Dynamic =
            to_dynamic(context.payload.clone()).map_err(ScriptError::EvalError)?;
        let input: Dynamic = to_dynamic(context.input.clone()).map_err(ScriptError::EvalError)?;

        scope.push_constant("steps", steps);
        scope.push_constant("main", main);
        scope.push_constant("payload", payload);
        scope.push_constant("input", input);

        self.script.evaluate_from_scope(&mut scope)
    }
}
