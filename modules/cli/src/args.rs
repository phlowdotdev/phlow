use phlow_sdk::prelude::*;
use std::{collections::HashMap, env};

#[derive(Debug)]
pub enum Error {
    InvalidInput,
}

#[derive(Debug, PartialEq, Clone)]
pub enum InputType {
    String,
    Number,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub long: Option<String>,
    pub short: Option<String>,
    pub help: String,
    pub default: Option<String>,
    pub required: bool,
    pub index: Option<usize>,
    pub input_type: InputType,
}

#[derive(Debug, Clone)]
pub struct Args {
    pub args: HashMap<String, Value>,
    pub schema: Vec<Arg>,
}

impl Args {
    pub fn run_help(&self) {
        let raw_args: Vec<String> = env::args().skip(1).collect();

        if raw_args.contains(&"--help".to_string()) || raw_args.contains(&"-H".to_string()) {
            println!("Usage:");
            for arg in &self.schema {
                let long = arg.long.as_deref().unwrap_or("");
                let short = arg.short.as_deref().unwrap_or("");
                let name = &arg.name;
                let help = &arg.help;
                let required = if arg.required {
                    "[required]"
                } else {
                    "[optional]"
                };
                let default = arg.default.as_deref().unwrap_or("");

                println!(
                    "  -{} --{} <{}> \t {} {} (default: {})",
                    short, long, name, help, required, default
                );
            }
            std::process::exit(0);
        }
    }
}

impl TryFrom<Value> for Args {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut arg_defs: Vec<Arg> = Vec::new();

        if value.is_null() {
            return Ok(Args {
                args: HashMap::new(),
                schema: Vec::new(),
            });
        }

        for item in value.as_array().ok_or(Error::InvalidInput)? {
            let long = item.get("long").map(Value::to_string);
            let short = item.get("short").map(Value::to_string);
            let help = item.get("help").map(Value::to_string).unwrap_or_default();
            let default = item.get("default").map(Value::to_string).map(String::from);
            let required = *item
                .get("required")
                .and_then(Value::as_bool)
                .unwrap_or(&false);
            let index = item
                .get("index")
                .and_then(Value::to_u64)
                .map(|i| i as usize);
            let input_type = match item.get("type").map(Value::to_string) {
                Some(ref t) if t == "number" => InputType::Number,
                Some(ref t) if t == "boolean" => InputType::Boolean,
                _ => InputType::String,
            };
            let name = item
                .get("name")
                .map(Value::to_string)
                .ok_or(Error::InvalidInput)?;

            arg_defs.push(Arg {
                name,
                long,
                short,
                help,
                default,
                required,
                index,
                input_type,
            });
        }

        let raw_args: Vec<String> = env::args().skip(1).collect();
        let mut parsed_args: HashMap<String, Value> = HashMap::new();

        for arg_def in &arg_defs {
            let mut found: Option<String> = None;

            if let Some(long_flag) = &arg_def.long {
                let flag = format!("--{}", long_flag);
                if let Some(pos) = raw_args.iter().position(|a| a == &flag) {
                    if arg_def.input_type == InputType::Boolean {
                        let next = raw_args.get(pos + 1);
                        if next.is_none() || next.unwrap().starts_with('-') {
                            found = Some("".to_string());
                        } else {
                            found = Some(next.unwrap().to_string());
                        }
                    } else {
                        found = raw_args.get(pos + 1).cloned();
                    }
                }
            }

            if found.is_none() {
                if let Some(short_flag) = &arg_def.short {
                    let flag = format!("-{}", short_flag);
                    if let Some(pos) = raw_args.iter().position(|a| a == &flag) {
                        if arg_def.input_type == InputType::Boolean {
                            let next = raw_args.get(pos + 1);
                            if next.is_none() || next.unwrap().starts_with('-') {
                                found = Some("".to_string());
                            } else {
                                found = Some(next.unwrap().to_string());
                            }
                        } else {
                            found = raw_args.get(pos + 1).cloned();
                        }
                    }
                }
            }

            if found.is_none() {
                if let Some(idx) = arg_def.index {
                    if raw_args.len() > idx {
                        found = Some(raw_args[idx].clone());
                    }
                }
            }

            if found.is_none() {
                found = arg_def.default.clone();
            }

            if found.is_none() && arg_def.required {
                return Err(Error::InvalidInput);
            }

            if let Some(value_str) = found {
                let value = match arg_def.input_type {
                    InputType::String => Value::from(value_str),
                    InputType::Number => value_str
                        .parse::<f64>()
                        .map(Value::from)
                        .map_err(|_| Error::InvalidInput)?,
                    InputType::Boolean => {
                        let v = match value_str.as_str() {
                            "" => Value::Boolean(true), // sinalizador sem valor
                            "true" | "1" => Value::Boolean(true),
                            "false" | "0" => Value::Boolean(false),
                            _ => return Err(Error::InvalidInput),
                        };
                        v
                    }
                };
                parsed_args.insert(arg_def.name.clone(), value);
            }
        }

        Ok(Args {
            args: parsed_args,
            schema: arg_defs,
        })
    }
}
