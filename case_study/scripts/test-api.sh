#!/bin/bash
set -e

# Test the Purchasing Flow API
# Usage: ./scripts/test-api.sh [HOST] [PORT]

HOST=${1:-"localhost"}
PORT=${2:-"8080"}
BASE_URL="http://$HOST:$PORT"

echo "Testing Purchasing Flow API at $BASE_URL"
echo ""

# Health check
echo "==================================="
echo "1. Health Check"
echo "==================================="
curl -s "$BASE_URL/health" | jq .
echo ""

# Execute purchasing flow
echo "==================================="
echo "2. Execute Purchasing Flow"
echo "==================================="
curl -s -X POST "$BASE_URL/purchasing/flow" \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}' | jq .
echo ""

echo "==================================="
echo "API test complete!"
echo "==================================="
