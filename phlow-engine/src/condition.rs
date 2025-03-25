use rhai::Engine;
use serde::Serialize;
use valu3::{prelude::StringBehavior, traits::ToValueBehavior, value::Value};

use crate::{
    context::Context,
    script::{Script, ScriptError},
};

#[derive(Debug)]
pub enum ConditionError {
    InvalidOperator(String),
    RightInvalid(String),
    LeftInvalid(String),
    AssertInvalid(String),
    ScriptError(ScriptError),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Operator {
    Or,
    And,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Regex,
    NotRegex,
}

impl ToValueBehavior for Operator {
    fn to_value(&self) -> Value {
        match self {
            Operator::Or => "or".to_value(),
            Operator::And => "and".to_value(),
            Operator::Equal => "equal".to_value(),
            Operator::NotEqual => "not_equal".to_value(),
            Operator::GreaterThan => "greater_than".to_value(),
            Operator::LessThan => "less_than".to_value(),
            Operator::GreaterThanOrEqual => "greater_than_or_equal".to_value(),
            Operator::LessThanOrEqual => "less_than_or_equal".to_value(),
            Operator::Contains => "contains".to_value(),
            Operator::NotContains => "not_contains".to_value(),
            Operator::StartsWith => "starts_with".to_value(),
            Operator::EndsWith => "ends_with".to_value(),
            Operator::Regex => "regex".to_value(),
            Operator::NotRegex => "not_regex".to_value(),
        }
    }
}

impl From<&Value> for Operator {
    fn from(value: &Value) -> Self {
        match value.as_str() {
            "or" => Operator::Or,
            "and" => Operator::And,
            "equal" => Operator::Equal,
            "not_equal" => Operator::NotEqual,
            "greater_than" => Operator::GreaterThan,
            "less_than" => Operator::LessThan,
            "greater_than_or_equal" => Operator::GreaterThanOrEqual,
            "less_than_or_equal" => Operator::LessThanOrEqual,
            "contains" => Operator::Contains,
            "not_contains" => Operator::NotContains,
            "starts_with" => Operator::StartsWith,
            "ends_with" => Operator::EndsWith,
            "regex" => Operator::Regex,
            "not_regex" => Operator::NotRegex,
            _ => panic!("Invalid operator"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Condition<'a> {
    pub(crate) expression: Script<'a>,
    pub(crate) raw: Value,
}

impl<'a> Condition<'a> {
    pub fn try_from_value(engine: &'a Engine, value: &Value) -> Result<Self, ConditionError> {
        if let Some(assert) = value.get("assert") {
            return Ok(Self::try_build_with_assert(engine, assert.to_string())?);
        }

        let left = match value.get("left") {
            Some(left) => left.to_string(),
            None => return Err(ConditionError::LeftInvalid("does not exist".to_string())),
        };

        let right = match value.get("right") {
            Some(right) => {
                if let Value::String(right) = right {
                    right.to_string()
                } else {
                    right.to_json(valu3::prelude::JsonMode::Inline)
                }
            }
            None => return Err(ConditionError::RightInvalid("does not exist".to_string())),
        };

        let operator = match value.get("operator") {
            Some(operator) => Operator::from(operator),
            None => {
                return Err(ConditionError::InvalidOperator(
                    "does not exist".to_string(),
                ))
            }
        };

        let condition = Self::try_build_with_operator(engine, left, right, operator)?;

        Ok(condition)
    }

    pub fn try_build_with_assert(
        engine: &'a Engine,
        assert: String,
    ) -> Result<Self, ConditionError> {
        let expression =
            Script::try_build(engine, &assert.to_value()).map_err(ConditionError::ScriptError)?;

        Ok(Self {
            expression,
            raw: assert.to_value(),
        })
    }

    pub fn try_build_with_operator(
        engine: &'a Engine,
        left: String,
        right: String,
        operator: Operator,
    ) -> Result<Self, ConditionError> {
        let left = Script::to_code_string(&left);
        let right = Script::to_code_string(&right);

        let assert = {
            match operator {
                Operator::Or => {
                    let query = format!("{{{{{} || {}}}}}", left, right);
                    query
                }
                Operator::And => {
                    let query = format!("{{{{{} && {}}}}}", left, right);
                    query
                }
                Operator::Equal => {
                    let query = format!("{{{{{} == {}}}}}", left, right);
                    query
                }
                Operator::NotEqual => {
                    let query = format!("{{{{{} != {}}}}}", left, right);
                    query
                }
                Operator::GreaterThan => {
                    let query = format!("{{{{{} > {}}}}}", left, right);
                    query
                }
                Operator::LessThan => {
                    let query = format!("{{{{{} < {}}}}}", left, right);
                    query
                }
                Operator::GreaterThanOrEqual => {
                    let query = format!("{{{{{} >= {}}}}}", left, right);
                    query
                }
                Operator::LessThanOrEqual => {
                    let query = format!("{{{{{} <= {}}}}}", left, right);
                    query
                }
                Operator::Contains => {
                    let query = format!("{{{{{} in {}}}}}", right, left);
                    query
                }
                Operator::NotContains => {
                    let query = format!("{{{{!({} in {})}}}}", right, left);
                    query
                }
                Operator::StartsWith => {
                    let query = format!("{{{{{} starts_with {}}}}}", left, right);
                    query
                }
                Operator::EndsWith => {
                    let query = format!("{{{{{} ends_with {}}}}}", left, right);
                    query
                }
                Operator::Regex => {
                    let query = format!("{{{{{} search {}}}}}", left, right);
                    query
                }
                Operator::NotRegex => {
                    let query = format!("{{{{!({} search {})}}}}", left, right);
                    query
                }
            }
        };

        let expression =
            Script::try_build(engine, &assert.to_value()).map_err(ConditionError::ScriptError)?;

        Ok(Self {
            expression,
            raw: assert.to_value(),
        })
    }

    pub fn evaluate(&self, context: &Context) -> Result<bool, ConditionError> {
        let result = self
            .expression
            .evaluate(context)
            .map_err(ConditionError::ScriptError)?;

        match result {
            Value::Boolean(result) => Ok(result),
            _ => Err(ConditionError::ScriptError(ScriptError::InvalidType(
                result,
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::engine::build_engine_async;

    use super::*;

    #[test]
    fn test_condition_execute_equal() {
        let engine = build_engine_async(None);
        let condition = Condition::try_build_with_operator(
            &engine,
            "10".to_string(),
            "20".to_string(),
            Operator::Equal,
        )
        .unwrap();

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_not_equal() {
        let engine = build_engine_async(None);
        let condition = Condition::try_build_with_operator(
            &engine,
            "10".to_string(),
            "20".to_string(),
            Operator::NotEqual,
        )
        .unwrap();

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_greater_than() {
        let engine = build_engine_async(None);
        let condition = Condition::try_build_with_operator(
            &engine,
            "10".to_string(),
            "20".to_string(),
            Operator::GreaterThan,
        )
        .unwrap();

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_contains() {
        let engine = build_engine_async(None);
        let condition = Condition::try_build_with_operator(
            &engine,
            "hello world".to_string(),
            "hello".to_string(),
            Operator::Contains,
        )
        .unwrap();

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_regex() {
        let engine = build_engine_async(None);
        let condition = Condition::try_build_with_operator(
            &engine,
            "hello".to_string(),
            "hello world".to_string(),
            Operator::Regex,
        )
        .unwrap();

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_not_regex() {
        let engine = build_engine_async(None);
        let condition = Condition::try_build_with_operator(
            &engine,
            "hello".to_string(),
            "hello world".to_string(),
            Operator::NotRegex,
        )
        .unwrap();

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_start_with() {
        let engine = build_engine_async(None);
        let condition = Condition::try_build_with_operator(
            &engine,
            "hello world".to_string(),
            "hello".to_string(),
            Operator::StartsWith,
        )
        .unwrap();

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_end_with() {
        let engine = build_engine_async(None);
        let condition = Condition::try_build_with_operator(
            &engine,
            "hello world".to_string(),
            "world".to_string(),
            Operator::EndsWith,
        )
        .unwrap();

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }
}
