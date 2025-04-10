#!/bin/bash

# Verificação básica
if [ -z "$1" ]; then
  echo "Uso: $0 <arquivo-para-adicionar>"
  exit 1
fi

ARQUIVO=$1
REPO_URL="https://github.com/lowcarboncode/phlow-packages.git"
FORK_URL="git@github.com:$(gh api user | jq -r .login)/phlow-packages.git"
BRANCH="publish"
TMP_DIR=$(mktemp -d)
CLONE_DIR="$TMP_DIR/phlow-packages"

# 1. Fork do repositório
echo "🔁 Fazendo fork do repositório..."
gh repo fork $REPO_URL --clone --remote=false

# 2. Clonar a branch 'publish'
echo "📥 Clonando a branch $BRANCH..."
git clone --branch $BRANCH $FORK_URL $CLONE_DIR

cd $CLONE_DIR || exit 1

# 3. Copiar o arquivo para a raiz
echo "📄 Adicionando o arquivo $ARQUIVO à raiz..."
cp "$OLDPWD/$ARQUIVO" .

# 4. Commit e push
git checkout -b "add-$(basename "$ARQUIVO" | sed 's/[^a-zA-Z0-9]/-/g')"
git add "$(basename "$ARQUIVO")"
git commit -m "feat: adiciona $(basename "$ARQUIVO")"
git push --set-upstream origin HEAD

# 5. Criar o PR
echo "🚀 Criando Pull Request..."
gh pr create --title "Adiciona $(basename "$ARQUIVO")" --body "Esse PR adiciona o arquivo $(basename "$ARQUIVO") à raiz do projeto." --base publish

echo "✅ PR criado com sucesso."
