#!/bin/bash
set -e

# Compila o projeto em modo release
cargo build --release --locked

# Define os diretórios
RELEASE_DIR="target/release"
DEST_DIR="phlow_packages"
MODULES_DIR="modules"
PACKAGE_DIR="phlow_packages"

# Cria os diretórios de destino se não existirem
mkdir -p "$DEST_DIR"
mkdir -p "$PACKAGE_DIR"

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
    modulename=${filename#lib}
    modulename_no_ext="${modulename%.so}"

    module_dest_dir="$DEST_DIR/$modulename_no_ext"
    mkdir -p "$module_dest_dir"

    # Copia e renomeia a .so como module.so
    cp "$file" "$module_dest_dir/module.so"
    echo "Copy: $file -> $module_dest_dir/module.so"

    version=""
    found_metadata=false

    # Procura phlow.{yaml,yml,json} no diretório correspondente em modules/
    for ext in yaml yml json; do
        props_file="$MODULES_DIR/$modulename_no_ext/phlow.$ext"
        if [ -f "$props_file" ]; then
            cp "$props_file" "$module_dest_dir/phlow.$ext"
            echo "Copy: $props_file -> $module_dest_dir/phlow.$ext"
            found_metadata=true

            # Extrai a versão do arquivo
            if [[ "$ext" == "json" ]]; then
                version=$(jq -r '.version // empty' "$props_file")
            else
                version=$(grep '^version:' "$props_file" | sed 's/version:[[:space:]]*//')
            fi
            break
        fi
    done

    if [ "$found_metadata" = false ]; then
        echo "Aviso: Nenhum arquivo phlow.{yaml,yml,json} encontrado para $modulename_no_ext"
        continue
    fi

    if [ -z "$version" ]; then
        echo "Erro: Não foi possível extrair a versão de $modulename_no_ext"
        exit 1
    fi

    # Compacta o diretório do módulo como .tar.gz
    package_name="${modulename_no_ext}-${version}.tar.gz"
    tar -czf "$PACKAGE_DIR/$package_name" -C "$DEST_DIR" "$modulename_no_ext"
    echo "Package: $PACKAGE_DIR/$package_name criado com sucesso"
done
