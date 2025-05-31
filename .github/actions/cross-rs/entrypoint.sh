#!/bin/bash

set -e

TARGET=$1
PROJECT_PATH=$2

echo "Building target: $TARGET"
echo "Project path: $PROJECT_PATH"

cd "$PROJECT_PATH"

cross build --target "$TARGET" --release
