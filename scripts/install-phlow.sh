#!/usr/bin/env bash

set -e

REPO="phlowdotdev/phlow"
BIN_NAME="phlow-x86_64-unknown-linux-gnu"
INSTALL_DIR="$HOME/.phlow"
BIN_PATH="$INSTALL_DIR/phlow"

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
mkdir -p "$INSTALL_DIR"
curl -L "$URL" -o "$BIN_PATH"

echo "⚙️ Making binary executable..."
chmod +x "$BIN_PATH"

echo "🔧 Updating shell configuration files..."
ADD_PATH_CMD='export PATH="$HOME/.phlow:$PATH"'

if [ -f "$HOME/.zshrc" ]; then
    if ! grep -Fxq "$ADD_PATH_CMD" "$HOME/.zshrc"; then
        echo "$ADD_PATH_CMD" >> "$HOME/.zshrc"
        echo "✅ Added ~/.phlow to PATH in .zshrc"
    else
        echo "ℹ️ ~/.phlow already in PATH in .zshrc"
    fi
fi

if [ -f "$HOME/.bashrc" ]; then
    if ! grep -Fxq "$ADD_PATH_CMD" "$HOME/.bashrc"; then
        echo "$ADD_PATH_CMD" >> "$HOME/.bashrc"
        echo "✅ Added ~/.phlow to PATH in .bashrc"
    else
        echo "ℹ️ ~/.phlow already in PATH in .bashrc"
    fi
fi

echo "🎉 Installation complete! Open a new terminal or run 'source ~/.zshrc' or 'source ~/.bashrc' to update your session."
