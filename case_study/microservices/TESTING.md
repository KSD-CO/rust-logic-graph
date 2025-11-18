# Testing Guide for Purchasing Flow Microservices

## Quick Start

### 1. Setup Database and Test Data

```bash
./scripts/setup-and-test.sh
```

This script will:
- Start MySQL container in Docker
- Create all required databases (oms_db, inventory_db, supplier_db, uom_db)
- Create all tables
- Insert test data for 5 different test scenarios

### 2. Start All Microservices

```bash
./scripts/start-all-services.sh
```

This will start all 7 services:
- OMS Service (HTTP: 8081, gRPC: 50051)
- Inventory Service (HTTP: 8082, gRPC: 50052)
- Supplier Service (HTTP: 8083, gRPC: 50053)
- UOM Service (HTTP: 8084, gRPC: 50054)
- Rule Engine Service (HTTP: 8085, gRPC: 50055)
- PO Service (HTTP: 8086, gRPC: 50056)
- Orchestrator Service (HTTP: 8080)

### 3. Run Test Suite

```bash
./scripts/test-purchasing-flow.sh
```

Or the start-all-services.sh script will automatically run tests after starting services.

## Test Cases

### Test Case 1: No Reorder Needed (PROD-001)
**Scenario**: Product has sufficient inventory
- Inventory: 100 units available
- Daily demand: 10 units
- Expected: `need_reorder = false`, no PO created

```bash
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

**Expected Response**:
```json
{
  "success": true,
  "po": null,
  "calculation": {
    "need_reorder": false,
    "shortage": 0.0,
    "order_qty": 0,
    "total_amount": 0.0
  },
  "message": "No reorder needed"
}
```

### Test Case 2: Standard Reorder (PROD-002)
**Scenario**: Low inventory, standard reorder
- Inventory: 5 units available
- Daily demand: 50 units
- Supplier MOQ: 100 units @ $15.50 each
- Expected: `need_reorder = true`, PO created and sent

```bash
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-002"}'
```

**Expected Response**:
```json
{
  "success": true,
  "po": {
    "po_id": "PO-...",
    "product_id": "PROD-002",
    "supplier_id": "SUP-002",
    "qty": 100,
    "unit_price": 15.50,
    "total_amount": 1550.00,
    "status": "sent"
  },
  "calculation": {
    "need_reorder": true,
    "shortage": 45.0,
    "order_qty": 100,
    "total_amount": 1550.00,
    "requires_approval": false
  }
}
```

### Test Case 3: High Value Order - Requires Approval (PROD-003)
**Scenario**: Very low inventory, expensive product, total > $10,000
- Inventory: 1 unit available
- Daily demand: 200 units
- Supplier: MOQ 50 @ $120.00 = **$6,000+**
- Expected: `requires_approval = true`

```bash
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-003"}'
```

**Expected Response**:
```json
{
  "calculation": {
    "requires_approval": true,
    "approval_status": "pending",
    "total_amount": 24000.00
  }
}
```

### Test Case 4: Increasing Demand Trend (PROD-004)
**Scenario**: Product with increasing demand trend
- Inventory: 10 units
- Daily demand: 80 units (increasing trend)
- Expected: Higher order quantity to accommodate trend

```bash
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-004"}'
```

### Test Case 5: Zero Inventory (PROD-005)
**Scenario**: Completely out of stock
- Inventory: 0 units
- Daily demand: 30 units
- Expected: Immediate reorder needed

```bash
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-005"}'
```

### Test Case 6: Product Not Found (PROD-999)
**Scenario**: Product doesn't exist in any database
- Expected: Error or default behavior

```bash
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-999"}'
```

## Manual Testing

### Check Individual Services

#### OMS Service
```bash
# REST API
curl http://localhost:8081/oms/data/PROD-001

# gRPC (using grpcurl)
grpcurl -plaintext -d '{"product_id": "PROD-001"}' \
  localhost:50051 oms.OmsService/GetData
```

#### Inventory Service
```bash
# REST API
curl http://localhost:8082/inventory/levels/PROD-001

# gRPC
grpcurl -plaintext -d '{"product_id": "PROD-001"}' \
  localhost:50052 inventory.InventoryService/GetLevels
