use clap::{Arg, ArgAction, Command};
use std::env;

#[derive(Debug)]
pub enum Error {
    #[allow(dead_code)]
    ModuleNotFound(String),
}
#[derive(Debug)]
pub struct Cli {
    pub main_target: Option<String>,
    pub only_download_modules: bool,
    pub package_path: Option<String>,
    pub no_run: bool,
}

impl Cli {
    pub fn load() -> Result<Cli, Error> {
        let command = Command::new("Phlow Runtime")
            .version(env!("CARGO_PKG_VERSION"))
            .arg(
                Arg::new("main_path")
                    .help("Main path/file to load")
                    .required(false)
                    .index(1),
            )
            .arg(
                Arg::new("install")
                    .long("install")
                    .short('i')
                    .action(ArgAction::SetTrue)
                    .value_parser(clap::builder::BoolishValueParser::new())
                    .help("Install dependencies")
                    .default_value("false"),
            )
            .arg(
                Arg::new("download")
                    .long("download")
                    .short('d')
                    .help("Enable download modules before running")
                    .action(ArgAction::SetTrue)
                    .value_parser(clap::builder::BoolishValueParser::new())
                    .default_value("true"),
            )
            .arg(
                Arg::new("package")
                    .long("package")
                    .help("Path to the package file"),
            )
            .arg(
                Arg::new("steps")
                    .long("show-steps")
                    .help("Show steps")
                    .value_parser(clap::builder::BoolishValueParser::new())
                    .action(ArgAction::SetTrue)
                    .default_value("false"),
            )
            .arg(
                Arg::new("no_run")
                    .long("no-run")
                    .short('n')
                    .help("Do not run the main file")
                    .value_parser(clap::builder::BoolishValueParser::new())
                    .action(ArgAction::SetTrue)
                    .default_value("false"),
            );

        let args: Vec<String> = env::args().collect();
        if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
            let _ = command.clone().print_help();
        }

        let matches = command
            .trailing_var_arg(true)
            .allow_external_subcommands(true)
            .get_matches();

        let main = match matches.get_one::<String>("main_path") {
            Some(target) => Some(target.clone()),
            None => Some(".".to_string()),
        };

        let install = *matches.get_one::<bool>("install").unwrap_or(&false);
        let package_path = matches.get_one::<String>("package").map(|s| s.to_string());

        let no_run = *matches.get_one::<bool>("no_run").unwrap_or(&false);

        Ok(Cli {
            main_target: main,
            only_download_modules: install,
            package_path,
            no_run,
        })
    }
}

#[derive(Debug)]
pub enum ModuleExtension {
    Json,
    Yaml,
    Toml,
}

impl From<&str> for ModuleExtension {
    fn from(extension: &str) -> Self {
        match extension {
            "json" => ModuleExtension::Json,
            "yaml" => ModuleExtension::Yaml,
            "yml" => ModuleExtension::Yaml,
            "toml" => ModuleExtension::Toml,
            _ => ModuleExtension::Json,
        }
    }
}
