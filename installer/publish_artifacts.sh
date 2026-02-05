#!/usr/bin/env bash
set -euo pipefail
: "${S3_BUCKET:?Need S3_BUCKET}"
: "${AWS_ACCESS_KEY_ID:?Need AWS_ACCESS_KEY_ID}"
: "${AWS_SECRET_ACCESS_KEY:?Need AWS_SECRET_ACCESS_KEY}"
aws s3 cp dist/ "s3://${S3_BUCKET}/plus/" --recursive
