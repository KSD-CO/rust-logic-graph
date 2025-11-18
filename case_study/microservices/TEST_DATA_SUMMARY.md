# Test Data Summary

## Overview

Test data has been created to demonstrate all scenarios in the purchasing flow system, including:
- No reorder needed
- Standard reorder
- High-value orders requiring approval
- Products with increasing demand trends
- Zero inventory situations
- Edge cases (product not found)

## Test Products

### PROD-001: No Reorder Scenario
**Purpose**: Demonstrate sufficient inventory case

| Attribute | Value |
|-----------|-------|
| **OMS Data** | |
| Avg Daily Demand | 10 units |
| Trend | stable |
| **Inventory** | |
| Current Qty | 100 units |
| Reserved Qty | 0 units |
| Available Qty | 100 units |
| Warehouse | WH-001 |
| **Supplier** | |
| Supplier ID | SUP-001 |
| MOQ | 50 units |
| Lead Time | 7 days |
| Unit Price | $25.00 |

**Expected Behavior**:
- Rule engine calculates: 100 available > 10 demand
- No reorder needed
- No PO created
- Response: `{"need_reorder": false}`

---

### PROD-002: Standard Reorder Scenario
**Purpose**: Demonstrate normal reorder flow

| Attribute | Value |
|-----------|-------|
| **OMS Data** | |
| Avg Daily Demand | 50 units |
| Trend | stable |
| **Inventory** | |
| Current Qty | 5 units |
| Reserved Qty | 0 units |
| Available Qty | 5 units |
| Warehouse | WH-001 |
| **Supplier** | |
| Supplier ID | SUP-002 |
| MOQ | 100 units |
| Lead Time | 5 days |
| Unit Price | $15.50 |

**Expected Behavior**:
- Rule engine calculates: 5 available < 50 demand
- Shortage: 45 units
- Order qty: 100 units (MOQ)
- Total amount: $1,550.00
- PO created and sent
- Response: `{"need_reorder": true, "po": {...}}`

---

### PROD-003: High-Value Approval Scenario
**Purpose**: Demonstrate approval workflow for expensive orders

| Attribute | Value |
|-----------|-------|
| **OMS Data** | |
| Avg Daily Demand | 200 units |
| Trend | stable |
| **Inventory** | |
| Current Qty | 1 unit |
| Reserved Qty | 0 units |
| Available Qty | 1 unit |
| Warehouse | WH-001 |
| **Supplier** | |
| Supplier ID | SUP-003 |
| MOQ | 50 units |
| Lead Time | 10 days |
| Unit Price | $120.00 |

**Expected Behavior**:
- Rule engine calculates: 1 available << 200 demand
- Huge shortage: 199 units
- Order qty: 200 units (to cover demand + safety stock)
- Total amount: **$24,000.00** (exceeds $10k threshold)
- **Requires approval**: true
- Approval status: pending
- PO created but NOT sent (awaits approval)

---

### PROD-004: Increasing Trend Scenario
**Purpose**: Demonstrate adaptive ordering based on trends

| Attribute | Value |
|-----------|-------|
| **OMS Data** | |
| Avg Daily Demand | 80 units |
| Trend | **increasing** |
| **Inventory** | |
| Current Qty | 10 units |
| Reserved Qty | 0 units |
| Available Qty | 10 units |
| Warehouse | WH-001 |
| **Supplier** | |
| Supplier ID | SUP-004 |
| MOQ | 75 units |
| Lead Time | 7 days |
| Unit Price | $35.00 |

**Expected Behavior**:
- Rule engine detects increasing trend
- Calculates higher safety stock
- Order qty: 75+ units (MOQ + trend adjustment)
- Total amount: $2,625+
- PO created and sent
- Response shows trend consideration in order_qty

---

### PROD-005: Zero Inventory Scenario
**Purpose**: Demonstrate urgent reorder for stockout

| Attribute | Value |
|-----------|-------|
| **OMS Data** | |
| Avg Daily Demand | 30 units |
| Trend | stable |
| **Inventory** | |
| Current Qty | **0 units** |
| Reserved Qty | 0 units |
| Available Qty | **0 units** |
| Warehouse | WH-001 |
| **Supplier** | |
| Supplier ID | SUP-005 |
| MOQ | 60 units |
| Lead Time | 5 days |
| Unit Price | $22.50 |

**Expected Behavior**:
- Rule engine detects stockout
- Critical shortage: 30 units
- Order qty: 60 units (MOQ)
- Total amount: $1,350.00
- Priority/urgent flag may be set
- PO created and sent immediately

---

## Business Rules Tested

### Rule 1: Reorder Point Calculation
```
IF available_qty < (avg_daily_demand * lead_time_days)
THEN need_reorder = true
```

Tested by: PROD-002, PROD-003, PROD-004, PROD-005

### Rule 2: Order Quantity Calculation
```
order_qty = MAX(shortage, MOQ)
```

