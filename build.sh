#!/bin/bash

# Compila o projeto em modo release
cargo build --release

# Define os diretórios
RELEASE_DIR="target/release"
DEST_DIR="anyflow_modules"

# Cria o diretório de destino se não existir
if [ ! -d "$DEST_DIR" ]; then
    mkdir -p "$DEST_DIR"
fi

# Localiza e copia os arquivos removendo o prefixo 'lib'
for file in "$RELEASE_DIR"/lib*.so; do
    # Extrai o nome do arquivo sem o caminho
    filename=$(basename "$file")
    
    # Remove o prefixo 'lib'
    new_filename=${filename#lib}
    
    # Copia para o diretório de destino
    cp "$file" "$DEST_DIR/$new_filename"
    echo "Copiado: $file -> $DEST_DIR/$new_filename"
done