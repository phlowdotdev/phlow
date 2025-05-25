#!/bin/bash

RAW_DIR="./raw"
DEST_DIR="./packages"

mkdir -p "$DEST_DIR"

echo "Resolve packages directory"
echo "  - Source: $RAW_DIR"
echo "  - Destination: $DEST_DIR"

ls "$RAW_DIR"


for filepath in "$RAW_DIR"/*.tar.gz; do
  echo "Processing: $filepath"
  [ -e "$filepath" ] || continue

  filename=$(basename "$filepath")
  base_name="${filename%.tar.gz}"

  echo "  - Base name: $base_name"
  echo "  - File path: $filepath"
  echo "  - Destination: $DEST_DIR"
  echo "  - File name: $filename"

  # Extrai nome e versão do padrão: nome-versão-architecture_123.tar.gz
  package_name="${base_name%%-*}"
  version="${base_name#*-}"
  version="${version%%-*}"

  # Padroniza para no mínimo 4 caracteres
  padded=$(printf "%-4s" "$package_name" | tr ' ' '_')

  prefix="${padded:0:2}"
  middle="${padded:2:2}"

  final_path="$DEST_DIR/$prefix/$middle/$package_name"
  metadata_file="$final_path/metadata.json"

  mkdir -p "$final_path"

  # Atualiza metadata.json com a versão mais recente
  jq -n \
    --arg name "$package_name" \
    --arg latest "$version" \
    '{name: $name, latest: $latest}' \
    > "$metadata_file"

  # Move o tar.gz
  cp "$filepath" "$final_path/$filename"

  echo "✅ Created package directory: $final_path"
done