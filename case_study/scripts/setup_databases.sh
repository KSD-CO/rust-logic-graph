#!/bin/bash

# Database setup script for purchasing_flow example
# This script creates the databases and tables needed for the example

DB_HOST="171.244.10.40"
DB_PORT="6033"
DB_USER="lune_dev"
DB_PASS="rfSxLLeSqVCGNeGc"

echo "=== Purchasing Flow Database Setup ==="
echo "Host: $DB_HOST:$DB_PORT"
echo "User: $DB_USER"
echo ""

# Check if mysql client is available
if ! command -v mysql &> /dev/null; then
    echo "ERROR: mysql client not found!"
    echo ""
    echo "Please install MySQL client:"
    echo "  macOS:   brew install mysql-client"
    echo "  Ubuntu:  sudo apt-get install mysql-client"
    echo "  CentOS:  sudo yum install mysql"
    echo ""
    echo "Alternatively, you can run the SQL script manually:"
    echo "  mysql -h $DB_HOST -P $DB_PORT -u $DB_USER -p'$DB_PASS' < examples/purchasing_flow_setup.sql"
    exit 1
fi

# Run the setup SQL script
echo "Creating databases and tables..."
mysql -h "$DB_HOST" -P "$DB_PORT" -u "$DB_USER" -p"$DB_PASS" < examples/purchasing_flow_setup.sql

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Database setup completed successfully!"
    echo ""
    echo "Created databases:"
    echo "  - oms_db"
    echo "  - inventory_db"
    echo "  - supplier_db"
    echo "  - uom_db"
    echo ""
    echo "You can now run the example:"
    echo "  cargo run --example purchasing_flow_realdb --features mysql"
else
    echo ""
    echo "✗ Database setup failed!"
    echo "Please check your connection settings and try again."
    exit 1
fi
