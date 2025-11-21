#!/bin/bash
# Build all microservices sequentially with progress tracking

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

echo "========================================="
echo "Building 7 Microservices"
echo "Estimated time: 30-40 minutes total"
echo "========================================="
echo ""

START_TIME=$(date +%s)

for i in "${!SERVICES[@]}"; do
  SERVICE="${SERVICES[$i]}"
  NUM=$((i + 1))
  
  echo "[$NUM/7] Building $SERVICE..."
  echo "Started at: $(date '+%H:%M:%S')"
  
  SERVICE_START=$(date +%s)
  
  if docker compose build "$SERVICE" 2>&1 | tee "build-$SERVICE.log" | grep -E "CACHED|DONE|ERROR" > /dev/null; then
    SERVICE_END=$(date +%s)
    DURATION=$((SERVICE_END - SERVICE_START))
    echo "✓ $SERVICE completed in ${DURATION}s"
    echo ""
  else
    echo "✗ $SERVICE failed! Check build-$SERVICE.log"
    exit 1
  fi
done

END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))
MINUTES=$((TOTAL_DURATION / 60))
SECONDS=$((TOTAL_DURATION % 60))

echo "========================================="
echo "✓ All services built successfully!"
echo "Total time: ${MINUTES}m ${SECONDS}s"
echo "========================================="
echo ""
echo "Next steps:"
echo "1. Start services: docker compose up -d"
echo "2. Check status: docker compose ps"
echo "3. Test flow: ./scripts/test-docker-flow.sh"
