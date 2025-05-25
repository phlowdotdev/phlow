#!/bin/bash
set -e

# Detect operating system
OS_SUFFIX=""
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS_SUFFIX="-darwin"
    echo "🍎 Detected macOS platform"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS_SUFFIX="-linux_gnu"
    echo "🐧 Detected Linux GNU platform"
else
    echo "⚠️ Unknown platform: $OSTYPE"
    OS_SUFFIX="-unknown"
fi

# verifica de pasta packages não existe, se não existir cria
if [ ! -d "./packages" ]; then
    echo "📦 Criando pasta ./packages"
    mkdir -p ./packages
fi

# apaga todo conteudo da pasta packages
echo "📦 Limpando pasta ./packages"
rm -rf ./packages/*

for dir in ./modules/*/; do
    if [ -d "$dir" ]; then
        echo "📦 Empacotando $dir"
        cargo run --release -p phlow-runtime -- --package "$dir"
        
        # rename tar.gz files to include OS suffix
        for tarfile in *.tar.gz; do
            # Get the filename without extension
            filename="${tarfile%.tar.gz}"
            # Rename with OS suffix
            echo "📦 Renaming $tarfile to ${filename}${OS_SUFFIX}.tar.gz"
            mv "$tarfile" "${filename}${OS_SUFFIX}.tar.gz"
        done
        
        # move para pasta packages
        mv *.tar.gz ./packages/
    fi
done