Tested by: All reorder scenarios

### Rule 3: Approval Threshold
```
IF total_amount > 10000
THEN requires_approval = true
AND approval_status = "pending"
```

Tested by: PROD-003

### Rule 4: Trend Adjustment
```
IF trend = "increasing"
THEN order_qty *= 1.2  (or similar adjustment)
```

Tested by: PROD-004

### Rule 5: Stockout Priority
```
IF available_qty = 0
THEN priority = "urgent"
```

Tested by: PROD-005

## UOM (Unit of Measure) Conversions

The UOM service provides unit conversions for all products:

| From Unit | To Unit | Factor | Example |
|-----------|---------|--------|---------|
| pieces | box | 0.1 | 10 pieces = 1 box |
| box | pieces | 10.0 | 1 box = 10 pieces |
| box | case | 0.2 | 5 boxes = 1 case |
| case | box | 5.0 | 1 case = 5 boxes |

## Test Execution

### Expected Flow for Each Test

1. **Client sends request** to Orchestrator (HTTP)
2. **Orchestrator makes parallel gRPC calls**:
   - OMS Service → Get demand data
   - Inventory Service → Get stock levels
   - Supplier Service → Get supplier info
   - UOM Service → Get unit conversions
3. **Orchestrator aggregates data** and calls Rule Engine (gRPC)
4. **Rule Engine evaluates** business rules using rust-logic-graph
5. **If reorder needed**:
   - Orchestrator calls PO Service to create PO (gRPC)
   - Orchestrator calls PO Service to send PO (gRPC)
6. **Orchestrator returns response** to client (HTTP)

### Verification Points

For each test case, verify:

✅ All gRPC connections successful
✅ Data retrieved from all services
✅ Rule engine evaluation correct
✅ Order quantity matches MOQ or calculated need
✅ Total amount calculated correctly
✅ Approval flag set when amount > $10k
✅ PO created with correct data
✅ PO sent (or pending if approval needed)

## Database Verification

### Check OMS Data
```sql
SELECT * FROM oms_db.sales_history WHERE product_id = 'PROD-001';
```

### Check Inventory
```sql
SELECT * FROM inventory_db.inventory_levels WHERE product_id = 'PROD-001';
```

### Check Supplier
```sql
SELECT * FROM supplier_db.supplier_catalog WHERE product_id = 'PROD-001';
```

## API Response Examples

### Success - No Reorder (PROD-001)
```json
{
  "success": true,
  "po": null,
  "calculation": {
    "need_reorder": false,
    "shortage": 0.0,
    "order_qty": 0,
    "total_amount": 0.0,
    "requires_approval": false,
    "approval_status": "unknown"
  },
  "message": "No reorder needed"
}
```

### Success - Reorder Created (PROD-002)
```json
{
  "success": true,
  "po": {
    "po_id": "PO-1732010123",
    "product_id": "PROD-002",
    "supplier_id": "SUP-002",
    "qty": 100,
    "unit_price": 15.50,
    "total_amount": 1550.00,
    "status": "sent",
    "created_at": "2025-11-18T15:48:43Z",
    "sent_at": "2025-11-18T15:48:43Z"
  },
  "calculation": {
    "need_reorder": true,
    "shortage": 45.0,
    "order_qty": 100,
    "total_amount": 1550.00,
    "requires_approval": false,
    "approval_status": "approved"
  },
  "message": "PO created and sent successfully"
}
```

### Success - Approval Required (PROD-003)
```json
{
  "success": true,
  "po": {
    "po_id": "PO-1732010234",
    "product_id": "PROD-003",
    "supplier_id": "SUP-003",
    "qty": 200,
    "unit_price": 120.00,
    "total_amount": 24000.00,
    "status": "draft",
    "created_at": "2025-11-18T15:50:34Z",
    "sent_at": null
  },
  "calculation": {
    "need_reorder": true,
    "shortage": 199.0,
    "order_qty": 200,
    "total_amount": 24000.00,
    "requires_approval": true,
    "approval_status": "pending"
  },
  "message": "PO created, awaiting approval"
}
```

## Performance Benchmarks

Expected response times (with all services local):

- **PROD-001** (no reorder): ~50-100ms
  - 4 parallel gRPC calls + rule evaluation

- **PROD-002** (standard reorder): ~100-150ms
  - 4 parallel gRPC calls + rule evaluation + 2 PO gRPC calls

- **PROD-003** (approval required): ~100-150ms
  - Similar to PROD-002 but PO not sent

## Troubleshooting Test Failures

### All tests return errors
- Check if all services are running
- Check MySQL container is running
- Verify gRPC ports are correct

### Specific product test fails
- Check database for that product's data
- Verify rule engine is evaluating correctly
- Check service logs in `/tmp/*.log`

### PO not created when expected
- Check Rule Engine service logs
- Verify shortage calculation
- Check PO Service is reachable on port 50056
