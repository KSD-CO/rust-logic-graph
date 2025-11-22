#!/bin/bash

# Simple script to run all services in background
# Each service logs to logs/ directory

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Create logs directory
mkdir -p logs

echo "🚀 Starting all microservices..."
echo ""

# Function to start a service
start_service() {
    local service=$1
    local port=$2
    
    echo "  Starting $service..."
    cd "services/$service"
    cargo run > "../../logs/$service.log" 2>&1 &
    echo $! > "../../logs/$service.pid"
    cd - > /dev/null
}

# Start all services
start_service "oms-service" 8081
start_service "inventory-service" 8082
start_service "supplier-service" 8083
start_service "uom-service" 8084
start_service "po-service" 8085
start_service "rule-engine-service" 8086
start_service "orchestrator-service" 8080

echo ""
echo "✅ All services starting..."
echo ""
echo "Wait 10-15 seconds for services to be ready"
echo ""
echo "Logs: logs/*.log"
echo "PIDs: logs/*.pid"
echo ""
echo "To stop all services:"
echo "  ./scripts/stop-all-services.sh"
echo ""
echo "To view logs:"
echo "  tail -f logs/inventory-service.log"
