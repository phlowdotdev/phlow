#!/usr/bin/env bash

set -e

BIN_NAME="phlow"
INSTALL_DIR="$HOME/.phlow"
BIN_PATH="$INSTALL_DIR/$BIN_NAME"

echo "🧹 Starting uninstallation..."

# Remove binary and directory
if [ -f "$BIN_PATH" ]; then
    echo "🗑️ Removing binary at $BIN_PATH..."
    rm -f "$BIN_PATH"
fi

if [ -d "$INSTALL_DIR" ]; then
    echo "🗑️ Removing directory $INSTALL_DIR..."
    rmdir "$INSTALL_DIR" 2>/dev/null || echo "ℹ️ Directory not empty or already removed."
fi

echo "🎉 Uninstallation complete! Please manually update your PATH if necessary."
echo "ℹ️ Don't forget to remove any references to '$INSTALL_DIR' or '$BIN_NAME' from your ~/.bashrc and ~/.zshrc files."
