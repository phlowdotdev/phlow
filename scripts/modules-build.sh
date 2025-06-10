#!/bin/bash

# Define os diretórios
RELEASE_DIR="target/release"
DEST_DIR="./phlow_packages"
MODULES_DIR="./modules"


cargo build --release

# Cria o diretório de destino se não existir
mkdir -p "$DEST_DIR"

# Habilita nullglob para evitar erro se não houver arquivos de biblioteca
if [[ "$(uname)" == "Linux" ]]; then
    LIB_EXT="so"
else
    LIB_EXT="dylib"
fi

lib_files=("$RELEASE_DIR"/lib*."$LIB_EXT")

if [ ${#lib_files[@]} -eq 0 ]; then
    echo "Nenhum arquivo .$LIB_EXT encontrado em $RELEASE_DIR"
    exit 1
fi

# Processa cada biblioteca
for file in "${lib_files[@]}"; do
    filename=$(basename "$file")
    modulename=${filename#lib}                      # Remove o prefixo 'lib'
    modulename_no_ext="${modulename%.$LIB_EXT}"     # Remove a extensão

    module_dest_dir="$DEST_DIR/$modulename_no_ext"
    mkdir -p "$module_dest_dir"

    # Copia e renomeia a biblioteca como module.$LIB_EXT
    cp "$file" "$module_dest_dir/module.$LIB_EXT"
    echo "Copiado: $file -> $module_dest_dir/module.$LIB_EXT"

    # Copia o phlow.yaml correspondente
    props_file="$MODULES_DIR/$modulename_no_ext/phlow.yaml"
    if [ -f "$props_file" ]; then
        cp "$props_file" "$module_dest_dir/phlow.yaml"
        echo "Copiado: $props_file -> $module_dest_dir/phlow.yaml"
    else
        echo "Aviso: $props_file não encontrado"
    fi
done
