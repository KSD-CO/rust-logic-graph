#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SERVICES_DIR="/Users/jamesvu/Documents/Personals/rust-logic-graph/case_study/microservices/services"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Starting All Microservices${NC}"
echo -e "${BLUE}========================================${NC}\n"

# Function to start a service
start_service() {
    local service_name=$1
    local http_port=$2
    local grpc_port=$3

    echo -e "${YELLOW}Starting ${service_name}...${NC}"

    cd "${SERVICES_DIR}/${service_name}"

    # Set environment variables
    export PORT=${http_port}
    export GRPC_PORT=${grpc_port}
    export DB_HOST=localhost
    export DB_PORT=3306
    export DB_USER=root
    export DB_PASSWORD=password

    # Start service in background
    cargo run > "/tmp/${service_name}.log" 2>&1 &
    local pid=$!

    echo -e "${GREEN}  ${service_name} started (PID: ${pid})${NC}"
    echo -e "${BLUE}  HTTP: ${http_port}, gRPC: ${grpc_port}${NC}"
    echo ""

    sleep 2
}

# Start all services
start_service "oms-service" 8081 50051
start_service "inventory-service" 8082 50052
start_service "supplier-service" 8083 50053
start_service "uom-service" 8084 50054
start_service "rule-engine-service" 8085 50055
start_service "po-service" 8086 50056

# Start orchestrator last
echo -e "${YELLOW}Starting orchestrator-service...${NC}"
cd "${SERVICES_DIR}/orchestrator-service"
export PORT=8080
export OMS_GRPC_URL="http://localhost:50051"
export INVENTORY_GRPC_URL="http://localhost:50052"
export SUPPLIER_GRPC_URL="http://localhost:50053"
export UOM_GRPC_URL="http://localhost:50054"
export RULE_ENGINE_GRPC_URL="http://localhost:50055"
export PO_GRPC_URL="http://localhost:50056"

cargo run > /tmp/orchestrator-service.log 2>&1 &
echo -e "${GREEN}  orchestrator-service started (PID: $!)${NC}"
echo -e "${BLUE}  HTTP: 8080${NC}"
echo ""

echo -e "${YELLOW}Waiting for all services to be ready...${NC}"
sleep 5

echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}  All Services Started!${NC}"
echo -e "${GREEN}========================================${NC}\n"

echo -e "${BLUE}Service Status:${NC}"
echo -e "  OMS Service:          http://localhost:8081  (gRPC: 50051)"
echo -e "  Inventory Service:    http://localhost:8082  (gRPC: 50052)"
echo -e "  Supplier Service:     http://localhost:8083  (gRPC: 50053)"
echo -e "  UOM Service:          http://localhost:8084  (gRPC: 50054)"
echo -e "  Rule Engine Service:  http://localhost:8085  (gRPC: 50055)"
echo -e "  PO Service:           http://localhost:8086  (gRPC: 50056)"
echo -e "  Orchestrator:         http://localhost:8080"
echo ""

echo -e "${BLUE}View logs:${NC}"
echo -e "  tail -f /tmp/oms-service.log"
echo -e "  tail -f /tmp/orchestrator-service.log"
echo ""

echo -e "${BLUE}Stop all services:${NC}"
echo -e "  pkill -f 'cargo run' # or use kill command with PIDs"
echo ""

echo -e "${YELLOW}Run test suite in 5 seconds...${NC}"
sleep 5

echo -e "\n${BLUE}========================================${NC}"
echo -e "${BLUE}  Running Test Suite${NC}"
echo -e "${BLUE}========================================${NC}\n"

/Users/jamesvu/Documents/Personals/rust-logic-graph/case_study/microservices/scripts/test-purchasing-flow.sh
