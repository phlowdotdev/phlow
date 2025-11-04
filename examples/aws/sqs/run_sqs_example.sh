#!/usr/bin/env bash
set -euo pipefail

QUEUE_NAME="phlow-sqs-demo"
REGION="us-east-1"
ENDPOINT="http://localhost:4566"

echo "[SQS Demo] Using LocalStack endpoint: ${ENDPOINT}"

# Check dependencies
if ! command -v aws >/dev/null 2>&1; then
  echo "Error: AWS CLI (aws) not found. Please install AWS CLI to run this script." >&2
  exit 1
fi

# Create queue if not exists
if aws --endpoint-url="${ENDPOINT}" --region "${REGION}" sqs get-queue-url --queue-name "${QUEUE_NAME}" >/dev/null 2>&1; then
  echo "[SQS Demo] Queue '${QUEUE_NAME}' already exists"
else
  echo "[SQS Demo] Creating queue '${QUEUE_NAME}'..."
  aws --endpoint-url="${ENDPOINT}" --region "${REGION}" sqs create-queue --queue-name "${QUEUE_NAME}" >/dev/null
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

# Show number of messages available
QURL=$(aws --endpoint-url="${ENDPOINT}" --region "${REGION}" sqs get-queue-url --queue-name "${QUEUE_NAME}" --output text --query 'QueueUrl')
aws --endpoint-url="${ENDPOINT}" --region "${REGION}" sqs get-queue-attributes --queue-url "$QURL" --attribute-names ApproximateNumberOfMessages || true

echo "[SQS Demo] Done."
