#!/usr/bin/env bash

set -e

BIN_PATH="/usr/local/bin/phlow"

echo "🔍 Detecting platform..."

if [ -f "$BIN_PATH" ]; then
  echo "🗑️ Removing $BIN_PATH (requires sudo)..."
  sudo rm "$BIN_PATH"
  echo "✅ Phlow has been uninstalled."
else
  echo "⚠️ Phlow binary not found at $BIN_PATH."
fi
