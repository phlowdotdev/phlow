use clap::{Arg, Command};

#[derive(Debug)]
pub enum Error {
    ModuleNotFound(String),
}

#[derive(Debug)]
pub struct Cli {
    pub main_path: String,
    pub main_ext: ModuleExtension,
}

impl Cli {
    pub fn load() -> Result<Cli, Error> {
        let matches = Command::new("Phlow Runtime")
            .version("0.1.0")
            .arg(
                Arg::new("main_path")
                    .help("Main path/file to load")
                    .required(false)
                    .index(1),
            )
            .get_matches();

        let (main_file_path, main_ext) = match matches.get_one::<String>("main_path") {
            Some(file) => get_main_file(file)?,
            None => match find_default_file("") {
                Some((file, ext)) => (file, ext),
                None => return Err(Error::ModuleNotFound("main".to_string())),
            },
        };

        Ok(Cli {
            main_path: main_file_path,
            main_ext,
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
