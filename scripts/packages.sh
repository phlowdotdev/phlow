echo "📦 Clean folder ./packages"
echo "🎉 All modules packaged successfully!"
#!/usr/bin/env bash

set -euo pipefail

# Usage:
#  ./scripts/packages.sh                -> prepare environment and run packaging in parallel
#  ./scripts/packages.sh --single <dir> -> package a single module (used by the parallel launcher)

# Initialize variables with defaults to avoid unbound variable errors
: "${OS_SUFFIX:=}"
: "${TARGET:=}"
: "${MODULE_EXTENSION:=}"

# If called in single-module mode, only run packaging logic for that module
if [[ "${1:-}" == "--single" ]]; then
  MODULE_DIR="$2"
  # ensure we fail fast if MODULE_DIR is empty
  if [[ -z "$MODULE_DIR" ]]; then
    echo "Missing module dir for --single"
    exit 2
  fi
  # Validate required env vars for single mode
  if [[ -z "$TARGET" || -z "$OS_SUFFIX" || -z "$MODULE_EXTENSION" ]]; then
    echo "ERROR: --single mode requires TARGET, OS_SUFFIX, and MODULE_EXTENSION environment variables"
    exit 2
  fi
fi

# ------------------------------------------------------------
# FUNÇÃO DE EMPACOTAMENTO DE MÓDULO
# ------------------------------------------------------------
package_module() {
    MODULE_DIR="$1"

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
    echo "⚙️ Building module: $MODULE_DIR..."
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

# If called in single mode, perform just that and exit
if [[ "${1:-}" == "--single" ]]; then
  if [[ -z "${2:-}" ]]; then
    echo "Expected module directory after --single"
    exit 2
  fi
  # the variables TARGET, OS_SUFFIX and MODULE_EXTENSION must be provided by the caller
  package_module "$2"
  exit 0
fi

# ------------------------------------------------------------
# MODO PRINCIPAL: prepara ambiente e dispara empacotamento em paralelo
# ------------------------------------------------------------

cargo install cross || true

# Detect operating system or target
# Define OS_SUFFIX, TARGET e MODULE_EXTENSION dinamicamente
if [[ -z "${OS_SUFFIX:-}" ]] || [[ -z "${TARGET:-}" ]] || [[ -z "${MODULE_EXTENSION:-}" ]]; then
  if [[ "$OSTYPE" == "darwin"* ]]; then
    OS_SUFFIX="${OS_SUFFIX:--darwin}"
    TARGET="${TARGET:-x86_64-apple-darwin}"
    if [[ "$(uname -m)" == "arm64" ]]; then
        OS_SUFFIX="-darwin-aarch64"
        TARGET="aarch64-apple-darwin"
    elif [[ "$(uname -m)" == "x86_64" ]]; then
        OS_SUFFIX="-darwin-x86_64"
        TARGET="x86_64-apple-darwin"
    fi
    MODULE_EXTENSION="dylib"
    echo "🍎 Detected macOS platform"
  elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    ARCH=$(uname -m)
    if [[ "$ARCH" == "x86_64" ]]; then
      OS_SUFFIX="${OS_SUFFIX:--linux-amd64}"
      TARGET="${TARGET:-x86_64-unknown-linux-gnu}"
      MODULE_EXTENSION="so"
      echo "🐧 Detected Linux amd64 platform"
    elif [[ "$ARCH" == "aarch64" ]]; then
      OS_SUFFIX="${OS_SUFFIX:--linux-aarch64}"
      TARGET="${TARGET:-aarch64-unknown-linux-gnu}"
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

# Default de paralelismo (padrão: número de CPUs, mas limite razoável para runners)
PARALLEL=${PARALLEL:-$(nproc 2>/dev/null || echo 2)}
if [[ "$PARALLEL" -gt 8 ]]; then
  PARALLEL=8
fi

echo "⚡ Running packaging in parallel (jobs=$PARALLEL)"

# Collect module directories
MODULES=()
for dir in ./modules/*/; do
    if [ -d "$dir" ]; then
        MODULES+=("$dir")
    fi
done

if [[ ${#MODULES[@]} -eq 0 ]]; then
  echo "No modules found in ./modules"
  exit 0
fi

# Export env needed by single-mode invocations
export OS_SUFFIX
export TARGET
export MODULE_EXTENSION

# Use xargs to run multiple instances of this script in parallel, each handling a single module
printf '%s\n' "${MODULES[@]}" | xargs -n1 -P "$PARALLEL" -I{} bash -c './scripts/packages.sh --single "{}"'

echo "🎉 All modules packaged successfully!"
