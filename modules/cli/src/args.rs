use colored::*;
use phlow_sdk::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    env,
};

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
    pub app_data: ApplicationData,
    pub error: Vec<String>,
}

impl Args {
    pub fn new(value: Value, app_data: ApplicationData) -> Self {
        let mut arg_defs: Vec<Arg> = Vec::new();
        let mut error = Vec::new();

        if value.is_null() {
            return Self {
                args: HashMap::new(),
                schema: Vec::new(),
                app_data,
                error,
            };
        }

        let array = match value.as_array() {
            Some(arr) => arr.clone(),
            None => {
                error.push("Invalid input: schema must be an array.".to_string());
                return Self {
                    args: HashMap::new(),
                    schema: Vec::new(),
                    app_data,
                    error,
                };
            }
        };

        for item in array {
            let name = match item.get("name").map(Value::to_string) {
                Some(n) => n,
                None => {
                    error.push("Missing required 'name' field in schema.".to_string());
                    continue;
                }
            };

            let long = item.get("long").map(Value::to_string);
            let short = item.get("short").map(Value::to_string);
            let help = item.get("help").map(Value::to_string).unwrap_or_default();
            let default = item.get("default").map(Value::to_string).map(String::from);
            let required_flag = *item
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

            arg_defs.push(Arg {
                name,
                long,
                short,
                help,
                default,
                required: required_flag,
                index,
                input_type,
            });
        }

        let raw_args: Vec<String> = env::args().skip(1).collect();
        let mut parsed_args: HashMap<String, Value> = HashMap::new();

        // 🔍 Construir conjunto de flags válidas
        let mut valid_flags: HashSet<String> = HashSet::new();
        for arg in &arg_defs {
            if let Some(l) = &arg.long {
                valid_flags.insert(format!("--{}", l));
            }
            if let Some(s) = &arg.short {
                valid_flags.insert(format!("-{}", s));
            }
        }

        let mut error = Vec::new();
        for (_, raw) in raw_args.iter().enumerate() {
            if raw.starts_with('-') {
                // Se for "--flag=valor", considera apenas a flag
                let flag = raw.split('=').next().unwrap_or(raw);
                if !valid_flags.contains(flag) && flag != "--help" && flag != "-H" && flag != "-h" {
                    error.push(format!(
                        "Unknown flag: {}. Use --help to see the available flags.",
                        flag
                    ));
                }
            }
        }

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
                        let value = raw_args[idx].clone();
                        if value.starts_with('-') {
                            error.push(format!(
                                "Invalid value for positional argument {}: cannot start with '-' or '--'. Found '{}'",
                                arg_def.name, value
                            ));
                        } else {
                            found = Some(value);
                        }
                    }
                }
            }

            if found.is_none() {
                found = arg_def.default.clone();
            }

            if found.is_none() && arg_def.required {
                error.push(format!("Missing required argument: {}", arg_def.name));
            }

            if let Some(value_str) = found {
                let value = match arg_def.input_type {
                    InputType::String => Value::from(value_str),
                    InputType::Number => match value_str.parse::<f64>().map(Value::from) {
                        Ok(v) => v,
                        Err(_) => {
                            error
                                .push(format!("Invalid value for {}: {}", arg_def.name, value_str));

                            Value::Null
                        }
                    },
                    InputType::Boolean => {
                        let v = match value_str.as_str() {
                            "" => Value::Boolean(true),
                            "true" | "1" => Value::Boolean(true),
                            "false" | "0" => Value::Boolean(false),
                            _ => {
                                error.push(format!(
                                    "Invalid value for {}: {}",
                                    arg_def.name, value_str
                                ));
                                Value::Null
                            }
                        };
                        v
                    }
                };
                parsed_args.insert(arg_def.name.clone(), value);
            }
        }

        Self {
            args: parsed_args,
            schema: arg_defs,
            app_data,
            error,
        }
    }

    pub fn is_help(&self) -> bool {
        let raw_args: Vec<String> = env::args().skip(1).collect();

        raw_args.contains(&"--help".to_string())
            || raw_args.contains(&"-h".to_string())
            || raw_args.contains(&"-H".to_string())
    }

    pub fn print_help(self, extra: Option<String>) {
        println!(
            "{}: {}",
            "Usage".bold().underline(),
            self.app_data
                .name
                .unwrap_or("Phlow Cli".to_string())
                .bold()
                .blue()
        );

        if let Some(version) = self.app_data.version {
            println!("       {}: {}", "Version", version);
        }

        if let Some(description) = self.app_data.description {
            println!("       {}: {}", "Description", description);
        }

        if let Some(license) = self.app_data.license {
            println!("       {}: {}", "License", license);
        }

        if let Some(author) = self.app_data.author {
            println!("       {}: {}", "Author", author);
        }

        if let Some(homepage) = self.app_data.homepage {
            println!("       {}: {}", "Homepage", homepage);
        }

        if let Some(repository) = self.app_data.repository {
            println!("       {}: {}", "Repository", repository);
        }

        println!("");

        if let Some(extra) = extra {
            println!("{}", extra);
        }

        let mut arguments = Vec::new();
        let mut options = Vec::new();

        for arg in &self.schema {
            let long = arg.long.as_deref().unwrap_or("");
            let short = arg.short.as_deref().unwrap_or("");
            let name = &arg.name;
            let help = &arg.help;
            let required = if arg.required {
                format!("{}", "[required]".yellow().bold())
            } else {
                "[optional]".to_string()
            };
            let default = arg.default.as_deref().unwrap_or("");

            if arg.index.is_some() {
                arguments.push(format!("{} {} {} {}", name.bold(), required, default, help));
            } else {
                let mut option = String::new();

                if !long.is_empty() {
                    option.push_str(&format!("--{}", long));
                }

                if !short.is_empty() {
                    option.push_str(&format!(", -{}", short));
                }

                option.push_str(format!(" {} {} {}", name.bold(), required, default).as_str());
                options.push(format!("{} {}", option, help));
            }
        }

        if !arguments.is_empty() {
            println!("{}:", "Arguments".bold().underline());
            for arg in arguments {
                println!("  {}", arg);
            }
            println!();
        }

        if !options.is_empty() {
            println!("{}:", "Options".bold().underline());
            for opt in options {
                println!("  {}", opt);
            }
            println!();
        }

        std::process::exit(0);
    }

    pub fn is_error(&self) -> bool {
        !self.error.is_empty()
    }

    pub fn print_error_with_help(self) {
        let errors = self
            .error
            .iter()
            .map(|err| format!("{}", err.bold().red().underline()))
            .collect::<Vec<_>>()
            .join("\n       ");

        let err = format!("{}\n       {}\n", "Error:".red().bold(), errors);

        self.print_help(Some(err));
    }
}
