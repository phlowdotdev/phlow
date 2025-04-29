#!/usr/bin/env bash

set -e

REPO="phlowdotdev/phlow"
BIN_NAME="phlow"
INSTALL_DIR="$HOME/.phlow"
BIN_PATH="$INSTALL_DIR/phlow"
ADD_PATH_CMD='export PATH="$HOME/.phlow:$PATH"'

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
