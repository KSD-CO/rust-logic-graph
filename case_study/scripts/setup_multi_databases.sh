#!/bin/bash

# Multi-database setup script for purchasing_flow example
# This creates 4 separate databases, simulating microservices architecture

DB_HOST="127.0.0.1"
DB_PORT="5432"
DB_USER="jamesvu"
DB_PASS=""

# PostgreSQL binary path
PSQL="/opt/homebrew/opt/postgresql@17/bin/psql"

# Database names
OMS_DB="oms_db"
INVENTORY_DB="inventory_db"
SUPPLIER_DB="supplier_db"
UOM_DB="uom_db"

echo "=== Purchasing Flow - Multi-Database Setup ==="
echo "Host: $DB_HOST:$DB_PORT"
echo "User: $DB_USER"
echo ""
echo "Creating 4 separate databases:"
echo "  1. $OMS_DB - Order Management System history"
echo "  2. $INVENTORY_DB - Inventory levels"
echo "  3. $SUPPLIER_DB - Supplier information"
echo "  4. $UOM_DB - Unit of Measurement conversions"
echo ""

# Function to create database if not exists
create_db() {
    local db_name=$1
    echo "📦 Creating database: $db_name"
    PGPASSWORD="$DB_PASS" $PSQL -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "CREATE DATABASE $db_name OWNER $DB_USER;" 2>/dev/null
    if [ $? -eq 0 ]; then
        echo "✅ Database $db_name created"
    else
        echo "ℹ️  Database $db_name already exists"
    fi
}

# Create all databases
create_db "$OMS_DB"
create_db "$INVENTORY_DB"
create_db "$SUPPLIER_DB"
create_db "$UOM_DB"

echo ""
echo "=== Creating tables and inserting sample data ==="
echo ""

# Setup OMS Database
echo "📊 Setting up OMS database..."
PGPASSWORD="$DB_PASS" $PSQL -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$OMS_DB" <<EOF
-- OMS History Table
CREATE TABLE IF NOT EXISTS oms_history (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    avg_daily_demand DECIMAL(10,2) NOT NULL,
    trend VARCHAR(20) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample data
INSERT INTO oms_history (product_id, avg_daily_demand, trend) VALUES
    ('PROD-001', 150.00, 'increasing'),
    ('PROD-002', 80.50, 'stable'),
    ('PROD-003', 200.00, 'decreasing')
ON CONFLICT DO NOTHING;

SELECT 'OMS History' as table_name, COUNT(*) as row_count FROM oms_history;
EOF

# Setup Inventory Database
echo "📊 Setting up Inventory database..."
PGPASSWORD="$DB_PASS" $PSQL -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$INVENTORY_DB" <<EOF
-- Inventory Table
CREATE TABLE IF NOT EXISTS inventory (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    available_qty DECIMAL(10,2) NOT NULL,
    reserved_qty DECIMAL(10,2) NOT NULL DEFAULT 0,
    warehouse_location VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample data
INSERT INTO inventory (product_id, available_qty, reserved_qty, warehouse_location) VALUES
    ('PROD-001', 500.00, 100.00, 'WH-A-01'),
    ('PROD-002', 300.00, 50.00, 'WH-B-02'),
    ('PROD-003', 150.00, 0.00, 'WH-C-03')
ON CONFLICT DO NOTHING;

SELECT 'Inventory' as table_name, COUNT(*) as row_count FROM inventory;
EOF

# Setup Supplier Database
echo "📊 Setting up Supplier database..."
PGPASSWORD="$DB_PASS" $PSQL -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$SUPPLIER_DB" <<EOF
-- Suppliers Table
CREATE TABLE IF NOT EXISTS suppliers (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    supplier_name VARCHAR(200) NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    moq DECIMAL(10,2) NOT NULL,
    lead_time INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample data
INSERT INTO suppliers (product_id, supplier_name, unit_price, moq, lead_time) VALUES
    ('PROD-001', 'ABC Supplies Co.', 15.50, 100.00, 7),
    ('PROD-002', 'XYZ Manufacturing', 25.00, 50.00, 14),
    ('PROD-003', 'Global Parts Ltd.', 42.75, 200.00, 21)
ON CONFLICT DO NOTHING;

SELECT 'Suppliers' as table_name, COUNT(*) as row_count FROM suppliers;
EOF

# Setup UOM Database
echo "📊 Setting up UOM database..."
PGPASSWORD="$DB_PASS" $PSQL -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$UOM_DB" <<EOF
-- UOM Conversions Table
CREATE TABLE IF NOT EXISTS uom_conversions (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    from_uom VARCHAR(20) NOT NULL,
    to_uom VARCHAR(20) NOT NULL,
    conversion_factor DECIMAL(10,4) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample data
INSERT INTO uom_conversions (product_id, from_uom, to_uom, conversion_factor) VALUES
    ('PROD-001', 'pieces', 'box', 12.0000),
    ('PROD-002', 'kg', 'lb', 2.2046),
    ('PROD-003', 'liter', 'gallon', 0.2642)
ON CONFLICT DO NOTHING;

SELECT 'UOM Conversions' as table_name, COUNT(*) as row_count FROM uom_conversions;
EOF

echo ""
echo "✅ Multi-database setup complete!"
echo ""
echo "Databases created:"
echo "  - $OMS_DB (oms_history table)"
echo "  - $INVENTORY_DB (inventory table)"
echo "  - $SUPPLIER_DB (suppliers table)"
echo "  - $UOM_DB (uom_conversions table)"
echo ""
echo "Each database has 3 sample products: PROD-001, PROD-002, PROD-003"
echo ""
echo "To verify:"
echo "  PGPASSWORD=$DB_PASS psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $OMS_DB -c 'SELECT * FROM oms_history;'"
echo "  PGPASSWORD=$DB_PASS psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $INVENTORY_DB -c 'SELECT * FROM inventory;'"
echo "  PGPASSWORD=$DB_PASS psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $SUPPLIER_DB -c 'SELECT * FROM suppliers;'"
echo "  PGPASSWORD=$DB_PASS psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $UOM_DB -c 'SELECT * FROM uom_conversions;'"
