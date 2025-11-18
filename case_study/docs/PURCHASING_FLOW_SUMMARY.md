# Purchasing Flow Example - Summary

> ⚠️ **OUTDATED DOCUMENTATION - v1.0**
>
> This summary references old files and structure that no longer exist.
>
> **For current documentation:**
> - **[Main README](../README.md)** - Current project overview
> - **[GRPC.md](../GRPC.md)** - gRPC implementation (NEW)
> - **[MICROSERVICES_DEPLOYMENT.md](../MICROSERVICES_DEPLOYMENT.md)** - K8s deployment
>
> **Current structure:** Dual architecture (Monolithic + Microservices) with gRPC + REST

---

## ⚠️ Historical Content Below (For Reference Only)

## Overview

Successfully upgraded the `purchasing_flow.rs` example to use **real MySQL databases** instead of mock data. Each node in the graph connects to a separate database to simulate a real microservices architecture.

## Files Created

### 1. Database Setup
- **[purchasing_flow_setup.sql](purchasing_flow_setup.sql)** - SQL script to create 4 databases and test data
  - `oms_db` - Order Management System
  - `inventory_db` - Inventory Management
  - `supplier_db` - Supplier Management
  - `uom_db` - Unit of Measure conversions

### 2. Application Code
- **[purchasing_flow_realdb.rs](purchasing_flow_realdb.rs)** - Rust example with real MySQL connections
  - Custom `MySQLDBNode` - each node has its own connection pool
  - Business logic nodes: `CalcOrderQty`, `CreatePO`, `SendPO`
  - Connects to 4 independent databases

### 3. Helper Scripts
- **[setup_databases.sh](setup_databases.sh)** - Automated database setup script
- **[run_realdb.sh](run_realdb.sh)** - Script to run the real database version
- **[run_advanced.sh](run_advanced.sh)** - Script to run advanced version with monitoring

### 4. Documentation
- **[purchasing_flow_README.md](purchasing_flow_README.md)** - Complete documentation
- **[PURCHASING_FLOW_SUMMARY.md](PURCHASING_FLOW_SUMMARY.md)** - This file

## Architecture

### Database Distribution

```
┌─────────────────────────────────────────────────────────────┐
│                    Rust Logic Graph                         │
└─────────────────────────────────────────────────────────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
    ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐
    │ OMS    │    │Inventory│   │Supplier│    │  UOM   │
    │ Node   │    │  Node   │   │  Node  │    │  Node  │
    └────────┘    └────────┘    └────────┘    └────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
    ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐
    │ oms_db │    │inventory│   │supplier│    │ uom_db │
    │        │    │  _db    │   │  _db   │    │        │
    └────────┘    └────────┘    └────────┘    └────────┘
```

Each node:
- Has its own connection pool
- Queries its own database
- Technically independent
- Simulates real external systems

## Database Information

**Note:** Database credentials are now stored in `.env` file (not committed to git) for security.

### Step 1: Create .env file

```bash
cd case_study
cp .env.example .env
# Edit .env with your credentials if needed
```

The `.env` file contains:
```bash
# MySQL Connection Settings
DB_USER=user
DB_PASSWORD=pass
DB_HOST=IP
DB_PORT=3306

# Database Names
OMS_DB=oms_db
INVENTORY_DB=inventory_db
SUPPLIER_DB=supplier_db
UOM_DB=uom_db
```

**Default demo database** :

### Databases & Tables

1. **oms_db**
   - Table: `oms_history`
   - Contains: order history, avg demand, trend

2. **inventory_db**
   - Table: `inventory_levels`
   - Contains: current inventory, reserved, available

3. **supplier_db**
   - Table: `supplier_info`
   - Contains: supplier information, MOQ, lead time, pricing

4. **uom_db**
   - Table: `uom_conversion`
   - Contains: conversion factors between units

## Business Logic

### Order Quantity Calculation

```
demand_during_lead_time = avg_daily_demand × lead_time_days
shortage = max(0, demand_during_lead_time - available_qty)
order_qty = ceil(shortage / moq) × moq
```

