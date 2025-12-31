use crate::context::Context;
use phlow_sdk::prelude::*;
use phs::ScriptError;
use rhai::{Engine, Scope, plugin::*, serde::to_dynamic};
use std::collections::HashMap;
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

        let steps: Dynamic =
            to_dynamic(context.get_steps().clone()).map_err(ScriptError::EvalError)?;
        let main: Dynamic =
            to_dynamic(context.get_main().clone()).map_err(ScriptError::EvalError)?;
        let payload: Dynamic =
            to_dynamic(context.get_payload().clone()).map_err(ScriptError::EvalError)?;
        let input: Dynamic =
            to_dynamic(context.get_input().clone()).map_err(ScriptError::EvalError)?;
        let setup: Dynamic =
            to_dynamic(context.get_setup().clone()).map_err(ScriptError::EvalError)?;
        let tests: Dynamic =
            to_dynamic(context.get_tests().clone()).map_err(ScriptError::EvalError)?;

        scope.push_constant("tests", tests);
        scope.push_constant("steps", steps);
        scope.push_constant("main", main);
        scope.push_constant("payload", payload);
        scope.push_constant("input", input);
        scope.push_constant("setup", setup);

        self.script.evaluate_from_scope(&mut scope)
    }

    pub fn compiled_debug(&self) -> Value {
        let mut map = HashMap::new();
        let mut compiled = HashMap::new();
        for (index, code) in self.script.compiled_sources() {
            compiled.insert(index.to_string(), code.to_value());
        }
        map.insert("template".to_string(), self.script.compiled_template());
        map.insert("compiled".to_string(), compiled.to_value());
        map.insert("expanded".to_string(), self.script.compiled_value());
        map.to_value()
    }
}
