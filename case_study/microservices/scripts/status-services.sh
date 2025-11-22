#!/bin/bash

# Script to check status of all services

echo "📊 Microservices Status:"
echo ""

check_service() {
    local name=$1
    local port=$2
    
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "  ✅ $name (port $port) - RUNNING"
    else
        echo "  ❌ $name (port $port) - STOPPED"
    fi
}

check_service "Orchestrator  " 8080
check_service "OMS           " 8081
check_service "Inventory     " 8082
check_service "Supplier      " 8083
check_service "UOM           " 8084
check_service "PO            " 8085
check_service "Rule Engine   " 8086

echo ""
echo "To test services:"
echo "  curl http://localhost:8082/health"
