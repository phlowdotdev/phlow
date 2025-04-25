#!/bin/bash

# Compila o projeto em modo release


# Define os diretórios
RELEASE_DIR="target/release"
DEST_DIR="./phlow_packages"
MODULES_DIR="./modules"

# Cria o diretório de destino se não existir
mkdir -p "$DEST_DIR"

# Habilita nullglob para evitar erro se não houver arquivos .so
shopt -s nullglob
so_files=("$RELEASE_DIR"/lib*.so)

if [ ${#so_files[@]} -eq 0 ]; then
    echo "Nenhum arquivo .so encontrado em $RELEASE_DIR"
    exit 1
fi

# Processa cada .so
for file in "${so_files[@]}"; do
    filename=$(basename "$file")
    modulename=${filename#lib}              # Remove o prefixo 'lib'
    modulename_no_ext="${modulename%.so}"   # Remove a extensão .so

    module_dest_dir="$DEST_DIR/$modulename_no_ext"
    mkdir -p "$module_dest_dir"

    # Copia e renomeia a .so como module.so
    cp "$file" "$module_dest_dir/module.so"
    echo "Copiado: $file -> $module_dest_dir/module.so"

    # Copia o phlow.yaml correspondente
    props_file="$MODULES_DIR/$modulename_no_ext/phlow.yaml"
    if [ -f "$props_file" ]; then
        cp "$props_file" "$module_dest_dir/phlow.yaml"
        echo "Copiado: $props_file -> $module_dest_dir/phlow.yaml"
    else
        echo "Aviso: $props_file não encontrado"
    fi
done

cd target