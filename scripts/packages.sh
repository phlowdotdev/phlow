#!/bin/bash
set -e

# verifica de pasta packages não existe, se não existir cria
if [ ! -d "./packages" ]; then
    echo "📦 Criando pasta ./packages"
    mkdir -p ./packages
fi

# apaga todo conteudo da pasta packages
echo "📦 Limpando pasta ./packages
rm -rf ./packages/*"

for dir in ./modules/*/; do
    if [ -d "$dir" ]; then
    echo "📦 Empacotando $dir"
    cargo run --release -p phlow-runtime -- --package "$dir"
    # move para pasta packages
    mv *.tar.gz ./packages/
    fi
done