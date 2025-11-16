#!/bin/bash

echo "=== Running Advanced Purchasing Flow (with Monitoring) ==="
echo ""

# Check if .env file exists
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="$SCRIPT_DIR/../.env"

if [ ! -f "$ENV_FILE" ]; then
    echo "✗ .env file not found!"
    echo ""
    echo "Please create a .env file in the case_study directory:"
    echo "  cp case_study/.env.example case_study/.env"
    echo ""
    echo "Then edit .env with your database credentials."
    echo ""
    exit 1
fi

# Load database config from .env file
source "$ENV_FILE"

# Check database connectivity first
DB_HOST="${DB_HOST:-171.244.10.40}"
DB_PORT="${DB_PORT:-6033}"

nc -z -v -w5 $DB_HOST $DB_PORT >/dev/null 2>&1

if [ $? -ne 0 ]; then
    echo "✗ Cannot connect to database server at $DB_HOST:$DB_PORT"
    echo ""
    echo "Please ensure:"
    echo "  1. Database server is running"
    echo "  2. Network connectivity is available"
    echo "  3. Databases are set up (run: ./scripts/setup_databases.sh)"
    echo ""
    exit 1
fi

echo "✓ Database server is reachable"
echo ""

cargo run --manifest-path "$(dirname "$0")/../Cargo.toml" --bin purchasing_flow_advanced --features mysql

echo ""
