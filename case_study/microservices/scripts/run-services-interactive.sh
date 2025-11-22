#!/bin/bash

# Script to run all microservices concurrently
# Each service runs in the background with logging to separate files

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICES_DIR="$SCRIPT_DIR/../services"
LOGS_DIR="$SCRIPT_DIR/../logs"

# Create logs directory if it doesn't exist
mkdir -p "$LOGS_DIR"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "  Starting All Microservices"
echo "=========================================="
echo ""

# Array to store PIDs
declare -a PIDS

# Function to start a service
start_service() {
    local service_name=$1
    local service_dir="$SERVICES_DIR/$service_name"
    local log_file="$LOGS_DIR/${service_name}.log"
    
    if [ ! -d "$service_dir" ]; then
        echo -e "${RED}❌ Service directory not found: $service_name${NC}"
        return 1
    fi
    
    echo -e "${BLUE}🚀 Starting $service_name...${NC}"
    
    # Start service in background, redirect output to log file
    cd "$service_dir"
    cargo run >> "$log_file" 2>&1 &
    local pid=$!
    PIDS+=($pid)
    
    echo -e "${GREEN}✅ $service_name started (PID: $pid)${NC}"
    echo "   Log: $log_file"
}

# Function to check if a port is available
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 1
    else
        return 0
    fi
}

# Function to wait for service to be ready
wait_for_service() {
    local service_name=$1
    local port=$2
    local max_wait=30
    local waited=0
    
    echo -n "   Waiting for $service_name to be ready on port $port"
    
    while [ $waited -lt $max_wait ]; do
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            echo -e " ${GREEN}✓${NC}"
            return 0
        fi
        echo -n "."
        sleep 1
        waited=$((waited + 1))
    done
    
    echo -e " ${YELLOW}⚠${NC} (timeout)"
    return 1
}

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}=========================================="
    echo "  Stopping All Services"
    echo -e "==========================================${NC}"
    echo ""
    
    for pid in "${PIDS[@]}"; do
        if kill -0 $pid 2>/dev/null; then
            echo -e "${YELLOW}⏹  Stopping process $pid${NC}"
            kill $pid 2>/dev/null
        fi
    done
    
    # Wait a bit for graceful shutdown
    sleep 2
    
    # Force kill if still running
    for pid in "${PIDS[@]}"; do
        if kill -0 $pid 2>/dev/null; then
            echo -e "${RED}🔪 Force killing process $pid${NC}"
            kill -9 $pid 2>/dev/null
        fi
    done
    
    echo ""
    echo -e "${GREEN}✅ All services stopped${NC}"
    exit 0
}

# Set up trap to cleanup on Ctrl+C
trap cleanup SIGINT SIGTERM

# Clear old logs
echo "📝 Clearing old logs..."
rm -f "$LOGS_DIR"/*.log

echo ""
echo "Starting services in dependency order..."
echo ""

# Start services in order (some depend on others)
start_service "oms-service"
wait_for_service "oms-service" 50051

start_service "inventory-service"
wait_for_service "inventory-service" 50052

start_service "supplier-service"
wait_for_service "supplier-service" 50053

start_service "uom-service"
wait_for_service "uom-service" 50054

start_service "po-service"
wait_for_service "po-service" 50055

start_service "rule-engine-service"
wait_for_service "rule-engine-service" 50056

# Start orchestrator last (depends on all others)
start_service "orchestrator-service"
wait_for_service "orchestrator-service" 50050

echo ""
echo -e "${GREEN}=========================================="
echo "  All Services Running!"
echo -e "==========================================${NC}"
echo ""
echo "Service Status:"
echo "  • Orchestrator:   http://localhost:8080  (gRPC: 50050)"
echo "  • OMS:            http://localhost:8081  (gRPC: 50051)"
echo "  • Inventory:      http://localhost:8082  (gRPC: 50052)"
echo "  • Supplier:       http://localhost:8083  (gRPC: 50053)"
echo "  • UOM:            http://localhost:8084  (gRPC: 50054)"
echo "  • PO:             http://localhost:8085  (gRPC: 50055)"
echo "  • Rule Engine:    http://localhost:8086  (gRPC: 50056)"
echo ""
echo "Logs are being written to: $LOGS_DIR"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
echo ""

# Keep script running and show live logs
echo "=========================================="
echo "  Live Logs (Ctrl+C to stop)"
echo "=========================================="
echo ""

# Tail all log files
tail -f "$LOGS_DIR"/*.log &
TAIL_PID=$!

# Wait for user interrupt
wait $TAIL_PID
