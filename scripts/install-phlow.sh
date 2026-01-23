#!/usr/bin/env bash

set -e

REPO="phlowdotdev/phlow"
BIN_NAME="phlow"
INSPECT_BIN_NAME="phlow-tui-inspect"
INSTALL_DIR="$HOME/.phlow"
BIN_PATH="$INSTALL_DIR/phlow"
INSPECT_BIN_PATH="$INSTALL_DIR/phlow-tui-inspect"
ADD_PATH_CMD='export PATH="$HOME/.phlow:$PATH"'

echo "🔍 Detecting platform..."

OS=$(uname -s)
ARCH=$(uname -m)

# Identificando o asset correto
if [[ "$OS" == "Darwin" ]]; then
    ASSET_NAME="${BIN_NAME}-macos"
    INSPECT_ASSET_NAME="${INSPECT_BIN_NAME}-macos"
elif [[ "$OS" == "Linux" ]]; then
    case "$ARCH" in
        x86_64)
            ASSET_NAME="${BIN_NAME}-amd64"
            INSPECT_ASSET_NAME="${INSPECT_BIN_NAME}-amd64"
            ;;
        aarch64 | arm64)
            ASSET_NAME="${BIN_NAME}-arm64"
            INSPECT_ASSET_NAME="${INSPECT_BIN_NAME}-arm64"
            ;;
        *)
            echo "❌ Unsupported Linux architecture: $ARCH"
            exit 1
            ;;
    esac
else
    echo "❌ Unsupported OS: $OS"
    exit 1
fi

echo "📦 Platform detected: OS=$OS ARCH=$ARCH"
echo "⬇️ Fetching latest version from GitHub..."

LATEST=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep tag_name | cut -d '"' -f 4)
echo "📦 Latest version is $LATEST"

URL="https://github.com/$REPO/releases/download/$LATEST/$ASSET_NAME"
INSPECT_URL="https://github.com/$REPO/releases/download/$LATEST/$INSPECT_ASSET_NAME"

echo "🚚 Downloading $BIN_NAME from $URL..."
mkdir -p "$INSTALL_DIR"
curl -L "$URL" -o "$BIN_PATH"

echo "🚚 Downloading $INSPECT_BIN_NAME from $INSPECT_URL..."
curl -L "$INSPECT_URL" -o "$INSPECT_BIN_PATH"

echo "⚙️ Making binary executable..."
chmod +x "$BIN_PATH"
chmod +x "$INSPECT_BIN_PATH"

echo "🔧 Updating shell configuration files..."
for shell_rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    if [ -f "$shell_rc" ]; then
        if ! grep -Fxq "$ADD_PATH_CMD" "$shell_rc"; then
            echo "$ADD_PATH_CMD" >> "$shell_rc"
            echo "✅ Added ~/.phlow to PATH in $(basename "$shell_rc")"
        else
            echo "ℹ️ ~/.phlow already in PATH in $(basename "$shell_rc")"
        fi
    fi
done

echo "🎉 Installation complete!"
echo "ℹ️ Open a new terminal, or run: source ~/.bashrc (or ~/.zshrc) to update your session."
