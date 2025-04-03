#!/usr/bin/env bash

# Obter o diretório deste script:
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

npx autocannon \
  -c 10000 \
  -d 10 \
  -m GET \
  http://localhost:3000/ping
