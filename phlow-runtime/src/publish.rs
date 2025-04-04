use std::{fs, path::PathBuf, process::Command};

use anyhow::{bail, Context, Result};

#[derive(Debug)]
pub struct Publish {
    pub modules_dir: PathBuf,
}

impl TryFrom<String> for Publish {
    type Error = anyhow::Error;

    fn try_from(modules_dir: String) -> Result<Self> {
        let modules_dir = PathBuf::from(modules_dir);
        if !modules_dir.exists() {
            bail!("Error: Directory {} does not exist", modules_dir.display());
        }
        Ok(Publish { modules_dir })
    }
}

impl Publish {
    pub fn run(&self) -> Result<()> {
        let release_dir = PathBuf::from("target/release");
        let dest_dir = PathBuf::from("phlow_modules");
        let package_dir = PathBuf::from("packages");
        let modules_dir = self.modules_dir.clone();

        // Compila o projeto
        Command::new("cargo")
            .args(["build", "--release", "--locked"])
            .status()
            .context("Compile error")?
            .success()
            .then_some(())
            .context("Compile Fail")?;

        // Cria diretórios
        fs::create_dir_all(&dest_dir)?;
        fs::create_dir_all(&package_dir)?;

        // Lê arquivos .so
        let so_files: Vec<_> = fs::read_dir(&release_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "so" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        if so_files.is_empty() {
            bail!("Nenhum arquivo .so encontrado em {}", release_dir.display());
        }

        for file in so_files {
            let filename = file.file_name().unwrap().to_string_lossy();
            let modulename = filename.trim_start_matches("lib").trim_end_matches(".so");
            let module_dest_dir = dest_dir.join(modulename);
            fs::create_dir_all(&module_dest_dir)?;

            // Copia .so
            let dest_so = module_dest_dir.join("module.so");
            fs::copy(&file, &dest_so)?;
            println!("Copy: {} -> {}", file.display(), dest_so.display());

            let mut version = None;
            let mut found_metadata = false;

            for ext in ["yaml", "yml", "json"] {
                let props_file = modules_dir.join(modulename).join(format!("phlow.{ext}"));
                if !props_file.exists() {
                    continue;
                }

                let dest_props = module_dest_dir.join(format!("phlow.{ext}"));
                fs::copy(&props_file, &dest_props)?;
                println!("Copy: {} -> {}", props_file.display(), dest_props.display());

                found_metadata = true;

                if ext == "json" {
                    let data: serde_json::Value =
                        serde_json::from_reader(fs::File::open(&props_file)?)?;
                    version = data
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                } else {
                    let content = fs::read_to_string(&props_file)?;
                    for line in content.lines() {
                        if let Some(v) = line.strip_prefix("version:") {
                            version = Some(v.trim().to_string());
                            break;
                        }
                    }
                }

                if version.is_some() {
                    break;
                }
            }

            if !found_metadata {
                eprintln!(
                    "Aviso: Nenhum arquivo phlow.yaml/yml/json encontrado para {}",
                    modulename
                );
                continue;
            }

            let version = version.ok_or_else(|| {
                anyhow::anyhow!("Erro: Não foi possível extrair a versão de {}", modulename)
            })?;

            // Compacta
            let package_name = format!("{modulename}-{version}.tar.gz");
            let output_path = package_dir.join(package_name);

            let status = Command::new("tar")
                .args([
                    "-czf",
                    output_path.to_str().unwrap(),
                    "-C",
                    dest_dir.to_str().unwrap(),
                    modulename,
                ])
                .status()?;

            if status.success() {
                println!("Package: {} criado com sucesso", output_path.display());
            } else {
                bail!("Erro ao criar pacote {}", output_path.display());
            }
        }

        Ok(())
    }
}
