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
    pub download: bool,
    pub print_yaml: bool,
    pub test: bool,
    pub test_filter: Option<String>,
    pub var_main: Option<String>,
    // analyzer options
    pub analyzer: bool,
    pub analyzer_files: bool,
    pub analyzer_modules: bool,
    pub analyzer_total_steps: bool,
    pub analyzer_total_pipelines: bool,
    pub analyzer_json: bool,
    pub analyzer_inner: bool,
    pub analyzer_all: bool,
}

impl Cli {
    pub fn load() -> Result<Cli, Error> {
        let command = Command::new("Phlow Runtime")
            .version(env!("CARGO_PKG_VERSION"))
            .arg(
                Arg::new("main_path")
                    .help("Main path/file to load")
                    .required(false)
                    .index(1)
                    .num_args(1..), // <= Aqui adiciona múltiplos valores possíveis
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
                    .value_parser(clap::builder::BoolValueParser::new())
                    .default_value("true"),
            )
            .arg(
                Arg::new("package")
                    .long("package")
                    .help("Path to the package file"),
            )
            .arg(
                Arg::new("no_run")
                    .long("no-run")
                    .short('n')
                    .help("Do not run the main file")
                    .value_parser(clap::builder::BoolishValueParser::new())
                    .action(ArgAction::SetTrue)
                    .default_value("false"),
            )
            .arg(
                Arg::new("print_yaml")
                    .long("print")
                    .short('p')
                    .help("Print the YAML file generated from the target file")
                    .value_parser(clap::builder::BoolishValueParser::new())
                    .action(ArgAction::SetTrue)
                    .default_value("false"),
            )
            .arg(
                Arg::new("test")
                    .long("test")
                    .short('t')
                    .help("Run tests defined in the phlow file")
                    .value_parser(clap::builder::BoolishValueParser::new())
                    .action(ArgAction::SetTrue)
                    .default_value("false"),
            )
            .arg(
                Arg::new("test_filter")
                    .long("test-filter")
                    .help("Filter tests by description (substring match)")
                    .requires("test")
                    .value_name("FILTER"),
            )
            .arg(
                Arg::new("var_main")
                    .long("var-main")
                    .help("Set the main variable value (simulates main module output)")
                    .value_name("VALUE"),
            );

        // Analyzer flags
        let command = command
            .arg(
                Arg::new("analyzer")
                    .long("analyzer")
                    .help("Run analyzer on the .phlow and return info without executing the runtime")
                    .action(ArgAction::SetTrue)
                    .value_parser(clap::builder::BoolishValueParser::new())
                    .default_value("false"),
            )
            .arg(
                Arg::new("files")
                    .long("files")
                    .help("Return all files loaded by the project, including files referenced by !include or !import")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("modules")
                    .long("modules")
                    .help("Return modules declared in the project and whether they are downloaded in phlow_packages")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("total_steps")
                    .long("total-steps")
                    .help("Return total steps in the project")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("total_pipelines")
                    .long("total-pipelines")
                    .help("Return total pipelines in the project")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("json")
                    .long("json")
                    .help("Output analyzer data as JSON")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("text")
                    .long("text")
                    .help("Output analyzer data as plain text (default)")
                    .action(ArgAction::SetTrue),
            );

        // analyzer --inner: enable recursive analysis/loading of internal modules (./module)
        let command = command.arg(
            Arg::new("inner")
                .long("inner")
                .help("When used with analyzer, allow analysis/recursive loading of internal modules (those declared with a leading '.')")
                .action(ArgAction::SetTrue),
        );

        // analyzer --all: print everything available
        let command = command.arg(
            Arg::new("all")
                .long("all")
                .help("When used with --analyzer, return all available analyzer information")
                .action(ArgAction::SetTrue),
        );

        let matches = command.get_matches();

        let main = match matches.get_one::<String>("main_path") {
            Some(target) => Some(target.clone()),
            None => None,
        };

        let install = *matches.get_one::<bool>("install").unwrap_or(&false);
        let package_path = matches.get_one::<String>("package").map(|s| s.to_string());

        let no_run = *matches.get_one::<bool>("no_run").unwrap_or(&false);

        let download = *matches.get_one::<bool>("download").unwrap_or(&true);

        let print_yaml = *matches.get_one::<bool>("print_yaml").unwrap_or(&false);

        let test = *matches.get_one::<bool>("test").unwrap_or(&false);

        let test_filter = matches
            .get_one::<String>("test_filter")
            .map(|s| s.to_string());

        let var_main = matches.get_one::<String>("var_main").map(|s| s.to_string());

        let analyzer = *matches.get_one::<bool>("analyzer").unwrap_or(&false);
        let analyzer_files = *matches.get_one::<bool>("files").unwrap_or(&false);
        let analyzer_modules = *matches.get_one::<bool>("modules").unwrap_or(&false);
        let analyzer_total_steps = *matches.get_one::<bool>("total_steps").unwrap_or(&false);
        let analyzer_total_pipelines =
            *matches.get_one::<bool>("total_pipelines").unwrap_or(&false);
        let analyzer_json = *matches.get_one::<bool>("json").unwrap_or(&false);
        let analyzer_inner = *matches.get_one::<bool>("inner").unwrap_or(&false);
        let analyzer_all = *matches.get_one::<bool>("all").unwrap_or(&false);

        Ok(Cli {
            main_target: main,
            only_download_modules: install,
            package_path,
            no_run,
            download,
            print_yaml,
            test,
            test_filter,
            var_main,
            analyzer,
            analyzer_files,
            analyzer_modules,
            analyzer_total_steps,
            analyzer_total_pipelines,
            analyzer_json,
            analyzer_inner,
            analyzer_all,
        })
    }
}
