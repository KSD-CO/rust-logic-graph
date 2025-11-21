#!/bin/bash
# Test purchasing flow with Docker services

set -e

ORCHESTRATOR_URL="http://localhost:8080"

echo "🧪 Testing Purchasing Flow Microservices"
echo "=========================================="
echo ""

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
for i in {1..30}; do
    if curl -s "${ORCHESTRATOR_URL}/health" > /dev/null 2>&1; then
        echo "✅ Orchestrator is ready!"
        break
    fi
    echo "   Attempt $i/30: Orchestrator not ready yet..."
    sleep 2
done

echo ""
echo "🧪 Test 1: PROD-001 (Low demand, sufficient inventory)"
echo "------------------------------------------------"
curl -s -X POST ${ORCHESTRATOR_URL}/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}' | jq -r 'if .calculation then 
    "Need reorder: \(.calculation.need_reorder)
Shortage: \(.calculation.shortage)
Order qty: \(.calculation.order_qty)
Total: $\(.calculation.total_amount)
PO: \(if .po then .po.po_id else "Not created" end)" 
  else . end'

echo ""
echo ""
echo "🧪 Test 2: PROD-002 (Standard reorder)"
echo "------------------------------------------------"
curl -s -X POST ${ORCHESTRATOR_URL}/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-002"}' | jq -r 'if .calculation then 
    "Need reorder: \(.calculation.need_reorder)
Shortage: \(.calculation.shortage)
Order qty: \(.calculation.order_qty)
Total: $\(.calculation.total_amount)
Requires approval: \(.calculation.requires_approval)
PO: \(if .po then .po.po_id + " (\(.po.status))" else "Not created" end)"
  else . end'

echo ""
echo ""
echo "🧪 Test 3: PROD-003 (High value - requires approval)"
echo "------------------------------------------------"
curl -s -X POST ${ORCHESTRATOR_URL}/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-003"}' | jq -r 'if .calculation then
    "Need reorder: \(.calculation.need_reorder)
Shortage: \(.calculation.shortage)
Order qty: \(.calculation.order_qty)
Total: $\(.calculation.total_amount)
Requires approval: \(.calculation.requires_approval)
Approval status: \(.calculation.approval_status)
PO: \(if .po then .po.po_id + " (\(.po.status))" else "Not created" end)"
  else . end'

echo ""
echo ""
echo "✅ All tests completed!"
echo ""
echo "📊 View service logs:"
echo "  docker-compose logs -f orchestrator-service"
echo "  docker-compose logs -f rule-engine-service"
echo ""
echo "🛑 Stop services:"
echo "  docker-compose down"
