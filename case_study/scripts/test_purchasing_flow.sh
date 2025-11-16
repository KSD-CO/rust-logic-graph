#!/bin/bash

# Test script for purchasing_flow_realdb example
# This script will attempt to run the example and provide helpful feedback

echo "=== Testing Purchasing Flow Example ==="
echo ""

# Check if the example is built
if [ ! -f "target/debug/examples/purchasing_flow_realdb" ]; then
    echo "Building the example..."
    cargo build --example purchasing_flow_realdb --features mysql
    if [ $? -ne 0 ]; then
        echo "✗ Build failed!"
        exit 1
    fi
    echo "✓ Build successful!"
    echo ""
fi

# Test database connectivity
echo "Testing database connectivity..."
echo ""

DB_HOST="171.244.10.40"
DB_PORT="6033"

# Simple connectivity test using nc (netcat)
nc -z -v -w5 $DB_HOST $DB_PORT >/dev/null 2>&1

if [ $? -eq 0 ]; then
    echo "✓ Database server is reachable at $DB_HOST:$DB_PORT"
    echo ""
    echo "Running the purchasing flow example..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    cargo run --example purchasing_flow_realdb --features mysql

    EXIT_CODE=$?
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if [ $EXIT_CODE -eq 0 ]; then
        echo "✓ Example completed successfully!"
    else
        echo "✗ Example failed with exit code: $EXIT_CODE"
        echo ""
        echo "Possible issues:"
        echo "  1. Database setup not completed - run: ./examples/setup_databases.sh"
        echo "  2. Authentication failed - check credentials"
        echo "  3. Missing data - verify test data exists in databases"
    fi
else
    echo "✗ Cannot connect to database server at $DB_HOST:$DB_PORT"
    echo ""
    echo "The database server appears to be unreachable."
    echo "Possible reasons:"
    echo "  1. Network connectivity issue"
    echo "  2. Firewall blocking the connection"
    echo "  3. Database server is down"
    echo "  4. Wrong host/port configuration"
    echo ""
    echo "To test the example logic without a real database,"
    echo "you can run the mock version:"
    echo "  cargo run --example purchasing_flow"
fi

echo ""
