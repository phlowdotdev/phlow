#!/usr/bin/env bash
set -euo pipefail

REGION="us-east-1"
ENDPOINT="http://localhost:4566"

echo "[SQS Demo] Using LocalStack endpoint: ${ENDPOINT}"

# Check dependencies
if ! command -v aws >/dev/null 2>&1; then
  echo "Error: AWS CLI (aws) not found. Please install AWS CLI to run this script." >&2
  exit 1
fi

# Run the phlow example
if command -v phlow >/dev/null 2>&1; then
  echo "[SQS Demo] Running: phlow examples/aws/sqs/main.phlow"
  phlow examples/aws/sqs/main.phlow
else
  echo "[SQS Demo] 'phlow' command not found. Trying cargo run -p phlow-runtime ..."
  cargo run -p phlow-runtime -- examples/aws/sqs/main.phlow || {
    echo "Error: Could not run phlow runtime. Ensure 'phlow' is installed or build with cargo." >&2
    exit 1
  }
fi

echo "[SQS Demo] Done."
