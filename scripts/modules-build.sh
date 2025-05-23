#!/bin/bash

# Define os diretórios
RELEASE_DIR="target/release"
DEST_DIR="./phlow_packages"
MODULES_DIR="./modules"

# Cria o diretório de destino se não existir
mkdir -p "$DEST_DIR"

# Habilita nullglob para evitar erro se não houver arquivos .dylib
# shopt -s nullglob
so_files=("$RELEASE_DIR"/lib*.dylib)

if [ ${#so_files[@]} -eq 0 ]; then
    echo "Nenhum arquivo .dylib encontrado em $RELEASE_DIR"
    exit 1
fi

# Processa cada .dylib
for file in "${so_files[@]}"; do
    filename=$(basename "$file")
    modulename=${filename#lib}              # Remove o prefixo 'lib'
    modulename_no_ext="${modulename%.dylib}"   # Remove a extensão .dylib

    module_dest_dir="$DEST_DIR/$modulename_no_ext"
    mkdir -p "$module_dest_dir"

    # Copia e renomeia a .dylib como module.dylib
    cp "$file" "$module_dest_dir/module.dylib"
    echo "Copiado: $file -> $module_dest_dir/module.dylib"

    # Copia o phlow.yaml correspondente
    props_file="$MODULES_DIR/$modulename_no_ext/phlow.yaml"
    if [ -f "$props_file" ]; then
        cp "$props_file" "$module_dest_dir/phlow.yaml"
        echo "Copiado: $props_file -> $module_dest_dir/phlow.yaml"
    else
        echo "Aviso: $props_file não encontrado"
    fi
done
