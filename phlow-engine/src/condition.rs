use std::{fmt::Display, sync::Arc};

use rhai::Engine;
use serde::Serialize;
use valu3::{prelude::StringBehavior, traits::ToValueBehavior, value::Value};

use crate::{context::Context, script::Script};

#[derive(Debug)]
pub enum ConditionError {
    InvalidOperator(String),
    RightInvalid(String),
    LeftInvalid(String),
    AssertInvalid(String),
    ScriptError(phs::ScriptError),
}

impl Display for ConditionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionError::InvalidOperator(err) => write!(f, "Invalid operator: {}", err),
            ConditionError::RightInvalid(err) => write!(f, "Right invalid: {}", err),
            ConditionError::LeftInvalid(err) => write!(f, "Left invalid: {}", err),
            ConditionError::AssertInvalid(err) => write!(f, "Assert invalid: {}", err),
            ConditionError::ScriptError(err) => write!(f, "Script error: {}", err),
        }
    }
}

impl std::error::Error for ConditionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConditionError::InvalidOperator(_) => None,
            ConditionError::RightInvalid(_) => None,
            ConditionError::LeftInvalid(_) => None,
            ConditionError::AssertInvalid(_) => None,
            ConditionError::ScriptError(_) => None, // ScriptError doesn't implement std::error::Error
        }
    }
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
            _ => panic!("Invalid operator"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub(crate) expression: Script,
    pub(crate) raw: Value,
}

impl Condition {
    pub fn try_from_value(engine: Arc<Engine>, value: &Value) -> Result<Self, ConditionError> {
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
        engine: Arc<Engine>,
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
        engine: Arc<Engine>,
        left: String,
        right: String,
        operator: Operator,
    ) -> Result<Self, ConditionError> {
        let left = phs::Script::to_code_string(&left);
        let right = phs::Script::to_code_string(&right);

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
            _ => Err(ConditionError::ScriptError(phs::ScriptError::InvalidType(
                result,
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use phs::build_engine;

    #[test]
    fn test_condition_execute_equal() {
        let engine = build_engine(None);
        let condition = Condition::try_build_with_operator(
            engine,
            "10".to_string(),
            "20".to_string(),
            Operator::Equal,
        )
        .unwrap();

        let context = Context::new();

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_not_equal() {
        let engine = build_engine(None);
        let condition = Condition::try_build_with_operator(
            engine,
            "10".to_string(),
            "20".to_string(),
            Operator::NotEqual,
        )
        .unwrap();

        let context = Context::new();

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_greater_than() {
        let engine = build_engine(None);
        let condition = Condition::try_build_with_operator(
            engine,
            "10".to_string(),
            "20".to_string(),
            Operator::GreaterThan,
        )
        .unwrap();

        let context = Context::new();

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_contains() {
        let engine = build_engine(None);
        let condition = Condition::try_build_with_operator(
            engine,
            "hello world".to_string(),
            "hello".to_string(),
            Operator::Contains,
        )
        .unwrap();

        let context = Context::new();

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }
}
