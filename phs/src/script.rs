use crate::variable::Variable;
use regex::Regex;
use rhai::{
    serde::{from_dynamic, to_dynamic},
    Engine, EvalAltResult, ParseError, Scope, AST,
};
use std::{collections::HashMap, fmt::Display, sync::Arc};
use valu3::prelude::*;

type Context = HashMap<String, Value>;

#[derive(Debug)]
pub enum ScriptError {
    EvalError(Box<EvalAltResult>),
    InvalidType(Value),
    CompileError(String, ParseError),
}

impl Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::EvalError(err) => write!(f, "Eval error: {}", err),
            ScriptError::InvalidType(value) => write!(f, "Invalid type: {}", value),
            ScriptError::CompileError(code, err) => write!(f, "Compile error: {}: {}", code, err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Script {
    map_extracted: Value,
    map_index_ast: HashMap<usize, AST>,
    engine: Arc<Engine>,
}

impl Script {
    pub fn try_build(engine: Arc<Engine>, script: &Value) -> Result<Self, ScriptError> {
        let mut map_index_ast = HashMap::new();
        let mut counter = 0;
        let map_extracted =
            Self::extract_primitives(&engine, &script, &mut map_index_ast, &mut counter)?;

        Ok(Self {
            map_extracted,
            map_index_ast,
            engine,
        })
    }

    pub fn to_code_string(code: &str) -> String {
        let code = code.trim();
        if code.starts_with("{{") && code.ends_with("}}") {
            code[2..code.len() - 2].to_string()
        } else if code.parse::<i128>().is_ok()
            || code.parse::<f64>().is_ok()
            || code == "true".to_string()
            || code == "false".to_string()
            || code == "null".to_string()
            || code == "undefined".to_string()
        {
            code.to_string()
        } else {
            format!("`{}`", code)
        }
    }

    pub fn evaluate_from_scope(&self, scope: &mut Scope) -> Result<Value, ScriptError> {
        Self::default_scope(scope)?;

        let mut result_map: HashMap<usize, Value> = HashMap::new();

        for (key, value) in self.map_index_ast.iter() {
            let value = self
                .engine
                .eval_ast_with_scope(scope, &value)
                .map_err(ScriptError::EvalError)?;

            result_map.insert(*key, from_dynamic(&value).map_err(ScriptError::EvalError)?);
        }

        let result = Self::replace_primitives(&self.map_extracted, &result_map);

        Ok(result)
    }

    pub fn evaluate(&self, context: &Context) -> Result<Value, ScriptError> {
        let mut scope = Scope::new();

        for (key, value) in context.iter() {
            let value = to_dynamic(value).map_err(ScriptError::EvalError)?;
            scope.push_constant(key, value);
        }

        self.evaluate_from_scope(&mut scope)
    }

    pub fn evaluate_without_context(&self) -> Result<Value, ScriptError> {
        self.evaluate(&Context::new())
    }

    pub fn evaluate_variable(&self, context: &Context) -> Result<Variable, ScriptError> {
        let value = self.evaluate(context)?;
        Ok(Variable::new(value))
    }

    fn default_scope(scope: &mut Scope) -> Result<(), ScriptError> {
        let envs = {
            let envs = std::env::vars()
                .map(|(key, value)| (key, value))
                .collect::<HashMap<String, String>>();

            to_dynamic(envs).map_err(ScriptError::EvalError)?
        };

        scope.push_constant("envs", envs);

        Ok(())
    }

    fn replace_null_safe(code: &str) -> String {
        let re = Regex::new(r"\bnull\b").unwrap();
        re.replace_all(code, "()").to_string()
    }

    fn extract_primitives(
        engine: &Engine,
        value: &Value,
        map_index_ast: &mut HashMap<usize, AST>,
        counter: &mut usize,
    ) -> Result<Value, ScriptError> {
        match value {
            Value::Object(map) => {
                let mut new_map = HashMap::new();

                for (key, value) in map.iter() {
                    let item = Self::extract_primitives(engine, value, map_index_ast, counter)?;
                    new_map.insert(key.to_string(), item);
                }

                Ok(Value::from(new_map))
            }
            Value::Array(array) => {
                let mut new_array = Vec::new();
                for value in array.into_iter() {
                    let item = Self::extract_primitives(engine, value, map_index_ast, counter)?;

                    new_array.push(item);
                }

                Ok(Value::from(new_array))
            }
            _ => {
                let code = Self::to_code_string(&value.to_string());

                let code_fixed = Self::replace_null_safe(&code);

                let ast = match engine.compile(&code_fixed) {
                    Ok(ast) => ast,
                    Err(err) => return Err(ScriptError::CompileError(code_fixed, err)),
                };
                map_index_ast.insert(*counter, ast);

                let result = Value::from(*counter);
                *counter += 1;

                Ok(result)
            }
        }
    }

    fn replace_primitives(map_extracted: &Value, result: &HashMap<usize, Value>) -> Value {
        match map_extracted {
            Value::Object(map) => {
                let mut new_map = HashMap::new();
                for (key, value) in map.iter() {
                    new_map.insert(key.to_string(), Self::replace_primitives(value, result));
                }
                Value::from(new_map)
            }
            Value::Array(array) => {
                let mut new_array = Vec::new();
                for value in array.into_iter() {
                    new_array.push(Self::replace_primitives(value, result));
                }
                Value::from(new_array)
            }
            _ => {
                let index = match map_extracted.to_i64() {
                    Some(index) => index as usize,
                    None => panic!("Index not found"),
                };
                let value = match result.get(&index) {
                    Some(value) => value.clone(),
                    None => panic!("Index not found"),
                };

                value
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::build_engine;
    use std::collections::HashMap;

    #[test]
    fn test_payload_execute() {
        let script: &str = r#"{{
            let a = 10;
            let b = 20;
            a + b
        }}"#;

        let context = Context::new();
        let engine = build_engine(None);
        let payload = Script::try_build(engine, &script.to_value()).unwrap();

        let result = payload.evaluate(&context).unwrap();
        assert_eq!(result, Value::from(30i64));
    }

    #[test]
    fn test_payload_json() {
        let script = r#"{{
            let a = 10;
            let b = 20;
            let c = "hello";
            
            #{
                a: a,
                b: b,
                sum: a + b
            }
        }}"#;

        let context = Context::new();
        let engine = build_engine(None);
        let payload = Script::try_build(engine, &script.to_value()).unwrap();

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
        let script = "hello world";

        let context = Context::new();
        let engine = build_engine(None);
        let payload = Script::try_build(engine, &script.to_value()).unwrap();

        let variable = payload.evaluate_variable(&context).unwrap();
        assert_eq!(variable, Variable::new(Value::from("hello world")));
    }

    #[test]
    fn test_payload_execute_variable_context() {
        let script = r#"{{
            let a = payload.a;
            let b = payload.b;
            a + b
        }}"#;

        let context = HashMap::from([(
            "payload".to_string(),
            HashMap::from([
                ("a".to_string(), Value::from(10i64)),
                ("b".to_string(), Value::from(20i64)),
            ])
            .to_value(),
        )]);

        let engine = build_engine(None);
        let payload = Script::try_build(engine, &script.to_value()).unwrap();

        let variable = payload.evaluate_variable(&context).unwrap();
        assert_eq!(variable, Variable::new(Value::from(30i64)));
    }

    #[test]
    fn test_payload_execute_variable_context_params() {
        let script = r#"{{payload.a}}"#;

        let context = HashMap::from([(
            "payload".to_string(),
            HashMap::from([
                ("a".to_string(), Value::from(10i64)),
                ("b".to_string(), Value::from(20i64)),
            ])
            .to_value(),
        )]);

        let engine = build_engine(None);
        let payload = Script::try_build(engine, &script.to_value()).unwrap();

        let variable = payload.evaluate_variable(&context).unwrap();
        assert_eq!(variable, Variable::new(Value::from(10i64)));
    }
}
