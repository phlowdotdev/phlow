use crate::MODULE_EXTENSION;
use anyhow::{Context, Result, anyhow, bail};
use log::info;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};

#[derive(Debug)]
pub struct Package {
    pub module_dir: PathBuf,
    pub package_target: String,
    pub create_tar: bool,
}

#[derive(Deserialize, Serialize)]
struct ModuleMetadata {
    name: String,
    version: String,
}

impl Package {
    pub fn new(module_dir: PathBuf, package_target: String, create_tar: bool) -> Result<Self> {
        if !module_dir.exists() {
            bail!("Directory not found: {}", module_dir.display());
        }

        info!(
            "Initializing Package struct for directory: {}",
            module_dir.display()
        );
        Ok(Package {
            module_dir,
            package_target,
            create_tar,
        })
    }

    pub fn run(&self) -> Result<()> {
        let archive_name = self.create_package().with_context(|| {
            format!("Failed to create package in {}", self.module_dir.display())
        })?;

        info!("Package created: {}", archive_name);
        Ok(())
    }

    fn create_package(&self) -> Result<String> {
        let release_dir = PathBuf::from("target/release");

        info!(
            "Searching for metadata file in: {}",
            self.module_dir.display()
        );

        let metadata_path = ["main.phlow", "phlow.yaml", "phlow.yml"]
            .iter()
            .map(|f| self.module_dir.join(f))
            .find(|p| p.exists())
            .ok_or_else(|| anyhow!("No main.phlow file found in {}", self.module_dir.display()))?;

        info!("Metadata file found: {}", metadata_path.display());

        let metadata: ModuleMetadata = {
            let content = fs::read_to_string(&metadata_path)?;
            serde_yaml::from_str(&content).with_context(|| {
                format!("Failed to parse YAML file: {}", metadata_path.display())
            })?
        };

        info!(
            "Metadata loaded:\n  - name: {}\n  - version: {}\n ",
            metadata.name, metadata.version
        );

        info!("Validating version...");
        let version_regex = Regex::new(r"^\d+\.\d+\.\d+(?:-[\w\.-]+)?(?:\+[\w\.-]+)?$")?;
        if !version_regex.is_match(&metadata.version) {
            bail!("Invalid version: must follow MAJOR.MINOR.PATCH-prerelease+build format");
        }

        info!("Starting project build...");
        Command::new("cargo")
            .args(["build", "--release", "--locked"])
            .status()
            .context("Failed to run cargo build")?
            .success()
            .then_some(())
            .context("Build failed")?;

        let so_name = format!("lib{}.{}", metadata.name, MODULE_EXTENSION);
        let so_path = release_dir.join(&so_name);
        if !so_path.exists() {
            bail!("Missing .{} file: {}", MODULE_EXTENSION, so_path.display());
        }

        if self.create_tar {
            // Create .tar.gz archive
            let temp_dir = PathBuf::from(format!(".tmp/{}", metadata.name));
            info!("Creating temporary directory: {}", temp_dir.display());
            fs::create_dir_all(&temp_dir)?;

            info!(
                "Copying .{} file from {} to {}",
                MODULE_EXTENSION,
                so_path.display(),
                temp_dir.display()
            );
            fs::copy(
                &so_path,
                temp_dir.join(format!("module.{}", MODULE_EXTENSION)),
            )?;

            info!("Copying metadata file to temp folder");
            fs::copy(
                &metadata_path,
                temp_dir.join(metadata_path.file_name().unwrap()),
            )?;

            let archive_name = format!("{}-{}.tar.gz", metadata.name, metadata.version);

            info!("Creating archive: {}", archive_name);
            let status = Command::new("tar")
                .args(["-czf", &archive_name, "-C"])
                .arg(temp_dir.to_str().unwrap())
                .arg(".")
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
        } else {
            // Save directly to package_target/module-name
            let package_dir = PathBuf::from(&self.package_target).join(&metadata.name);

            info!("Creating package directory: {}", package_dir.display());
            fs::create_dir_all(&package_dir)?;

            info!(
                "Copying .{} file from {} to {}",
                MODULE_EXTENSION,
                so_path.display(),
                package_dir.display()
            );
            fs::copy(
                &so_path,
                package_dir.join(format!("module.{}", MODULE_EXTENSION)),
            )?;

            info!("Copying metadata file to package folder");
            fs::copy(
                &metadata_path,
                package_dir.join(metadata_path.file_name().unwrap()),
            )?;

            let result = format!("{}/{}", self.package_target, metadata.name);
            info!("Success! Package created in: {} 🎉", result);

            Ok(result)
        }
    }
}
