#!/bin/bash
set -e

for dir in ./modules/*/; do
    if [ -d "$dir" ]; then
    echo "📦 Empacotando $dir"
    cargo run --release -p phlow-runtime -- --package "$dir"
    fi
done