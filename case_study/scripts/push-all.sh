#!/bin/bash
set -e

# Push all microservices Docker images to registry
# Usage: ./scripts/push-all.sh [REGISTRY]

REGISTRY=${1:-"purchasing-flow"}

echo "Pushing all microservices to registry: $REGISTRY"

services=(
  "oms-service"
  "inventory-service"
  "supplier-service"
  "uom-service"
  "rule-engine-service"
  "po-service"
  "orchestrator-service"
)

for service in "${services[@]}"; do
  echo ""
  echo "==================================="
  echo "Pushing $service..."
  echo "==================================="
  docker push $REGISTRY/$service:latest
  echo "✓ $service pushed successfully"
done

echo ""
echo "==================================="
echo "All services pushed successfully!"
echo "==================================="
