# Purchasing Flow Example with Real MySQL Databases

This example demonstrates the purchasing flow using the rust-logic-graph framework with **real MySQL databases**. Each node in the graph connects to a separate database to simulate different external systems in a distributed architecture.

## Prerequisites

### Configure Environment Variables (REQUIRED)

**Database credentials are now stored in `.env` file for security**. Before running anything, you must configure your environment:

```bash
# Navigate to case study directory
cd case_study

# Copy the .env template
cp .env.example .env

# Edit .env with your actual database credentials
vim .env  # or nano, code, etc.
```

The `.env` file contains:
```bash
# MySQL Connection Settings
DB_USER=lune_dev
DB_PASSWORD=rfSxLLeSqVCGNeGc
DB_HOST=171.244.10.40
DB_PORT=6033

# Database Names
OMS_DB=oms_db
INVENTORY_DB=inventory_db
SUPPLIER_DB=supplier_db
UOM_DB=uom_db
```

**Important:**
- The `.env` file is in `.gitignore` and will NOT be committed to version control
- Never commit database credentials to git
- For production, use a secrets manager (AWS Secrets Manager, HashiCorp Vault, etc.)
- The helper scripts will automatically check for `.env` file existence

## Architecture Overview

The example models a real-world purchasing flow with the following components:

```
┌─────────────────┐     ┌──────────────────────┐     ┌─────────────────┐
│   OMS Node      │────▶│   Rule Engine Node   │────▶│ Calc Order Qty  │
│  (oms_db)       │     │   (in-memory)        │     │  (in-memory)    │
└─────────────────┘     └──────────────────────┘     └─────────────────┘
                                   ▲                           │
┌─────────────────┐                │                           │
│ Inventory Node  │────────────────┘                           │
│ (inventory_db)  │                                            │
└─────────────────┘                                            ▼
                                                       ┌─────────────────┐
┌─────────────────┐                                   │   Create PO     │
│ Supplier Node   │────────────────┐                  │  (in-memory)    │
│ (supplier_db)   │                │                  └─────────────────┘
└─────────────────┘                │                           │
                                   ▼                           │
┌─────────────────┐     ┌──────────────────────┘              │
│   UOM Node      │────▶│                                     ▼
│  (uom_db)       │     │                            ┌─────────────────┐
└─────────────────┘     │                            │    Send PO      │
                        │                            │  (in-memory)    │
                        └────────────────────────────┴─────────────────┘
```

### Database Distribution

Each data collection node connects to its own MySQL database:

1. **OMS Node** → `oms_db` - Order Management System historical data
2. **Inventory Node** → `inventory_db` - Warehouse inventory levels
3. **Supplier Node** → `supplier_db` - Supplier information and pricing
4. **UOM Node** → `uom_db` - Unit of Measure conversion tables

## Database Setup

### Connection Information

**Note:** Connection details are loaded from `case_study/.env` file (see Prerequisites above).

Default demo database (pre-configured in `.env.example`):
```
Host: 171.244.10.40
Port: 6033
Username: lune_dev
Password: rfSxLLeSqVCGNeGc
```

**To use your own database:** Edit `case_study/.env` with your credentials.

### Step 1: Run the Setup Script

**IMPORTANT**: You must setup the databases before running the example!

#### Option A: Using the helper script (recommended)

```bash
cd case_study
./scripts/setup_databases.sh
```

#### Option B: Manual setup

Execute the SQL setup script directly (ensures .env is configured first):

```bash
cd case_study
source .env
mysql -h $DB_HOST -P $DB_PORT -u $DB_USER -p"$DB_PASSWORD" < sql/purchasing_flow_setup.sql
```

If you don't have MySQL client installed:
- macOS: `brew install mysql-client`
- Ubuntu: `sudo apt-get install mysql-client`
- CentOS: `sudo yum install mysql`

This will create:
- 4 separate databases: `oms_db`, `inventory_db`, `supplier_db`, `uom_db`
- Tables in each database
- Sample test data for products PROD-001, PROD-002, PROD-003

### Step 2: Verify the Setup

Check that all databases and tables are created:

```bash
# Connect to MySQL (using credentials from .env)
cd case_study
source .env
mysql -h $DB_HOST -P $DB_PORT -u $DB_USER -p"$DB_PASSWORD"
```

```sql
-- Check databases
SHOW DATABASES LIKE '%_db';

-- Check OMS data
USE oms_db;
SELECT * FROM oms_history;

-- Check Inventory data
USE inventory_db;
SELECT * FROM inventory_levels;

-- Check Supplier data
USE supplier_db;
SELECT * FROM supplier_info;

-- Check UOM data
USE uom_db;
SELECT * FROM uom_conversion;
```

## Running the Example

### Prerequisites

1. Enable the `mysql` feature in Cargo.toml (already configured)
2. Ensure all databases are set up (see Database Setup above)

### Build and Run

#### Quick Test (Recommended)

Use the test script which checks connectivity and provides helpful feedback:

```bash
./examples/test_purchasing_flow.sh
```

#### Manual Build and Run

```bash
# Build the example
cargo build --example purchasing_flow_realdb --features mysql

# Run the example
cargo run --example purchasing_flow_realdb --features mysql
```

**Note**: If the databases are not set up, the example will fail with connection errors. Make sure to run the setup script first!

### Expected Output

The example will:
1. Connect to all 4 databases independently
2. Query data for product PROD-001
3. Execute the purchasing flow logic
4. Generate a purchase order based on real data

Example output:

