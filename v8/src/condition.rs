use serde::Serialize;
use valu3::{prelude::StringBehavior, value::Value};

use crate::{
    script::{Script, ScriptError},
    v8::Context,
};

#[derive(Debug)]
pub enum ConditionError {
    InvalidOperator(String),
    RightInvalid(String),
    LeftInvalid(String),
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Condition {
    pub(crate) expression: Script,
}

impl Condition {
    pub fn new(left: String, right: String, operator: Operator) -> Self {
        let expression = {
            match operator {
                Operator::Or => {
                    let query = format!("{} || {}", left, right);
                    query
                }
                Operator::And => {
                    let query = format!("{} && {}", left, right);
                    query
                }
                Operator::Equal => {
                    let query = format!("{} == {}", left, right);
                    query
                }
                Operator::NotEqual => {
                    let query = format!("{} != {}", left, right);
                    query
                }
                Operator::GreaterThan => {
                    let query = format!("{} > {}", left, right);
                    query
                }
                Operator::LessThan => {
                    let query = format!("{} < {}", left, right);
                    query
                }
                Operator::GreaterThanOrEqual => {
                    let query = format!("{} >= {}", left, right);
                    query
                }
                Operator::LessThanOrEqual => {
                    let query = format!("{} <= {}", left, right);
                    query
                }
                Operator::Contains => {
                    let query = format!("({} in {})", left, right);
                    query
                }
                Operator::NotContains => {
                    let query = format!("{} !in {}", left, right);
                    query
                }
                Operator::StartsWith => {
                    let query = format!("{} starts_with {}", left, right);
                    query
                }
                Operator::EndsWith => {
                    let query = format!("{} ends_with {}", left, right);
                    query
                }
                Operator::Regex => {
                    let query = format!("{} search {}", left, right);
                    query
                }
                Operator::NotRegex => {
                    let query = format!("!({} search {})", left, right);
                    query
                }
            }
        };

        Self {
            expression: Script::from(expression),
        }
    }

    fn remove_quotes(script: &str) -> String {
        let mut script = script.to_string();
        script.pop();
        script.remove(0);
        script
    }

    pub fn evaluate(&self, context: &Context) -> Result<bool, ConditionError> {
        let result = self
            .expression
            .evaluate(context)
            .map_err(ConditionError::ScriptError)?;

        match result {
            Value::Boolean(result) => Ok(result),
            _ => Err(ConditionError::ScriptError(ScriptError::InvalidType)),
        }
    }
}

impl From<(String, String, Operator)> for Condition {
    fn from((left, right, operator): (String, String, Operator)) -> Self {
        Self::new(left, right, operator)
    }
}

impl TryFrom<&Value> for Condition {
    type Error = ConditionError;

    fn try_from(value: &Value) -> Result<Self, ConditionError> {
        let left = match value.get("left") {
            Some(left) => {
                if let Value::String(left) = left {
                    left.to_string()
                } else {
                    left.to_json(valu3::prelude::JsonMode::Inline)
                }
            }
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

        Ok(Self::new(left, right, operator))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_condition_execute_equal() {
        let condition = Condition::new("10".to_string(), "20".to_string(), Operator::Equal);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_not_equal() {
        let condition = Condition::new("10".to_string(), "20".to_string(), Operator::NotEqual);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_greater_than() {
        let condition = Condition::new("10".to_string(), "20".to_string(), Operator::GreaterThan);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_contains() {
        let condition = Condition::new(
            r#""hello""#.to_string(),
            r#""hello world""#.to_string(),
            Operator::Contains,
        );

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_regex() {
        let condition = Condition::new(
            // regex find "hello" in "hello world"
            r#""hello""#.to_string(),
            r#""hello world""#.to_string(),
            Operator::Regex,
        );

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_not_regex() {
        let condition = Condition::new(
            r#""hello""#.to_string(),
            r#""hello world""#.to_string(),
            Operator::NotRegex,
        );

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_start_with() {
        let condition = Condition::new(
            r#""hello world""#.to_string(),
            r#""hello""#.to_string(),
            Operator::StartsWith,
        );

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_end_with() {
        let condition = Condition::new(
            r#""hello world""#.to_string(),
            r#""world""#.to_string(),
            Operator::EndsWith,
        );

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }
}
