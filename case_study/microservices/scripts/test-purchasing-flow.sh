#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

ORCHESTRATOR_URL="http://localhost:8080"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Purchasing Flow Test Suite${NC}"
echo -e "${BLUE}========================================${NC}\n"

# Test Case 1: No reorder needed (sufficient inventory)
echo -e "${YELLOW}Test 1: No Reorder Needed (PROD-001)${NC}"
echo -e "${BLUE}Expected: sufficient inventory, no PO created${NC}"
curl -s -X POST ${ORCHESTRATOR_URL}/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}' | jq '.'
echo -e "\n"

# Test Case 2: Reorder needed - standard case
echo -e "${YELLOW}Test 2: Reorder Needed - Standard (PROD-002)${NC}"
echo -e "${BLUE}Expected: low inventory, PO created and sent${NC}"
curl -s -X POST ${ORCHESTRATOR_URL}/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-002"}' | jq '.'
echo -e "\n"

# Test Case 3: High value order requiring approval
echo -e "${YELLOW}Test 3: High Value Order - Requires Approval (PROD-003)${NC}"
echo -e "${BLUE}Expected: total > 10000, requires approval${NC}"
curl -s -X POST ${ORCHESTRATOR_URL}/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-003"}' | jq '.'
echo -e "\n"

# Test Case 4: Product with increasing trend
echo -e "${YELLOW}Test 4: Increasing Demand Trend (PROD-004)${NC}"
echo -e "${BLUE}Expected: higher order qty due to trend${NC}"
curl -s -X POST ${ORCHESTRATOR_URL}/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-004"}' | jq '.'
echo -e "\n"

# Test Case 5: Zero inventory
echo -e "${YELLOW}Test 5: Zero Inventory (PROD-005)${NC}"
echo -e "${BLUE}Expected: immediate reorder needed${NC}"
curl -s -X POST ${ORCHESTRATOR_URL}/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-005"}' | jq '.'
echo -e "\n"

# Test Case 6: Product not found
echo -e "${YELLOW}Test 6: Product Not Found (PROD-999)${NC}"
echo -e "${BLUE}Expected: error or default behavior${NC}"
curl -s -X POST ${ORCHESTRATOR_URL}/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-999"}' | jq '.'
echo -e "\n"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  All Tests Completed${NC}"
echo -e "${GREEN}========================================${NC}"