```
=== Purchasing Flow with Real MySQL Databases ===
Each node connects to a separate database:
  - OMS Node        -> oms_db
  - Inventory Node  -> inventory_db
  - Supplier Node   -> supplier_db
  - UOM Node        -> uom_db

Creating database connections for each node...
  [oms_history] Connecting to oms_db...
  [inventory_levels] Connecting to inventory_db...
  [supplier_info] Connecting to supplier_db...
  [uom_conversion] Connecting to uom_db...
All database connections established successfully!

Starting graph execution...

[oms_history] Database: oms_db | Executing query: SELECT...
[inventory_levels] Database: inventory_db | Executing query: SELECT...
[supplier_info] Database: supplier_db | Executing query: SELECT...
[uom_conversion] Database: uom_db | Executing query: SELECT...
[calc_order_qty] Calculating order quantity...
[create_po] Creating purchase order...
[send_po] Sending purchase order...

=== Execution Complete ===

Final Purchase Order:
{
  "po_id": "PO-1731715200",
  "product_id": "PROD-001",
  "supplier_id": "SUP-001",
  "qty": 80,
  "unit_price": 15.99,
  "total_amount": 1279.2,
  "status": "sent",
  "created_at": "2024-11-16T03:20:00Z",
  "sent_at": "2024-11-16T03:20:00Z"
}

Calculation Details:
{
  "order_qty": 80,
  "avg_demand": 15.5,
  "available_qty": 20,
  "demand_during_lead_time": 108.5,
  "shortage": 88.5,
  "moq": 20,
  "lead_time_days": 7
}
```

## Business Logic

### Order Quantity Calculation

The system calculates the order quantity based on:

1. **Average Daily Demand** (from OMS)
2. **Lead Time** (from Supplier)
3. **Available Inventory** (from Inventory System)
4. **Minimum Order Quantity (MOQ)** (from Supplier)

Formula:
```
demand_during_lead_time = avg_daily_demand × lead_time_days
shortage = max(0, demand_during_lead_time - available_qty)
order_qty = ceil(shortage / moq) × moq
```

### Example Calculation (PROD-001)

- Average demand: 15.5 units/day
- Lead time: 7 days
- Available inventory: 20 units (25 current - 5 reserved)
- MOQ: 20 units

Calculation:
- Demand during lead time: 15.5 × 7 = 108.5 units
- Shortage: 108.5 - 20 = 88.5 units
- Order quantity: ceil(88.5 / 20) × 20 = 5 × 20 = **100 units**

## Test Data

### Product PROD-001
- **OMS**: avg_demand=15.5, trend=increasing
- **Inventory**: current=25, reserved=5, available=20
- **Supplier**: SUP-001, moq=20, lead_time=7 days, price=$15.99
- **UOM**: CASE to PIECE = 12x

### Product PROD-002
- **OMS**: avg_demand=8.3, trend=stable
- **Inventory**: current=50, reserved=10, available=40
- **Supplier**: SUP-002, moq=50, lead_time=14 days, price=$8.50
- **UOM**: CASE to PIECE = 24x

### Product PROD-003
- **OMS**: avg_demand=22.0, trend=increasing
- **Inventory**: current=10, reserved=3, available=7
- **Supplier**: SUP-003, moq=10, lead_time=5 days, price=$25.00
- **UOM**: PALLET to CASE = 48x

## Customization

### Testing Different Products

Modify the `product_id` variable in `main()`:

```rust
let product_id = "PROD-002"; // Change to test other products
```

### Adding More Products

Insert data into each database:

```sql
-- Add to OMS DB
USE oms_db;
INSERT INTO oms_history (product_id, avg_daily_demand, trend)
VALUES ('PROD-004', 30.0, 'increasing');

-- Add to Inventory DB
USE inventory_db;
INSERT INTO inventory_levels (product_id, warehouse_id, current_qty, reserved_qty)
VALUES ('PROD-004', 'WH-001', 100, 20);

-- Add to Supplier DB
USE supplier_db;
INSERT INTO supplier_info (supplier_id, product_id, moq, lead_time_days, unit_price, is_active)
VALUES ('SUP-004', 'PROD-004', 30, 10, 12.50, TRUE);

-- Add to UOM DB
USE uom_db;
INSERT INTO uom_conversion (product_id, from_uom, to_uom, conversion_factor)
VALUES ('PROD-004', 'BOX', 'PIECE', 6.0000);
```

## Troubleshooting

### Missing .env File

If you see ".env file not found" error:
```bash
cd case_study
cp .env.example .env
# Edit .env with your credentials
```

### Connection Errors

If you see connection errors:
1. **First, ensure `.env` file exists and is properly configured:**
   ```bash
   cd case_study
   cat .env  # Verify credentials are correct
   ```
2. Verify credentials in `.env` match your database
3. Check network connectivity to your database server
4. Ensure MySQL server is running
5. Check firewall settings
6. Test connection manually:
   ```bash
   source .env
   mysql -h $DB_HOST -P $DB_PORT -u $DB_USER -p"$DB_PASSWORD"
   ```

### Missing Data

If queries return empty results:
1. Verify the setup script ran successfully
2. Check that the product_id exists in all databases
3. Ensure `is_active = TRUE` for suppliers

### Dependency Issues

If you see compilation errors:
```bash
# Make sure the mysql feature is enabled
cd case_study
cargo clean
cargo build --bin purchasing_flow_realdb --features mysql
```

## Architecture Benefits

This multi-database architecture demonstrates:

1. **Separation of Concerns**: Each system owns its data
2. **Distributed Systems**: Nodes connect to different data sources
3. **Scalability**: Each database can be scaled independently
4. **Real-world Patterns**: Mimics microservices architecture
5. **Fault Tolerance**: Failure in one DB doesn't affect others

## Next Steps

1. Add error handling for database failures
2. Implement retry logic for transient errors
3. Add caching layer to reduce database calls
4. Implement connection pooling optimization
5. Add monitoring and logging for each database connection
6. Test with concurrent executions
7. Add support for transactions across multiple databases
