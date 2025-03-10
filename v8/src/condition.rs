use serde::Serialize;
use valu3::{prelude::StringBehavior, value::Value};

use crate::{
    payload::{Payload, PayloadError},
    v8::Context,
};

#[derive(Debug)]
pub enum ConditionError {
    InvalidOperator(String),
    RightInvalid(String),
    LeftInvalid(String),
    PayloadError(PayloadError),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Operator {
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
    pub(crate) left: Payload,
    pub(crate) right: Payload,
    pub(crate) operator: Operator,
}

impl TryFrom<&Value> for Condition {
    type Error = ConditionError;

    fn try_from(value: &Value) -> Result<Self, ConditionError> {
        let left = match value.get("left") {
            Some(left) => Payload::from(left),
            None => return Err(ConditionError::LeftInvalid("does not exist".to_string())),
        };

        let right = match value.get("right") {
            Some(right) => Payload::from(right),
            None => return Err(ConditionError::RightInvalid("does not exist".to_string())),
        };

        match value.get("operator") {
            Some(operator) => {
                let operator = Operator::from(operator);

                Ok(Self {
                    left,
                    right,
                    operator,
                })
            }
            None => Err(ConditionError::InvalidOperator(format!(
                "does not exist: {:?}",
                value
            ))),
        }
    }
}

impl Condition {
    pub fn new(left: Payload, right: Payload, operator: Operator) -> Self {
        Self {
            left,
            right,
            operator,
        }
    }

    pub fn evaluate(&self, context: &Context) -> Result<bool, ConditionError> {
        let left = self
            .left
            .evaluate_variable(context)
            .map_err(ConditionError::PayloadError)?;
        let right = self
            .right
            .evaluate_variable(context)
            .map_err(ConditionError::PayloadError)?;

        println!("---------------------------------------------");
        println!("{:?} {:?} {:?}", left, self.operator, right);

        match self.operator {
            Operator::Equal => Ok(left.equal(&right)),
            Operator::NotEqual => Ok(!left.equal(&right)),
            Operator::GreaterThan => Ok(left.greater_than(&right)),
            Operator::LessThan => Ok(left.less_than(&right)),
            Operator::GreaterThanOrEqual => Ok(left.greater_than_or_equal(&right)),
            Operator::LessThanOrEqual => Ok(left.less_than_or_equal(&right)),
            Operator::Contains => Ok(left.contains(&right)),
            Operator::NotContains => Ok(!left.contains(&right)),
            Operator::StartsWith => Ok(left.starts_with(&right)),
            Operator::EndsWith => Ok(left.ends_with(&right)),
            Operator::Regex => Ok(left.regex(&right)),
            Operator::NotRegex => Ok(!left.regex(&right)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use valu3::value::Value;

    #[test]
    fn test_condition_execute_equal() {
        let left = Payload::new("10".to_string());
        let right = Payload::new("20".to_string());
        let condition = Condition::new(left, right, Operator::Equal);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_not_equal() {
        let left = Payload::new("10".to_string());
        let right = Payload::new("20".to_string());
        let condition = Condition::new(left, right, Operator::NotEqual);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_greater_than() {
        let left = Payload::new("10".to_string());
        let right = Payload::new("20".to_string());
        let condition = Condition::new(left, right, Operator::GreaterThan);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_contains() {
        let left = Payload::new(r#""hello world""#.to_string());
        let right = Payload::new(r#""hello""#.to_string());
        let condition = Condition::new(left, right, Operator::Contains);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_regex() {
        let left = Payload::new(r#""hello world""#.to_string());
        let right = Payload::new(r#""hello""#.to_string());
        let condition = Condition::new(left, right, Operator::Regex);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_not_regex() {
        let left = Payload::new(r#""hello world""#.to_string());
        let right = Payload::new(r#""hello""#.to_string());
        let condition = Condition::new(left, right, Operator::NotRegex);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_start_with() {
        let left = Payload::new(r#""hello world""#.to_string());
        let right = Payload::new(r#""hello""#.to_string());
        let condition = Condition::new(left, right, Operator::StartsWith);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_end_with() {
        let left = Payload::new(r#""hello world""#.to_string());
        let right = Payload::new(r#""world""#.to_string());
        let condition = Condition::new(left, right, Operator::EndsWith);

        let context = Context::new(None);

        let result = condition.evaluate(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_from_value() {
        let value = Value::json_to_value(
            r#"{
            "left": 10,
            "right": 20,
            "operator": "greater_than"
        }"#,
        )
        .unwrap();

        let condition = Condition::try_from(&value).unwrap();

        assert_eq!(condition.left, Payload::new("10".to_string()));
        assert_eq!(condition.right, Payload::new("20".to_string()));
        assert_eq!(condition.operator, Operator::GreaterThan);
    }
}
