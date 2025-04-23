#!/bin/bash
set -e

# verifica de pasta raw exste, se existe remove
if [ -d "./raw" ]; then
    echo "🗑️  Removendo pasta ./raw"
    rm -rf ./raw
fi
# cria pasta raw
mkdir -p ./raw

for dir in ./modules/*/; do
    if [ -d "$dir" ]; then
    echo "📦 Empacotando $dir"
    cargo run --release -p phlow-runtime -- --package "$dir"
    # move para pasta raw
    mv *.tar.gz ./raw/
    fi
done