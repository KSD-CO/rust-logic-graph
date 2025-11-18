#!/bin/bash

# Test script for Purchasing Flow
# Tests both Monolithic and Microservices architectures

echo "=== Testing Purchasing Flow ==="
echo ""

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="$SCRIPT_DIR/../.env"

# Check if .env exists
if [ ! -f "$ENV_FILE" ]; then
    echo "✗ .env file not found!"
    echo ""
    echo "Please create .env file:"
    echo "  cp case_study/.env.example case_study/.env"
    echo ""
    exit 1
fi

# Load environment
source "$ENV_FILE"

# Test database connectivity
echo "Testing database connectivity..."
DB_HOST="${DB_HOST:-171.244.10.40}"
DB_PORT="${DB_PORT:-6033}"

nc -z -v -w5 $DB_HOST $DB_PORT >/dev/null 2>&1

if [ $? -eq 0 ]; then
    echo "✓ Database server is reachable at $DB_HOST:$DB_PORT"
    echo ""
else
    echo "✗ Cannot connect to database server at $DB_HOST:$DB_PORT"
    echo ""
    echo "Please ensure:"
    echo "  1. Database server is running"
    echo "  2. Databases are set up: ./scripts/setup_databases.sh"
    echo ""
    exit 1
fi

# Choose test type
echo "Select test type:"
echo "  1) Monolithic (Clean Architecture)"
echo "  2) Microservices (Docker Compose)"
echo "  3) Both"
echo ""
read -p "Enter choice [1-3]: " choice

case $choice in
    1)
        echo ""
        echo "==================================="
        echo "Testing Monolithic Architecture"
        echo "==================================="
        echo ""
        cd "$SCRIPT_DIR/../monolithic"
        cargo run --bin purchasing_flow --features mysql
        ;;
    2)
        echo ""
        echo "==================================="
        echo "Testing Microservices Architecture"
        echo "==================================="
        echo ""
        echo "Starting services with Docker Compose..."
        cd "$SCRIPT_DIR/../microservices"
        docker-compose up -d

        echo "Waiting for services to be ready..."
        sleep 10

        echo ""
        echo "Testing API..."
        curl -s -X POST http://localhost:8080/purchasing/flow \
          -H "Content-Type: application/json" \
          -d '{"product_id": "PROD-001"}' | jq .

        echo ""
        echo "View logs with:"
        echo "  docker-compose logs -f orchestrator-service"
        ;;
    3)
        echo ""
        echo "==================================="
        echo "Testing Monolithic Architecture"
        echo "==================================="
        echo ""
        cd "$SCRIPT_DIR/../monolithic"
        cargo run --bin purchasing_flow --features mysql

        echo ""
        echo "==================================="
        echo "Testing Microservices Architecture"
        echo "==================================="
        echo ""
        cd "$SCRIPT_DIR/../microservices"
        docker-compose up -d
        sleep 10

        curl -s -X POST http://localhost:8080/purchasing/flow \
          -H "Content-Type: application/json" \
          -d '{"product_id": "PROD-001"}' | jq .
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "==================================="
echo "Test complete!"
echo "==================================="
echo ""
