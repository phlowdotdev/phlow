use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, bail, Context, Result};
use phlow_sdk::tracing::info;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug)]
pub struct Publish {
    pub module_dir: PathBuf,
}

#[derive(Deserialize)]
struct ModuleMetadata {
    name: String,
    version: String,
    repository: String,
    license: String,
    author: String,
}

impl Publish {
    pub fn run(&self, default_package_repository_url: &str) -> Result<()> {
        let archive_name = self.create_package().with_context(|| {
            format!("Failed to create package in {}", self.module_dir.display())
        })?;
        let archive_path = PathBuf::from(archive_name);

        self.organize_package(archive_path.as_path())
            .with_context(|| format!("Failed to organize package: {}", archive_path.display()))?;

        Ok(())
    }

    pub fn create_package(&self) -> Result<String> {
        let release_dir = PathBuf::from("target/release");

        info!(
            "Searching for metadata file in: {}",
            self.module_dir.display()
        );

        let metadata_path = ["phlow.yaml", "phlow.yml", "phlow.json"]
            .iter()
            .map(|f| self.module_dir.join(f))
            .find(|p| p.exists())
            .ok_or_else(|| {
                anyhow!(
                    "No phlow.yaml/yml/json file found in {}",
                    self.module_dir.display()
                )
            })?;

        info!("Metadata file found: {}", metadata_path.display());

        let metadata: ModuleMetadata = match metadata_path.extension().and_then(|ext| ext.to_str())
        {
            Some("json") => {
                info!("Reading metadata as JSON");
                serde_json::from_reader(File::open(&metadata_path)?).with_context(|| {
                    format!("Failed to parse JSON file: {}", metadata_path.display())
                })?
            }
            Some("toml") => {
                info!("Reading metadata as TOML");
                let content = fs::read_to_string(&metadata_path)?;
                toml::de::from_str(&content).with_context(|| {
                    format!("Failed to parse TOML file: {}", metadata_path.display())
                })?
            }
            _ => {
                info!("Reading metadata as YAML");
                let content = fs::read_to_string(&metadata_path)?;
                serde_yaml::from_str(&content).with_context(|| {
                    format!("Failed to parse YAML file: {}", metadata_path.display())
                })?
            }
        };

        info!("Metadata loaded:\n  - name: {}\n  - version: {}\n  - repository: {}\n  - license: {}\n  - author: {}",
            metadata.name, metadata.version, metadata.repository, metadata.license, metadata.author);

        info!("Validating version...");
        let version_regex = Regex::new(r"^\d+\.\d+\.\d+(?:-[\w\.-]+)?(?:\+[\w\.-]+)?$")?;
        if !version_regex.is_match(&metadata.version) {
            bail!("Invalid version: must follow MAJOR.MINOR.PATCH-prerelease+build format");
        }

        info!("Validating author...");
        let author_regex = Regex::new(r"^.+ <.+@.+>$")?;
        if !author_regex.is_match(&metadata.author) {
            bail!("Invalid author: must follow the pattern 'name <email>'");
        }

        info!("Validating license...");
        let known_licenses = [
            "MIT",
            "Apache-2.0",
            "GPL-3.0",
            "BSD-3-Clause",
            "MPL-2.0",
            "LGPL-3.0",
            "CDDL-1.0",
            "EPL-2.0",
            "Unlicense",
        ];
        if !known_licenses.contains(&metadata.license.as_str()) {
            if !metadata.license.starts_with("http://") && !metadata.license.starts_with("https://")
            {
                bail!("Invalid license: must be a known open source license or a URL to license terms");
            }
        }

        info!("Starting project build...");
        Command::new("cargo")
            .args(["build", "--release", "--locked"])
            .status()
            .context("Failed to run cargo build")?
            .success()
            .then_some(())
            .context("Build failed")?;

        let temp_dir = PathBuf::from(format!(".tmp/{}", metadata.name));
        info!("Creating temporary directory: {}", temp_dir.display());
        fs::create_dir_all(&temp_dir)?;

        let so_name = format!("lib{}.so", metadata.name);
        let so_path = release_dir.join(&so_name);
        if !so_path.exists() {
            bail!("Missing .so file: {}", so_path.display());
        }
        info!(
            "Copying .so file from {} to {}",
            so_path.display(),
            temp_dir.display()
        );
        fs::copy(&so_path, temp_dir.join("module.so"))?;

        info!("Copying metadata file to temp folder");
        fs::copy(
            &metadata_path,
            temp_dir.join(metadata_path.file_name().unwrap()),
        )?;

        let archive_name = format!("{}-{}.tar.gz", metadata.name, metadata.version);

        info!("Creating archive: {}", archive_name);
        let status = Command::new("tar")
            .args(["-czf", &archive_name, "-C"])
            .arg(temp_dir.to_str().unwrap()) // entra direto na pasta criada, ex: .tmp/nome
            .arg(".") // empacota apenas o conteúdo, sem incluir a pasta
            .status()
            .context("Failed to create archive")?;

        if !status.success() {
            bail!("Failed to generate package: {}", archive_name);
        }

        info!("Success! Package created: {} 🎉", archive_name);

        info!("Cleaning up temporary directory: {}", temp_dir.display());
        fs::remove_dir_all(&temp_dir).with_context(|| {
            format!(
                "Failed to remove temporary directory: {}",
                temp_dir.display()
            )
        })?;

        Ok(archive_name)
    }

    pub fn organize_package(&self, archive_path: &Path) -> Result<()> {
        let filename = archive_path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid archive file name"))?;

        let base_name = filename.trim_end_matches(".tar.gz");
        let parts: Vec<&str> = base_name.rsplitn(2, '-').collect();
        if parts.len() != 2 {
            anyhow::bail!("Filename does not follow expected format: name-version.tar.gz");
        }

        let version = parts[0];
        let package_name = parts[1];
        let padded = format!("{:<4}", package_name).replace(' ', "_");

        let prefix = &padded[0..2];
        let middle = &padded[2..4];
        let final_path = PathBuf::from(format!(
            ".publish-{}_{}/packages/{}/{}/{}",
            package_name,
            { middle },
            prefix,
            middle,
            package_name
        ));
        fs::create_dir_all(&final_path)?;

        // Atualiza index.json
        let index_file = final_path.join("index.json");
        let new_entry = json!({
            "name": package_name,
            "version": version,
            "repository": "https://github.com/lowcarboncode/phlow-packages"
        });

        if index_file.exists() {
            let mut entries: Vec<serde_json::Value> =
                serde_json::from_reader(File::open(&index_file)?)?;
            entries.push(new_entry);
            fs::write(&index_file, serde_json::to_vec_pretty(&entries)?)?;
        } else {
            let entries = vec![new_entry];
            fs::write(&index_file, serde_json::to_vec_pretty(&entries)?)?;
        }

        // Caminho de destino em packages
        let package_dest = final_path.join(filename);
        fs::rename(archive_path, &package_dest)?;

        Ok(())
    }
}

impl TryFrom<String> for Publish {
    type Error = anyhow::Error;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        let module_dir = PathBuf::from(&path);
        if !module_dir.exists() {
            bail!("Directory not found: {}", module_dir.display());
        }
        println!(
            "[INFO] Initializing Publish struct for directory: {}",
            module_dir.display()
        );
        Ok(Publish { module_dir })
    }
}
