#!/usr/bin/env bash
# Script to extract all AWS resource types from the OpenTofu/Terraform AWS provider
# This helps identify which resources don't have icons in Agedashi

set -e

# Check if tofu or terraform is available
if command -v tofu &> /dev/null; then
    TF_CMD="tofu"
elif command -v terraform &> /dev/null; then
    TF_CMD="terraform"
else
    echo "Error: Neither 'tofu' nor 'terraform' found in PATH"
    echo "Please install OpenTofu (https://opentofu.org) or Terraform (https://terraform.io)"
    exit 1
fi

echo "Using: $TF_CMD"
echo "Extracting AWS provider schema..."
echo

# Create temporary directory for provider init
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Create minimal terraform config to initialize AWS provider
cat > main.tf <<EOF
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  skip_credentials_validation = true
  skip_metadata_api_check     = true
  skip_requesting_account_id  = true
}
EOF

# Initialize provider
$TF_CMD init -upgrade > /dev/null 2>&1

# Extract all AWS resources
echo "AWS Provider Resources:"
echo "======================="
$TF_CMD providers schema -json | \
  jq -r '
    .provider_schemas[].resource_schemas |
    keys[] |
    select(startswith("aws_"))
  ' | \
  sort | \
  tee "$TEMP_DIR/aws-resources.txt"

TOTAL=$(wc -l < "$TEMP_DIR/aws-resources.txt")
echo
echo "Total: $TOTAL resources"
echo

# Save to project directory
OUTPUT_FILE="$(git rev-parse --show-toplevel 2>/dev/null || echo ".")/test/aws-provider-resources.txt"
cp "$TEMP_DIR/aws-resources.txt" "$OUTPUT_FILE"
echo "Resource list saved to: $OUTPUT_FILE"

# Cleanup
cd - > /dev/null
rm -rf "$TEMP_DIR"