### Example with PROD-001

**Input from databases:**
- OMS: avg_demand = 15.5 units/day, trend = increasing
- Inventory: current = 25, reserved = 5, available = 20
- Supplier: moq = 20, lead_time = 7 days, price = $15.99
- UOM: CASE to PIECE = 12x

**Calculation:**
- Demand during lead time: 15.5 × 7 = 108.5 units
- Shortage: 108.5 - 20 = 88.5 units
- Order quantity: ceil(88.5 / 20) × 20 = 100 units
- Total amount: 100 × $15.99 = $1,599

## How to Use

### Step 0: Configure Environment (REQUIRED)

```bash
# Create .env file
cd case_study
cp .env.example .env

# Edit credentials if needed (default is OK for demo)
vim .env
```

### Step 1: Setup Databases

```bash
# Run setup script (automatically uses credentials from .env)
./scripts/setup_databases.sh

# Or manual (ensure .env is loaded first)
source .env
mysql -h $DB_HOST -P $DB_PORT -u $DB_USER -p"$DB_PASSWORD" \
  < sql/purchasing_flow_setup.sql
```

### Step 2: Run Example

```bash
# Quick test (recommended - automatically checks .env)
./scripts/run_realdb.sh

# Or run directly
cargo run --bin purchasing_flow_realdb --features mysql
```

### Step 3: Verify Output

The example will output:
1. Connection status for 4 databases
2. Query results from each database
3. Calculation details
4. Final Purchase Order (JSON)

## Test Data

3 products available for testing:

| Product  | Avg Demand | Stock | MOQ | Lead Time | Price  |
|----------|-----------|-------|-----|-----------|--------|
| PROD-001 | 15.5/day  | 20    | 20  | 7 days    | $15.99 |
| PROD-002 | 8.3/day   | 40    | 50  | 14 days   | $8.50  |
| PROD-003 | 22.0/day  | 7     | 10  | 5 days    | $25.00 |

To test a different product, edit in code:

```rust
let product_id = "PROD-002"; // Change here
```

## Key Features

1. **Distributed Database Architecture**
   - Each node = 1 separate database
   - Simulates real microservices
   - Independent scaling

2. **Real MySQL Connections**
   - Connection pooling with sqlx
   - Async/await queries
   - Error handling

3. **Business Logic**
   - Demand forecasting
   - Inventory calculation
   - MOQ compliance
   - PO generation

4. **Production-Ready Patterns**
   - Separation of concerns
   - Testable architecture
   - Clear data flow

## Troubleshooting

### Missing .env File

```bash
# Create .env file
cd case_study
cp .env.example .env
```

### Connection Failed

```bash
# Test connectivity (load credentials from .env)
source .env
nc -z -v $DB_HOST $DB_PORT

# Check credentials
mysql -h $DB_HOST -P $DB_PORT -u $DB_USER -p"$DB_PASSWORD"
```

### Missing Data

```sql
-- Verify data exists
USE oms_db;
SELECT * FROM oms_history WHERE product_id = 'PROD-001';

USE inventory_db;
SELECT * FROM inventory_levels WHERE product_id = 'PROD-001';
```

### Build Errors

```bash
# Clean rebuild
cd case_study
cargo clean
cargo build --bin purchasing_flow_realdb --features mysql
```

## Next Steps

Possible extensions:

1. **Error Handling**
   - Retry logic
   - Circuit breaker
   - Fallback strategies

2. **Performance**
   - Connection pool tuning
   - Query optimization
   - Caching layer

3. **Monitoring**
   - Query timing
   - Connection metrics
   - Business metrics

4. **Testing**
   - Integration tests
   - Load testing
   - Concurrent executions

5. **Advanced Features**
   - Transaction support
   - Event sourcing
   - Audit logging

## Conclusion

This example demonstrates:
- ✅ Real database integration with secure credential management
- ✅ Distributed architecture
- ✅ Production-ready patterns
- ✅ Scalable design
- ✅ Complete documentation
- ✅ Security best practices (.env file)

Ready to run and test with real databases!
