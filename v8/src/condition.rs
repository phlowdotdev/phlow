use serde::Serialize;

use crate::{payload::Payload, pipeline::Context, Error};

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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Condition {
    left: Payload,
    right: Payload,
    operator: Operator,
}

impl Condition {
    pub fn new(left: Payload, right: Payload, operator: Operator) -> Self {
        Self {
            left,
            right,
            operator,
        }
    }

    pub fn execute(&self, context: &Context) -> Result<bool, Error> {
        let left = self
            .left
            .execute_variable(context)
            .map_err(Error::PayloadError)?;
        let right = self
            .right
            .execute_variable(context)
            .map_err(Error::PayloadError)?;

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

        let context = Context::new(Value::Null);

        let result = condition.execute(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_not_equal() {
        let left = Payload::new("10".to_string());
        let right = Payload::new("20".to_string());
        let condition = Condition::new(left, right, Operator::NotEqual);

        let context = Context::new(Value::Null);

        let result = condition.execute(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_greater_than() {
        let left = Payload::new("10".to_string());
        let right = Payload::new("20".to_string());
        let condition = Condition::new(left, right, Operator::GreaterThan);

        let context = Context::new(Value::Null);

        let result = condition.execute(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_contains() {
        let left = Payload::new("hello world".to_string());
        let right = Payload::new("world".to_string());
        let condition = Condition::new(left, right, Operator::Contains);

        let context = Context::new(Value::Null);

        let result = condition.execute(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_regex() {
        let left = Payload::new("hello world".to_string());
        let right = Payload::new("world".to_string());
        let condition = Condition::new(left, right, Operator::Regex);

        let context = Context::new(Value::Null);

        let result = condition.execute(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_not_regex() {
        let left = Payload::new("hello world".to_string());
        let right = Payload::new("world".to_string());
        let condition = Condition::new(left, right, Operator::NotRegex);

        let context = Context::new(Value::Null);

        let result = condition.execute(&context).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_condition_execute_start_with() {
        let left = Payload::new("hello world".to_string());
        let right = Payload::new("hello".to_string());
        let condition = Condition::new(left, right, Operator::StartsWith);

        let context = Context::new(Value::Null);

        let result = condition.execute(&context).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_condition_execute_end_with() {
        let left = Payload::new("hello world".to_string());
        let right = Payload::new("world".to_string());
        let condition = Condition::new(left, right, Operator::EndsWith);

        let context = Context::new(Value::Null);

        let result = condition.execute(&context).unwrap();
        assert_eq!(result, true);
    }
}
