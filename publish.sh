#!/bin/bash

# Verificação básica
if [ -z "$1" ]; then
  echo "Uso: $0 <arquivo-para-adicionar>"
  exit 1
fi

ARQUIVO=$1
REPO_ORIGINAL="https://github.com/lowcarboncode/phlow-packages.git"
TMP_DIR=$(mktemp -d)
CLONE_DIR="$TMP_DIR/phlow-packages"
BRANCH_BASE="publish"
BRANCH_NOVA="add/$(basename "$ARQUIVO" | sed 's/[^a-zA-Z0-9]/-/g')"

# Clona somente a branch 'publish'
git clone --depth 1 --branch "$BRANCH_BASE" --single-branch "$REPO_ORIGINAL" "$CLONE_DIR"

# Cria a nova branch
cd "$CLONE_DIR" || exit 1
git checkout -b "$BRANCH_NOVA"

# Copia o arquivo para o repositório clonado
cp "$OLDPWD/$ARQUIVO" .

# Adiciona, comita e faz push
git add "$(basename "$ARQUIVO")"
git commit -m "feat: adiciona $(basename "$ARQUIVO")"
git push origin "$BRANCH_NOVA"
