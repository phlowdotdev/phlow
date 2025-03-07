use serde::Serialize;

use crate::{payload::Payload, Context, Error, Operator};

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
