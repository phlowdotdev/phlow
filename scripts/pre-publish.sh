#!/bin/bash

# Caminho base dos módulos
BASE_DIR="./modules"

# Varre todos os diretórios dentro de modules
find "$BASE_DIR" -type f -name "Cargo.toml" | while read -r cargo_toml_path; do
  # Obtém o diretório do módulo
  module_dir=$(dirname "$cargo_toml_path")

  echo "Publicando módulo: $module_dir"

  # Executa o comando cargo run com o path encontrado
  cargo run -p phlow-runtime --release -- --publish "$module_dir"
done
