#!/usr/bin/env bash

set -e

# Função para exibir ajuda
show_help() {
    echo "Usage: $0 [module_name]"
    echo ""
    echo "Build and package phlow modules"
    echo ""
    echo "Arguments:"
    echo "  module_name    Optional. Name of specific module to build (e.g., echo, cli, amqp)"
    echo "                 If not provided, all modules will be built"
    echo ""
    echo "Examples:"
    echo "  $0              # Build all modules"
    echo "  $0 echo         # Build only the echo module"
    echo "  $0 cli          # Build only the cli module"
    echo ""
    echo "Available modules:"
    for dir in ./modules/*/; do
        if [ -d "$dir" ]; then
            basename "$dir"
        fi
    done
}

# Verifica se foi pedida ajuda
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    show_help
    exit 0
fi

# Captura o argumento do módulo específico (se fornecido)
SPECIFIC_MODULE="$1"

cargo install cross

# Detect operating system or target
# Use OS_SUFFIX and TARGET from environment if already set
if [[ -z "$OS_SUFFIX" || -z "$TARGET" ]]; then
  if [[ -z "$OS_SUFFIX" ]]; then OS_SUFFIX=""; fi
  if [[ -z "$TARGET" ]]; then TARGET=""; fi

  if [[ -z "$OS_SUFFIX" || -z "$TARGET" ]]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        if [[ -z "$OS_SUFFIX" ]]; then OS_SUFFIX="-darwin"; fi
        if [[ -z "$TARGET" ]]; then TARGET="x86_64-apple-darwin"; fi
        if [[ "$(uname -m)" == "arm64" ]]; then
            OS_SUFFIX="-darwin-aarch64"
            TARGET="aarch64-apple-darwin"
        fi
        if [[ "$(uname -m)" == "x86_64" ]]; then
            OS_SUFFIX="-darwin-x86_64"
            TARGET="x86_64-apple-darwin"
        fi
        MODULE_EXTENSION="dylib"
        echo "🍎 Detected macOS platform"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        ARCH=$(uname -m)
        if [[ "$ARCH" == "x86_64" ]]; then
            if [[ -z "$OS_SUFFIX" ]]; then OS_SUFFIX="-linux-amd64"; fi
            if [[ -z "$TARGET" ]]; then TARGET="x86_64-unknown-linux-gnu"; fi
            MODULE_EXTENSION="so"
            echo "🐧 Detected Linux amd64 platform"
        elif [[ "$ARCH" == "aarch64" ]]; then
            if [[ -z "$OS_SUFFIX" ]]; then OS_SUFFIX="-linux-aarch64"; fi
            if [[ -z "$TARGET" ]]; then TARGET="aarch64-unknown-linux-gnu"; fi
            MODULE_EXTENSION="so"
            echo "🐧 Detected Linux aarch64 platform"
        else
            echo "⚠️ Unknown Linux architecture: $ARCH"
            exit 1
        fi
    else
        echo "⚠️ Unknown OSTYPE: $OSTYPE"
        exit 1
    fi
  fi
fi

# Define a extensão padrão se não foi definida
if [[ -z "$MODULE_EXTENSION" ]]; then
    MODULE_EXTENSION="so"
fi

# Cria a pasta de destino
DEST_DIR="./phlow_packages"
mkdir -p "$DEST_DIR"

# Verifica dependências
if ! command -v yq &> /dev/null; then
  echo "yq not found. Please install yq (https://github.com/mikefarah/yq)"
  exit 1
fi

# ------------------------------------------------------------
# FUNÇÃO DE EMPACOTAMENTO DE MÓDULO
# ------------------------------------------------------------

package_module() {
    MODULE_DIR="$1"
    MODULE_EXTENSION="${MODULE_EXTENSION:-so}"
    
    # Salva o diretório atual (raiz do projeto)
    PROJECT_ROOT="$(pwd)"

    cd "$MODULE_DIR"

    # Busca o metadata
    if [ -f "phlow.yaml" ]; then
      METADATA_FILE="phlow.yaml"
    elif [ -f "phlow.yml" ]; then
      METADATA_FILE="phlow.yml"
    else
      echo "No phlow.yaml/yml file found in $MODULE_DIR"
      exit 1
    fi

    echo "📄 Metadata file found: $METADATA_FILE"

    NAME=$(yq -r '.name' "$METADATA_FILE")
    VERSION=$(yq -r '.version' "$METADATA_FILE")
    REPOSITORY=$(yq -r '.repository' "$METADATA_FILE")
    LICENSE=$(yq -r '.license' "$METADATA_FILE")
    AUTHOR=$(yq -r '.author' "$METADATA_FILE")

    echo "🔎 Loaded metadata:"
    echo "  name: $NAME"
    echo "  version: $VERSION"
    echo "  repository: $REPOSITORY"
    echo "  license: $LICENSE"
    echo "  author: $AUTHOR"

    # Validações
    if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-a-zA-Z0-9\.]+)?(\+[a-zA-Z0-9\.]+)?$ ]]; then
      echo "❌ Invalid version format: $VERSION"
      exit 1
    fi

    KNOWN_LICENSES=("MIT" "Apache-2.0" "GPL-3.0" "BSD-3-Clause" "MPL-2.0" "LGPL-3.0" "CDDL-1.0" "EPL-2.0" "Unlicense")
    VALID_LICENSE=false
    for lic in "${KNOWN_LICENSES[@]}"; do
      if [ "$LICENSE" == "$lic" ]; then
        VALID_LICENSE=true
        break
      fi
    done

    if ! $VALID_LICENSE; then
      if ! [[ "$LICENSE" =~ ^https?://.*$ ]]; then
        echo "❌ Invalid license: $LICENSE"
        exit 1
      fi
    fi

    # Build do projeto (apenas o pacote do módulo)
    echo "⚙️ Building module..."
    # Usa especificação com versão para evitar ambiguidades quando existir
    # um dependency externo com o mesmo nome (ex: crate `log` do crates.io).
    # Formato suportado pelo Cargo: nome@versao
    cross build -p "${NAME}@${VERSION}" --target "$TARGET" --release --locked

    SO_NAME="lib${NAME}.${MODULE_EXTENSION}"
    RELEASE_PATH="../../target/$TARGET/release/$SO_NAME"

    if [ ! -f "$RELEASE_PATH" ]; then
      echo "❌ Missing built file: $RELEASE_PATH"
      exit 1
    fi

    # Copia os arquivos para phlow_packages/NOME_DO_MODULO
    MODULE_DEST="$PROJECT_ROOT/$DEST_DIR/$NAME"
    mkdir -p "$MODULE_DEST"
    cp "$RELEASE_PATH" "$MODULE_DEST/module.${MODULE_EXTENSION}"
    cp "$METADATA_FILE" "$MODULE_DEST/"

    echo "✅ Module installed to ./phlow_packages/$NAME"

    cd - > /dev/null
}

# ------------------------------------------------------------
# LOOP EM CADA MÓDULO
# ------------------------------------------------------------

if [ -n "$SPECIFIC_MODULE" ]; then
    # Build apenas o módulo específico
    MODULE_DIR="./modules/$SPECIFIC_MODULE"
    if [ -d "$MODULE_DIR" ]; then
        echo "🚀 Processing specific module: $MODULE_DIR"
        package_module "$MODULE_DIR"
        echo "🎉 Module $SPECIFIC_MODULE installed in ./phlow_packages!"
    else
        echo "❌ Module '$SPECIFIC_MODULE' not found in ./modules/"
        echo "Available modules:"
        for dir in ./modules/*/; do
            if [ -d "$dir" ]; then
                echo "  - $(basename "$dir")"
            fi
        done
        exit 1
    fi
else
    # Build todos os módulos
    for dir in ./modules/*/; do
        if [ -d "$dir" ]; then
            echo "🚀 Processing module: $dir"
            package_module "$dir"
        fi
    done
    echo "🎉 All modules installed in ./phlow_packages!"
fi
