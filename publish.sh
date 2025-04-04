#!/bin/bash
set -euo pipefail

# Diretórios
RELEASE_DIR="target/release"
DEST_DIR="phlow_modules"
MODULES_DIR="modules"
PACKAGE_DIR="phlow_packages"
FINAL_DIR="packages"
RAW_DIR="./$PACKAGE_DIR"

# Garante que os diretórios existam
mkdir -p "$DEST_DIR" "$PACKAGE_DIR" "$FINAL_DIR"

# Ativa nullglob para evitar erro se não houver arquivos .so
shopt -s nullglob
so_files=("$RELEASE_DIR"/lib*.so)

if [ ${#so_files[@]} -eq 0 ]; then
    echo "Nenhum arquivo .so encontrado em $RELEASE_DIR"
    exit 1
fi

# Verifica dependência
command -v jq >/dev/null 2>&1 || {
    echo >&2 "Erro: 'jq' não está instalado."
    exit 1
}

get_version() {
    local file=$1
    local ext=$2
    if [[ "$ext" == "json" ]]; then
        jq -r '.version // empty' "$file"
    else
        grep '^version:' "$file" | sed 's/version:[[:space:]]*//'
    fi
}

# Build
echo "Compilando projeto em modo release..."
cargo build --release --locked

# Processa os arquivos .so
for file in "${so_files[@]}"; do
    filename=$(basename "$file")
    modulename=${filename#lib}
    modulename_no_ext="${modulename%.so}"

    module_dest_dir="$DEST_DIR/$modulename_no_ext"
    mkdir -p "$module_dest_dir"

    cp "$file" "$module_dest_dir/module.so"
    echo "Copy: $file -> $module_dest_dir/module.so"

    version=""
    found_metadata=false

    for ext in yaml yml json; do
        props_file="$MODULES_DIR/$modulename_no_ext/phlow.$ext"
        if [ -f "$props_file" ]; then
            cp "$props_file" "$module_dest_dir/phlow.$ext"
            echo "Copy: $props_file -> $module_dest_dir/phlow.$ext"
            version=$(get_version "$props_file" "$ext")
            found_metadata=true
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

    package_name="${modulename_no_ext}-${version}.tar.gz"
    tar -czf "$PACKAGE_DIR/$package_name" -C "$DEST_DIR" "$modulename_no_ext"
    echo "Pacote criado: $PACKAGE_DIR/$package_name"
done

# Distribui os pacotes em estrutura m/e/u/d/i/r
echo ""
echo "Distribuindo pacotes em estrutura de diretórios..."
for filepath in "$RAW_DIR"/*.tar.gz; do
    [ -e "$filepath" ] || continue

    filename=$(basename "$filepath")
    base_name="${filename%.tar.gz}"

    if [ ${#base_name} -lt 6 ]; then
        echo "Aviso: Nome $base_name muito curto para distribuição. Pulando."
        continue
    fi

    current_path="$FINAL_DIR"
    for (( i=0; i<${#base_name}; i++ )); do
        letter="${base_name:$i:1}"
        current_path="$current_path/$letter"
        mkdir -p "$current_path"
    done

    mv -n "$filepath" "$current_path/$filename"
    echo "Movido: $filepath -> $current_path/$filename"
done

echo ""
echo "Processo finalizado com sucesso!"
