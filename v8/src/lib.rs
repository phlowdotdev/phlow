mod payload;
mod variable;
use payload::Payload;
use serde::Serialize;
use std::{collections::HashMap, hash::Hash};
use valu3::{prelude::*, Error as ValueError};

pub type InnerId = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    JsonParseError(ValueError),
    InvalidPipeline(InnerId),
    InvalidCondition,
    InvalidStep(InnerId),
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

    // pub fn execute(&self, context: &Context) {
    //     let left = self.left.execute(context);
    //     let right = self.right.execute(context);

    //     match self.operator {
    //         Operator::Equal => left == right,
    //         Operator::NotEqual => left != right,
    //         Operator::GreaterThan => left > right,
    //         Operator::LessThan => left < right,
    //         Operator::GreaterThanOrEqual => left >= right,
    //         Operator::LessThanOrEqual => left <= right,
    //         Operator::Contains => left.contains(right),
    //         Operator::NotContains => !left.contains(right),
    //         Operator::StartsWith => left.starts_with(right),
    //         Operator::EndsWith => left.ends_with(right),
    //         Operator::Regex => left.matches(right),
    //         Operator::NotRegex => !left.matches(right),
    //     }
    // }
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
