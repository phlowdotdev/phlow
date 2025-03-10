use std::collections::HashMap;

use crate::v8::Context;
use crate::variable::Variable;
use regex::Regex;
use rhai::plugin::*;
use rhai::serde::{from_dynamic, to_dynamic};
use rhai::{Engine, EvalAltResult, Scope};
use serde::{Deserialize, Serialize};
use valu3::prelude::*;

#[derive(Debug)]
pub enum ScriptError {
    EvalError(Box<EvalAltResult>),
    InvalidType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    map_extracted: Value,
    map_index: HashMap<usize, Value>,
}

impl Script {
    fn new(script: String) -> Self {
        let mut map_index = HashMap::new();
        let mut counter = 0;
        let map_extracted = extract_primitives(&Value::from(script), &mut map_index, &mut counter);

        Self {
            map_extracted,
            map_index,
        }
    }

    pub fn evaluate(&self, context: &Context) -> Result<Value, ScriptError> {
        let mut engine = Engine::new();

        engine
            .register_custom_operator("starts_with", 80)
            .unwrap()
            .register_fn("start_withs", |x: String, y: String| x.starts_with(&y));

        engine
            .register_custom_operator("ends_with", 81)
            .unwrap()
            .register_fn("ends_with", |x: String, y: String| x.ends_with(&y));

        engine
            .register_custom_operator("search", 82)
            .unwrap()
            .register_fn("search", |x: String, y: String| {
                // regex
                Regex::new(&x).unwrap().is_match(&y)
            });

        let mut scope = Scope::new();

        let steps: Dynamic = to_dynamic(context.steps.clone()).unwrap();
        let params: Dynamic = to_dynamic(context.params.clone()).unwrap();

        scope.push_constant("steps", steps);
        scope.push_constant("params", params);

        let mut new_map_index: HashMap<usize, Value> = HashMap::new();

        for (key, value) in self.map_index.iter() {
            let value = engine
                .eval_with_scope(&mut scope, &value.to_string())
                .map_err(ScriptError::EvalError)?;

            new_map_index.insert(*key, from_dynamic(&value).unwrap());
        }

        let result = replace_primitives(&self.map_extracted, &new_map_index);

        Ok(result)
    }

    pub fn evaluate_variable(&self, context: &Context) -> Result<Variable, ScriptError> {
        let value = self.evaluate(context)?;
        Ok(Variable::new(value))
    }
}

impl From<&str> for Script {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl From<String> for Script {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

fn extract_primitives(
    value: &Value,
    map_exp: &mut HashMap<usize, Value>,
    counter: &mut usize,
) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = HashMap::new();
            for (key, value) in map.iter() {
                new_map.insert(key.to_string(), extract_primitives(value, map_exp, counter));
            }
            Value::from(new_map)
        }
        Value::Array(array) => {
            let mut new_array = Vec::new();
            for value in array.into_iter() {
                new_array.push(extract_primitives(value, map_exp, counter));
            }
            Value::from(new_array)
        }
        _ => {
            map_exp.insert(*counter, value.clone());
            let result = Value::from(*counter);
            *counter += 1;
            result
        }
    }
}

fn replace_primitives(value: &Value, map_exp: &HashMap<usize, Value>) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = HashMap::new();
            for (key, value) in map.iter() {
                new_map.insert(key.to_string(), replace_primitives(value, map_exp));
            }
            Value::from(new_map)
        }
        Value::Array(array) => {
            let mut new_array = Vec::new();
            for value in array.into_iter() {
                new_array.push(replace_primitives(value, map_exp));
            }
            Value::from(new_array)
        }
        Value::Number(number) => {
            let index = number.to_i64().unwrap() as usize;
            map_exp.get(&index).unwrap().clone()
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod test {
    use crate::{id::ID, step_worker::StepWorker};

    use super::*;
    use std::collections::HashMap;
    use valu3::{json, traits::ToValueBehavior, value::Value};

    #[test]
    fn test_payload_extract_primitive_object() {
        let value = json!({
            "a": "Hello",
            "b": 20,
            "c": {
                "d": 30,
                "e": "World"
            },
            "f": [1, 2, 3, 4]
        });

        let mut map_exp = HashMap::new();
        let mut counter = 0;
        let result = extract_primitives(&value, &mut map_exp, &mut counter);
        let result = replace_primitives(&result, &map_exp);

        assert_eq!(result, value);
    }

    #[test]
    fn test_payload_extract_primitive_array() {
        let value = json!(["Hello", 20, {"d": 30, "e": "World"}, [1, 2, 3, 4]]);

        let mut map_exp = HashMap::new();
        let mut counter = 0;
        let result = extract_primitives(&value, &mut map_exp, &mut counter);

        let result = replace_primitives(&result, &map_exp);

        assert_eq!(result, value);
    }

    #[test]
    fn test_payload_extract_primitive_string() {
        let value = json!("Hello World");

        let mut map_exp = HashMap::new();
        let mut counter = 0;
        let result = extract_primitives(&value, &mut map_exp, &mut counter);

        let result = replace_primitives(&result, &map_exp);

        assert_eq!(result, value);
    }

    #[test]
    fn test_payload_execute() {
        let script = r#"
            let a = 10;
            let b = 20;
            a + b
        "#;

        let context = Context::new(None);
        let payload = Script::from(script.to_string());

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
        let payload = Script::from(script.to_string());

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
        let payload = Script::from(script.to_string());

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

        let payload = Script::from(script.to_string());

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

        let payload = Script::from(script.to_string());

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

        let payload = Script::from(script.to_string());
        let variable = payload.evaluate_variable(&context).unwrap();

        assert_eq!(variable, Variable::new(Value::from(30i64)));
    }
}
