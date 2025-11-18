#!/bin/bash

set -e

echo "🔨 Building Docker images sequentially to avoid memory issues..."
echo ""

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
    echo "📦 Building $service..."
    docker-compose build "$service"
    echo "✅ $service built successfully"
    echo ""
done

echo "🎉 All services built successfully!"
echo ""
echo "To start the services, run:"
echo "  docker-compose up -d"
