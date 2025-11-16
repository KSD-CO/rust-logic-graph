# Purchasing Flow Example - Quick Start

## TL;DR

```bash
# 0. Configure environment (first-time only)
cd case_study
cp .env.example .env
# Edit .env with your database credentials if needed

# 1. Setup databases (one-time)
./scripts/setup_databases.sh

# 2. Run the example
./scripts/run_realdb.sh
```

## Environment Setup (IMPORTANT)

**Database credentials are now stored in `.env` file for security.**

### Step 1: Create .env file

```bash
cd case_study
cp .env.example .env
```

### Step 2: Edit credentials (if needed)

The `.env.example` already contains demo database credentials. For custom database:

```bash
# Edit .env with your credentials
vim .env  # or nano, code, etc.
```

**Note:** The `.env` file is gitignored and will NOT be committed to version control.

## Database Info

**Default demo database** (pre-configured in `.env.example`):

```
Host: 171.244.10.40:6033
User: lune_dev
Pass: rfSxLLeSqVCGNeGc

Databases:
  - oms_db
  - inventory_db
  - supplier_db
  - uom_db
```

**For your own database:** Edit `case_study/.env` with your credentials.

## Files

| File | Purpose |
|------|---------|
| `purchasing_flow_setup.sql` | Creates databases & test data |
| `purchasing_flow_realdb.rs` | Main example code |
| `setup_databases.sh` | Auto setup script |
| `test_purchasing_flow.sh` | Test & run script |
| `purchasing_flow_README.md` | Full documentation |

## Manual Commands

```bash
# Setup (if script doesn't work)
# First, load credentials from .env
cd case_study
source .env
mysql -h $DB_HOST -P $DB_PORT -u $DB_USER -p"$DB_PASSWORD" \
  < sql/purchasing_flow_setup.sql

# Build
cargo build --bin purchasing_flow_realdb --features mysql

# Run
cargo run --bin purchasing_flow_realdb --features mysql
```

## Test Different Products

Edit in code:
```rust
let product_id = "PROD-002"; // PROD-001, PROD-002, or PROD-003
```

## Expected Output

```
=== Purchasing Flow with Real MySQL Databases ===
Each node connects to a separate database:
  - OMS Node        -> oms_db
  - Inventory Node  -> inventory_db
  - Supplier Node   -> supplier_db
  - UOM Node        -> uom_db

[Database queries...]

Final Purchase Order:
{
  "po_id": "PO-...",
  "product_id": "PROD-001",
  "qty": 100,
  "total_amount": 1599.0,
  "status": "sent"
}
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `.env file not found` | `cp case_study/.env.example case_study/.env` |
| Connection failed | Check `.env` credentials, network, firewall |
| Build error | `cargo clean && cargo build --features mysql` |
| Missing data | Run setup script again |
| No mysql client | `brew install mysql-client` (macOS) |

## Architecture

```
OMS Node (oms_db) ─┐
Inventory (inventory_db) ─┼─> Rule Engine -> Calc -> Create PO -> Send PO
Supplier (supplier_db) ─┤
UOM Node (uom_db) ─┘
```

Each node = Separate database connection

## For More Details

See [purchasing_flow_README.md](purchasing_flow_README.md)