```

#### Supplier Service
```bash
# REST API
curl http://localhost:8083/supplier/info/SUP-001/PROD-001

# gRPC
grpcurl -plaintext -d '{"supplier_id": "SUP-001", "product_id": "PROD-001"}' \
  localhost:50053 supplier.SupplierService/GetInfo
```

#### Rule Engine Service
```bash
# gRPC
grpcurl -plaintext -d '{
  "oms_data": {"product_id": "PROD-002", "avg_daily_demand": 50, "trend": "stable"},
  "inventory_data": {"product_id": "PROD-002", "available_qty": 5},
  "supplier_data": {"product_id": "PROD-002", "moq": 100, "unit_price": 15.50}
}' localhost:50055 rule_engine.RuleEngineService/Evaluate
```

#### PO Service
```bash
# gRPC - Create PO
grpcurl -plaintext -d '{
  "product_id": "PROD-002",
  "supplier_id": "SUP-002",
  "qty": 100,
  "unit_price": 15.50,
  "total_amount": 1550.00
}' localhost:50056 po.PoService/Create
```

## Health Checks

Check all services are running:

```bash
curl http://localhost:8081/health  # OMS
curl http://localhost:8082/health  # Inventory
curl http://localhost:8083/health  # Supplier
curl http://localhost:8084/health  # UOM
curl http://localhost:8085/health  # Rule Engine
curl http://localhost:8086/health  # PO
curl http://localhost:8080/health  # Orchestrator
```

## Viewing Logs

Service logs are written to `/tmp`:

```bash
tail -f /tmp/oms-service.log
tail -f /tmp/inventory-service.log
tail -f /tmp/supplier-service.log
tail -f /tmp/uom-service.log
tail -f /tmp/rule-engine-service.log
tail -f /tmp/po-service.log
tail -f /tmp/orchestrator-service.log
```

## Stopping Services

```bash
# Stop all running cargo processes
pkill -f 'cargo run'

# Stop MySQL container
docker stop purchasing-mysql
docker rm purchasing-mysql
```

## Database Access

Access MySQL directly:

```bash
# Connect to MySQL
docker exec -it purchasing-mysql mysql -uroot -ppassword

# View data
USE oms_db;
SELECT * FROM sales_history;

USE inventory_db;
SELECT * FROM inventory_levels;

USE supplier_db;
SELECT * FROM supplier_catalog;
```

## Architecture

### Communication Patterns

1. **External Client → Orchestrator**: HTTP REST
2. **Orchestrator → Services**: gRPC (high performance)
3. **Services → Database**: MySQL (via sqlx)
4. **Rule Engine**: Uses rust-logic-graph library

### Data Flow

```
Client Request (HTTP)
    ↓
Orchestrator (8080)
    ↓ (gRPC calls in parallel)
    ├→ OMS Service (50051)
    ├→ Inventory Service (50052)
    ├→ Supplier Service (50053)
    └→ UOM Service (50054)
    ↓
Orchestrator aggregates data
    ↓ (gRPC call)
Rule Engine Service (50055)
    ↓ (uses rust-logic-graph)
Business Rules Evaluation
    ↓
If reorder needed:
    ↓ (gRPC call)
PO Service (50056) - Create PO
    ↓ (gRPC call)
PO Service (50056) - Send PO
    ↓
Return result to Client (HTTP)
```

## Troubleshooting

### MySQL Connection Issues

```bash
# Check if MySQL is running
docker ps | grep mysql

# Restart MySQL
docker restart purchasing-mysql

# View MySQL logs
docker logs purchasing-mysql
```

### Service Not Starting

```bash
# Check if port is already in use
lsof -i :8080
lsof -i :50051

# Kill process using port
kill -9 <PID>
```

### gRPC Connection Errors

Verify all services are listening on correct ports:

```bash
lsof -i :50051  # OMS gRPC
lsof -i :50052  # Inventory gRPC
lsof -i :50053  # Supplier gRPC
lsof -i :50054  # UOM gRPC
lsof -i :50055  # Rule Engine gRPC
lsof -i :50056  # PO gRPC
```
