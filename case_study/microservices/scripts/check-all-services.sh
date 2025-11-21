#!/bin/bash
# Check all microservices compile correctly

set -e

SERVICES=(
    "oms-service"
    "inventory-service"
    "supplier-service"
    "uom-service"
    "rule-engine-service"
    "po-service"
    "orchestrator-service"
)

echo "🔍 Checking all microservices..."
echo ""

for service in "${SERVICES[@]}"; do
    echo "📦 Checking $service..."
    cd "services/$service"
    
    if cargo check --quiet 2>&1 | grep -q "error\|^error:"; then
        echo "❌ $service has compilation errors!"
        cargo check 2>&1 | grep -A 5 "error"
        exit 1
    else
        echo "✅ $service compiles successfully"
    fi
    
    cd ../..
    echo ""
done

echo "✅ All services compile successfully!"
echo ""
echo "Next steps:"
echo "1. Build Docker images: docker-compose build"
echo "2. Start services: docker-compose up"
echo "3. Test endpoint: curl -X POST http://localhost:8080/purchasing/flow -H 'Content-Type: application/json' -d '{\"product_id\": \"PROD-001\"}'"
