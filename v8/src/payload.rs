use crate::pipeline::Context;
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
    script: String,
}

impl Payload {
    pub fn new(script: String) -> Self {
        Self { script }
    }

    pub fn execute(&self, context: &Context) -> Result<Value, PayloadError> {
        let engine = Engine::new();
        let mut scope = Scope::new();

        let steps: Dynamic = to_dynamic(context.steps.clone()).unwrap();
        let context: Dynamic = to_dynamic(context.context.clone()).unwrap();

        scope.push_constant("steps", steps);
        scope.push_constant("context", context);

        let result = engine
            .eval_with_scope(&mut scope, &self.script)
            .map_err(PayloadError::EvalError)?;

        let result: Value = from_dynamic(&result).unwrap();

        Ok(result)
    }

    pub fn execute_variable(&self, context: &Context) -> Result<Variable, PayloadError> {
        let value = self.execute(context)?;
        Ok(Variable::new(value))
    }
}

#[cfg(test)]
mod test {
    use crate::{step::StepType, Step};

    use super::*;
    use std::collections::HashMap;
    use valu3::value::Value;

    #[test]
    fn test_payload_execute() {
        let script = r#"
            let a = 10;
            let b = 20;
            a + b
        "#;

        let context = Context::new(Value::Null);
        let payload = Payload::new(script.to_string());

        let result = payload.execute(&context).unwrap();
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

        let context = Context::new(Value::Null);
        let payload = Payload::new(script.to_string());

        let result = payload.execute(&context).unwrap();
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

        let context = Context::new(Value::Null);
        let payload = Payload::new(script.to_string());

        let variable = payload.execute_variable(&context).unwrap();
        assert_eq!(variable, Variable::new(Value::from("hello world")));
    }

    #[test]
    fn test_payload_execute_variable_context() {
        let script = r#"
            let a = context.a;
            let b = context.b;
            a + b
        "#;

        let context = Context::new(Value::from({
            let mut map = HashMap::new();
            map.insert("a".to_string(), Value::from(10i64));
            map.insert("b".to_string(), Value::from(20i64));
            map
        }));

        let payload = Payload::new(script.to_string());

        let variable = payload.execute_variable(&context).unwrap();
        assert_eq!(variable, Variable::new(Value::from(30i64)));
    }

    #[test]
    fn test_payload_execute_variable_step() {
        let script = r#"
            let a = steps.0.a;
            let b = steps.0.b;
            println(a)
            a + b
        "#;
        let step = Step::new(
            Some("0".to_string()),
            None,
            StepType::Default,
            None,
            None,
            None,
            None,
            None,
            Some(Value::Null),
        );

        let mut context = Context::new(Value::Null);
        context.add_step_output(&step, {
            let mut map = HashMap::new();
            map.insert("a".to_string(), Value::from(10i64));
            map.insert("b".to_string(), Value::from(20i64));
            map
        });

        let payload = Payload::new(script.to_string());
        let variable = payload.execute_variable(&context).unwrap();

        assert_eq!(variable, Variable::new(Value::from(30i64)));
    }
}
