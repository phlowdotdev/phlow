#!/usr/bin/env bash

# Obter o diretório deste script:
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

npx autocannon \
  -c 1 \
  -d 10 \
  -m POST \
  -H "Content-Type: application/json" \
  -b "$(cat "$SCRIPT_DIR/data.json")" \
  http://localhost:3000/data?test=1
