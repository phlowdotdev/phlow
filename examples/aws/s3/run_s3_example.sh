#!/usr/bin/env bash
set -euo pipefail

BUCKET="phlow-demo"
REGION="us-east-1"
ENDPOINT="http://localhost:4566"

echo "[S3 Demo] Using LocalStack endpoint: ${ENDPOINT}"

# Check dependencies
if ! command -v aws >/dev/null 2>&1; then
  echo "Error: AWS CLI (aws) not found. Please install AWS CLI to run this script." >&2
  exit 1
fi

# Create bucket if not exists
if aws --endpoint-url="${ENDPOINT}" s3api head-bucket --bucket "${BUCKET}" 2>/dev/null; then
  echo "[S3 Demo] Bucket '${BUCKET}' already exists"
else
  echo "[S3 Demo] Creating bucket '${BUCKET}' in region ${REGION}..."
  # Create-bucket for us-east-1 typically should not pass LocationConstraint
  aws --endpoint-url="${ENDPOINT}" --region "${REGION}" s3api create-bucket --bucket "${BUCKET}" || true
fi

# Run the phlow example
if command -v phlow >/dev/null 2>&1; then
  echo "[S3 Demo] Running: phlow examples/aws/s3/main.phlow"
  phlow examples/aws/s3/main.phlow
else
  echo "[S3 Demo] 'phlow' command not found. Trying cargo run -p phlow-runtime ..."
  cargo run -p phlow-runtime -- examples/aws/s3/main.phlow || {
    echo "Error: Could not run phlow runtime. Ensure 'phlow' is installed or build with cargo." >&2
    exit 1
  }
fi

# List buckets and objects to verify
echo "[S3 Demo] Buckets:"
aws --endpoint-url="${ENDPOINT}" s3 ls || true

echo "[S3 Demo] Objects in s3://${BUCKET}/:" 
aws --endpoint-url="${ENDPOINT}" s3 ls "s3://${BUCKET}/" || true

echo "[S3 Demo] Done."
