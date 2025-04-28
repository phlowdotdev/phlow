#!/usr/bin/env bash

set -e

REPO="phlowdotdev/phlow"
BIN_NAME="phlow-x86_64-unknown-linux-gnu"
INSTALL_PATH="/usr/local/bin/phlow"

echo "🔍 Detecting platform..."
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    *) echo "❌ Unsupported architecture: $ARCH" && exit 1 ;;
esac

echo "⬇️ Fetching latest version from GitHub..."
LATEST=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep tag_name | cut -d '"' -f 4)
echo "📦 Latest version is $LATEST"

URL="https://github.com/$REPO/releases/download/$LATEST/$BIN_NAME"

echo "🚚 Downloading $BIN_NAME from $URL..."
curl -L "$URL" -o "$BIN_NAME"

echo "⚙️ Making binary executable..."
chmod +x "$BIN_NAME"

echo "📁 Moving to $INSTALL_PATH (requires sudo)..."
sudo mv "$BIN_NAME" "$INSTALL_PATH"

echo "✅ $BIN_NAME installed successfully at $INSTALL_PATH"
echo "🚀 Run it with: phlow --help"
