mod payload;
mod variable;
use payload::Payload;
use serde::Serialize;
use std::{collections::HashMap, hash::Hash};
use valu3::{prelude::*, Error as ValueError};

pub type InnerId = u32;

#[derive(Debug)]
pub enum Error {
    JsonParseError(ValueError),
    InvalidPipeline(InnerId),
    InvalidCondition,
    InvalidStep(InnerId),
    PayloadError(payload::PayloadError),
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
        let left = self.left.execute(context).map_err(Error::PayloadError)?;
        let right = self.right.execute(context).map_err(Error::PayloadError)?;

        match self.operator {
            Operator::Equal => Ok(left == right),
            Operator::NotEqual => Ok(left != right),
            Operator::GreaterThan => Ok(left > right),
            Operator::LessThan => Ok(left < right),
            Operator::GreaterThanOrEqual => Ok(left >= right),
            Operator::LessThanOrEqual => Ok(left <= right),
            Operator::Contains => {
                if left.is_string() && right.is_string() {
                    let left = left.as_str();
                    let right = right.as_str();

                    return Ok(left.contains(right));
                } else if left.is_array() && right.is_array() {
                    let left = match left.as_array() {
                        Some(array) => array,
                        None => return Err(Error::InvalidCondition),
                    };
                    let right = match right.as_array() {
                        Some(array) => array,
                        None => return Err(Error::InvalidCondition),
                    };

                    return Ok(left.into_iter().any(|x| right.into_iter().any(|y| x == y)));
                }

                Err(Error::InvalidCondition)
            }
            _ => Err(Error::InvalidCondition),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StepType {
    Default,
    ThenCase,
    ElseCase,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    id: Option<String>, // id do json enviado pelo cliente
    name: Option<String>,
    step_type: StepType,
    inner_id: InnerId,
    echo: Option<String>,
    condition: Option<Condition>,
    payload: Option<String>,
    output: Option<Value>,
    then_case: Option<InnerId>,
    else_case: Option<InnerId>,
    return_case: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Context {
    context: Value,
    steps: HashMap<InnerId, Step>,
}

impl Context {
    pub fn new(context: Value) -> Self {
        Self {
            context,
            steps: HashMap::new(),
        }
    }

    pub fn add_step(&mut self, step: Step) {
        self.steps.insert(step.inner_id, step);
    }

    pub fn get_step(&self, inner_id: InnerId) -> Option<&Step> {
        self.steps.get(&inner_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    name: Option<String>,
    id: Option<String>,
    inner_id: InnerId,
    steps: Vec<Step>,
    context: Context,
}

impl Pipeline {
    pub fn new(
        name: Option<String>,
        id: Option<String>,
        inner_id: InnerId,
        context: Context,
    ) -> Self {
        Self {
            name,
            id,
            inner_id,
            steps: Vec::new(),
            context,
        }
    }
}
