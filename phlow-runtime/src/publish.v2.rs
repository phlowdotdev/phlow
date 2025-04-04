use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub struct Publish {
    pub modules_dir: PathBuf,
}

impl TryFrom<String> for Publish {
    type Error = anyhow::Error;

    fn try_from(modules_dir: String) -> Result<Self> {
        let modules_dir = PathBuf::from(&modules_dir);
        if !modules_dir.exists() {
            bail!("Error: Directory {} does not exist", modules_dir.display());
        }
        Ok(Publish { modules_dir })
    }
}

#[derive(Serialize, Deserialize)]
struct IndexEntry {
    version: String,
    repository: String,
    archive: String,
}

impl Publish {
    pub fn run(&self) -> Result<()> {
        let release_dir = PathBuf::from("target/release");
        let dest_dir = PathBuf::from(".tmp/modules");
        let package_dir = PathBuf::from(".tmp/packages");
        let final_dir = PathBuf::from("packages");
        let indexs_dir = PathBuf::from("indexs");

        Command::new("cargo")
            .args(["build", "--release", "--locked"])
            .status()
            .context("Erro ao compilar")?
            .success()
            .then_some(())
            .context("Compilação falhou")?;

        fs::create_dir_all(&dest_dir)?;
        fs::create_dir_all(&package_dir)?;
        fs::create_dir_all(&final_dir)?;

        let module_dirs = self.discover_modules()?;

        for modulename in module_dirs {
            let so_path = release_dir.join(format!("lib{modulename}.so"));
            if !so_path.exists() {
                eprintln!("Aviso: Arquivo {} não encontrado", so_path.display());
                continue;
            }

            let module_dest_dir = dest_dir.join(&modulename);
            fs::create_dir_all(&module_dest_dir)?;

            fs::copy(&so_path, module_dest_dir.join("module.so"))?;

            let (props_path, version, repository) =
                self.extract_metadata(&modulename, &module_dest_dir)?;

            let package_name = format!("{modulename}-{version}.tar.gz");
            let package_path = package_dir.join(&package_name);

            let status = Command::new("tar")
                .args([
                    "-czf",
                    package_path.to_str().unwrap(),
                    "-C",
                    dest_dir.to_str().unwrap(),
                    &modulename,
                ])
                .status()?;

            if !status.success() {
                bail!("Erro ao criar pacote {}", package_path.display());
            }

            self.update_index(
                &modulename,
                &version,
                &repository,
                &package_name,
                &indexs_dir,
            )?;
        }

        self.distribute_packages(&package_dir, &final_dir)?;

        fs::remove_dir_all(&package_dir).ok();
        fs::remove_dir_all(&dest_dir).ok();

        println!("\nProcesso concluído com sucesso! 🎉");
        Ok(())
    }

    fn discover_modules(&self) -> Result<Vec<String>> {
        let mut result = vec![];

        if self.modules_dir.join("phlow.yaml").exists()
            || self.modules_dir.join("phlow.yml").exists()
            || self.modules_dir.join("phlow.json").exists()
        {
            let name = self
                .modules_dir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let temp_dir = PathBuf::from(".tmp/single");
            let target = temp_dir.join(&name);
            fs::create_dir_all(&target)?;
            for entry in fs::read_dir(&self.modules_dir)? {
                let entry = entry?;
                fs::copy(entry.path(), target.join(entry.file_name()))?;
            }
            result.push(name);
        } else {
            for entry in fs::read_dir(&self.modules_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    result.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        Ok(result)
    }

    fn extract_metadata(
        &self,
        modulename: &str,
        dest_dir: &Path,
    ) -> Result<(PathBuf, String, String)> {
        for ext in ["yaml", "yml", "json"] {
            let props_path = self
                .modules_dir
                .join(modulename)
                .join(format!("phlow.{ext}"));
            if !props_path.exists() {
                continue;
            }

            fs::copy(&props_path, dest_dir.join(props_path.file_name().unwrap()))?;

            if ext == "json" {
                let data: Value = serde_json::from_reader(File::open(&props_path)?)?;
                let version = data["version"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Versão ausente"))?
                    .to_string();
                let repo = data["repository"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Repositório ausente"))?
                    .to_string();
                return Ok((props_path, version, repo));
            } else {
                let content = fs::read_to_string(&props_path)?;
                let mut version = None;
                let mut repo = None;
                for line in content.lines() {
                    if let Some(v) = line.strip_prefix("version:") {
                        version = Some(v.trim().to_string());
                    }
                    if let Some(r) = line.strip_prefix("repository:") {
                        repo = Some(r.trim().to_string());
                    }
                }
                if let (Some(v), Some(r)) = (version, repo) {
                    return Ok((props_path, v, r));
                }
            }
        }
        bail!("Arquivo de metadados não encontrado para {modulename}");
    }

    fn update_index(
        &self,
        modulename: &str,
        version: &str,
        repository: &str,
        archive: &str,
        indexs_dir: &Path,
    ) -> Result<()> {
        let (p1, p2) = Self::build_path_from_name(modulename);
        let index_path = indexs_dir.join(p1).join(p2);
        fs::create_dir_all(&index_path)?;
        let index_file = index_path.join(format!("{modulename}.json"));

        let mut entries: Vec<IndexEntry> = if index_file.exists() {
            serde_json::from_reader(File::open(&index_file)?)?
        } else {
            vec![]
        };

        if entries.iter().any(|e| e.version == version) {
            println!("Versão {version} já existe para {modulename}");
            return Ok(());
        }

        entries.push(IndexEntry {
            version: version.to_string(),
            repository: repository.to_string(),
            archive: archive.to_string(),
        });

        let mut file = File::create(&index_file)?;
        file.write_all(serde_json::to_string_pretty(&entries)?.as_bytes())?;
        println!("Indice atualizado: {}", index_file.display());

        Ok(())
    }

    fn distribute_packages(&self, package_dir: &Path, final_dir: &Path) -> Result<()> {
        for entry in fs::read_dir(package_dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = entry.file_name().into_string().unwrap();

            if !filename.ends_with(".tar.gz") {
                continue;
            }

            let base = filename.trim_end_matches(".tar.gz");
            let module = base.rsplit_once('-').map(|(m, _)| m).unwrap_or("");
            if module.len() < 2 {
                eprintln!("Nome muito curto: {module}");
                continue;
            }

            let (p1, p2) = Self::build_path_from_name(module);
            let target = final_dir.join(p1).join(p2).join(module);
            fs::create_dir_all(&target)?;

            fs::rename(path, target.join(&filename))?;
            println!("Movido: {filename} -> {}", target.display());
        }
        Ok(())
    }

    fn build_path_from_name(name: &str) -> (String, String) {
        let padded = format!("{name}____");
        let p1 = &padded[0..2];
        let p2 = &padded[2..4];
        (p1.to_string(), p2.to_string())
    }
}
