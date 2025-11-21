#!/bin/bash
# Build all Docker images for microservices

set -e

echo "🐳 Building Docker images for all microservices..."
echo ""

# Build with docker-compose
cd "$(dirname "$0")/.."

echo "📦 Building images (this may take 10-15 minutes)..."
docker-compose build

echo ""
echo "✅ All Docker images built successfully!"
echo ""
echo "Next steps:"
echo "1. Start services: docker-compose up"
echo "2. Or start in background: docker-compose up -d"
echo "3. View logs: docker-compose logs -f"
echo "4. Stop services: docker-compose down"
