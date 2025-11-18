#!/bin/bash
set -e

# Build all microservices Docker images
# Usage: ./scripts/build-all.sh [REGISTRY]

REGISTRY=${1:-"purchasing-flow"}

echo "Building all microservices with registry: $REGISTRY"

services=(
  "oms-service"
  "inventory-service"
  "supplier-service"
  "uom-service"
  "rule-engine-service"
  "po-service"
  "orchestrator-service"
)

cd "$(dirname "$0")/.."

for service in "${services[@]}"; do
  echo ""
  echo "==================================="
  echo "Building $service..."
  echo "==================================="
  docker build -f microservices/services/$service/Dockerfile -t $REGISTRY/$service:latest .
  echo "✓ $service built successfully"
done

echo ""
echo "==================================="
echo "All services built successfully!"
echo "==================================="
echo ""
echo "To push images to registry, run:"
echo "  ./scripts/push-all.sh $REGISTRY"
