use clap::{Arg, ArgAction, Command};
use std::env;

#[derive(Debug)]
pub enum Error {
    #[allow(dead_code)]
    ModuleNotFound(String),
}

#[derive(Debug)]
pub struct MainArgs {
    pub path: String,
    pub ext: ModuleExtension,
}

#[derive(Debug)]
pub struct Cli {
    pub main: Option<MainArgs>,
    pub only_download_modules: bool,
    pub package_path: Option<String>,
    pub no_run: bool,
}

impl Cli {
    pub fn load() -> Result<Cli, Error> {
        let command = Command::new("Phlow Runtime")
            .version("0.1.0")
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
            Some(file) => {
                let (path, ext) = get_main_file(file)?;
                Some(MainArgs { path, ext })
            }
            None => match find_default_file("") {
                Some((path, ext)) => Some(MainArgs { path, ext }),
                None => None,
            },
        };

        let install = *matches.get_one::<bool>("install").unwrap_or(&false);
        let package_path = matches.get_one::<String>("package").map(|s| s.to_string());

        let no_run = *matches.get_one::<bool>("no_run").unwrap_or(&false);

        Ok(Cli {
            main,
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

fn get_main_file(main_path: &str) -> Result<(String, ModuleExtension), Error> {
    let path = std::path::Path::new(&main_path);
    if path.is_dir() {
        let file = find_default_file(&main_path);
        match file {
            Some(data) => return Ok(data),
            None => return Err(Error::ModuleNotFound("main".to_string())),
        }
    }

    if path.exists() {
        let extension = match main_path.split('.').last() {
            Some(extension) => extension,
            None => return Err(Error::ModuleNotFound(main_path.to_string())),
        };
        return Ok((main_path.to_string(), ModuleExtension::from(extension)));
    }

    Err(Error::ModuleNotFound(main_path.to_string()))
}

fn find_default_file(base: &str) -> Option<(String, ModuleExtension)> {
    let files = vec!["main.yaml", "main.yml", "main.json", "main.toml"];

    for file in files {
        let path = if base.is_empty() || base == "." {
            file.to_string()
        } else {
            format!("{}/{}", base, file)
        };
        if std::path::Path::new(&path).exists() {
            let extension = match file.split('.').last() {
                Some(extension) => extension,
                None => return None,
            };
            return Some((path.to_string(), ModuleExtension::from(extension)));
        }
    }

    None
}
