use crate::v8::Context;
use crate::variable::Variable;
use rhai::plugin::*;
use rhai::serde::{from_dynamic, to_dynamic};
use rhai::{Engine, EvalAltResult, Scope};
use serde::{Deserialize, Serialize};
use valu3::value::Value;

#[derive(Debug)]
pub enum PayloadError {
    EvalError(Box<EvalAltResult>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    pub(crate) script: String,
}

impl From<Value> for Payload {
    fn from(value: Value) -> Self {
        let script = value.to_string();
        Self { script }
    }
}

impl From<&Value> for Payload {
    fn from(value: &Value) -> Self {
        let script = value.to_string();
        Self { script }
    }
}

impl From<&str> for Payload {
    fn from(value: &str) -> Self {
        let script = value.to_string();
        Self { script }
    }
}

impl Payload {
    pub fn new(script: String) -> Self {
        Self { script }
    }

    pub fn evaluate(&self, context: &Context) -> Result<Value, PayloadError> {
        let engine = Engine::new();
        let mut scope = Scope::new();

        let steps: Dynamic = to_dynamic(context.steps.clone()).unwrap();
        let params: Dynamic = to_dynamic(context.params.clone()).unwrap();

        scope.push_constant("steps", steps);
        scope.push_constant("params", params);

        let result = engine
            .eval_with_scope(&mut scope, &self.script)
            .map_err(PayloadError::EvalError)?;

        let result: Value = from_dynamic(&result).unwrap();
        println!("result {:?}", result);
        Ok(result)
    }

    pub fn evaluate_variable(&self, context: &Context) -> Result<Variable, PayloadError> {
        let value = self.evaluate(context)?;
        Ok(Variable::new(value))
    }
}

#[cfg(test)]
mod test {
    use crate::{id::ID, step_worker::StepWorker};

    use super::*;
    use std::collections::HashMap;
    use valu3::{traits::ToValueBehavior, value::Value};

    #[test]
    fn test_payload_execute() {
        let script = r#"
            let a = 10;
            let b = 20;
            a + b
        "#;

        let context = Context::new(None);
        let payload = Payload::new(script.to_string());

        let result = payload.evaluate(&context).unwrap();
        assert_eq!(result, Value::from(30i64));
    }

    #[test]
    fn test_payload_json() {
        let script = r#"
            let a = 10;
            let b = 20;
            
            #{
                a: a,
                b: b,
                sum: a + b
            }
        "#;

        let context = Context::new(None);
        let payload = Payload::new(script.to_string());

        let result = payload.evaluate(&context).unwrap();
        let expected = Value::from({
            let mut map = HashMap::new();
            map.insert("a".to_string(), Value::from(10i64));
            map.insert("b".to_string(), Value::from(20i64));
            map.insert("sum".to_string(), Value::from(30i64));
            map
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_payload_execute_variable() {
        let script = r#""hello world""#;

        let context = Context::new(None);
        let payload = Payload::new(script.to_string());

        let variable = payload.evaluate_variable(&context).unwrap();
        assert_eq!(variable, Variable::new(Value::from("hello world")));
    }

    #[test]
    fn test_payload_execute_variable_context() {
        let script = r#"
            let a = params.a;
            let b = params.b;
            a + b
        "#;

        let context = Context::new(Some(Value::from({
            let mut map = HashMap::new();
            map.insert("a".to_string(), Value::from(10i64));
            map.insert("b".to_string(), Value::from(20i64));
            map
        })));

        let payload = Payload::new(script.to_string());

        let variable = payload.evaluate_variable(&context).unwrap();
        assert_eq!(variable, Variable::new(Value::from(30i64)));
    }

    #[test]
    fn test_payload_execute_variable_context_params() {
        let script = r#"params.a"#;

        let context = Context::new(Some(Value::from({
            let mut map = HashMap::new();
            map.insert("a".to_string(), Value::from(10i64));
            map.insert("b".to_string(), Value::from(20i64));
            map
        })));

        let payload = Payload::new(script.to_string());

        let variable = payload.evaluate_variable(&context).unwrap();
        assert_eq!(variable, Variable::new(Value::from(10i64)));
    }

    #[test]
    fn test_payload_execute_variable_step() {
        let script = r#"
            let a = steps.me.a;
            let b = steps.me.b;
   
            a + b
        "#;
        let step = StepWorker {
            id: ID::from("me"),
            ..Default::default()
        };

        let mut context = Context::new(None);
        context.add_step_output(step.get_id().clone(), {
            let mut map = HashMap::new();
            map.insert("a".to_string(), Value::from(10i64));
            map.insert("b".to_string(), Value::from(20i64));
            map.to_value()
        });

        let payload = Payload::new(script.to_string());
        let variable = payload.evaluate_variable(&context).unwrap();

        assert_eq!(variable, Variable::new(Value::from(30i64)));
    }
}
