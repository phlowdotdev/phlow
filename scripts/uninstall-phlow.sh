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

# Remove PATH entry
REMOVE_PATH_CMD='export PATH="$HOME/.phlow:$PATH"'

remove_from_shell_config() {
    local file="$1"
    if [ -f "$file" ]; then
        if grep -Fxq "$REMOVE_PATH_CMD" "$file"; then
            echo "✏️ Removing PATH modification from $file..."
            sed -i.bak "\|$REMOVE_PATH_CMD|d" "$file"
            echo "✅ Updated $file (backup created as $file.bak)"
        else
            echo "ℹ️ No PATH modification found in $file."
        fi
    fi
}

remove_from_shell_config "$HOME/.zshrc"
remove_from_shell_config "$HOME/.bashrc"

echo "🎉 Uninstallation complete! Open a new terminal or run 'source ~/.zshrc' or 'source ~/.bashrc' to update your session."
