#!/usr/bin/env bash

set -e

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
        echo "🍎 Detected macOS platform"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        ARCH=$(uname -m)
        if [[ "$ARCH" == "x86_64" ]]; then
            if [[ -z "$OS_SUFFIX" ]]; then OS_SUFFIX="-linux-amd64"; fi
            if [[ -z "$TARGET" ]]; then TARGET="x86_64-unknown-linux-gnu"; fi
            echo "🐧 Detected Linux amd64 platform"
        elif [[ "$ARCH" == "aarch64" ]]; then
            if [[ -z "$OS_SUFFIX" ]]; then OS_SUFFIX="-linux-aarch64"; fi
            if [[ -z "$TARGET" ]]; then TARGET="aarch64-unknown-linux-gnu"; fi
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

# Cria a pasta packages
if [ ! -d "./packages" ]; then
    echo "📦 Create folder ./packages"
    mkdir -p ./packages
fi

echo "📦 Clean folder ./packages"
rm -rf ./packages/*

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

    # Build do projeto
    echo "⚙️ Building module..."
    cross build --target "$TARGET" --release --locked

    TMP_DIR=".tmp/${NAME}"
    mkdir -p "$TMP_DIR"

    SO_NAME="lib${NAME}.${MODULE_EXTENSION}"
    RELEASE_PATH="../../target/$TARGET/release/$SO_NAME"

    if [ ! -f "$RELEASE_PATH" ]; then
      echo "❌ Missing built file: $RELEASE_PATH"
      exit 1
    fi

    cp "$RELEASE_PATH" "$TMP_DIR/module.${MODULE_EXTENSION}"
    cp "$METADATA_FILE" "$TMP_DIR/"

    ARCHIVE_NAME="${NAME}-${VERSION}.tar.gz"

    echo "📦 Creating archive: $ARCHIVE_NAME"
    tar -czf "$ARCHIVE_NAME" -C "$TMP_DIR" .

    rm -rf "$TMP_DIR"

    cd - > /dev/null

    # Renomeia com OS_SUFFIX
    RENAMED_ARCHIVE="${NAME}-${VERSION}${OS_SUFFIX}.tar.gz"
    mv "$MODULE_DIR/$ARCHIVE_NAME" "./packages/$RENAMED_ARCHIVE"

    echo "✅ Module packaged: $RENAMED_ARCHIVE"
}

# ------------------------------------------------------------
# LOOP EM CADA MÓDULO
# ------------------------------------------------------------

for dir in ./modules/*/; do
    if [ -d "$dir" ]; then
        echo "🚀 Processing module: $dir"
        package_module "$dir"
    fi
done

echo "🎉 All modules packaged successfully!"
