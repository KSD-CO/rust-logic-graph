#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Setup Test Environment${NC}"
echo -e "${BLUE}========================================${NC}\n"

# Step 1: Check if MySQL container is running
echo -e "${YELLOW}Checking MySQL container...${NC}"
if ! docker ps | grep -q purchasing-mysql; then
    echo -e "${RED}MySQL container not running. Starting it...${NC}"

    # Start MySQL container
    docker run -d \
        --name purchasing-mysql \
        -e MYSQL_ROOT_PASSWORD=password \
        -p 3306:3306 \
        mysql:8.0

    echo -e "${YELLOW}Waiting for MySQL to be ready...${NC}"
    sleep 20
else
    echo -e "${GREEN}MySQL container already running${NC}"
fi

# Step 2: Create databases if they don't exist
echo -e "\n${YELLOW}Creating databases...${NC}"
docker exec -i purchasing-mysql mysql -uroot -ppassword <<EOF
CREATE DATABASE IF NOT EXISTS oms_db;
CREATE DATABASE IF NOT EXISTS inventory_db;
CREATE DATABASE IF NOT EXISTS supplier_db;
CREATE DATABASE IF NOT EXISTS uom_db;
EOF

# Step 3: Create tables
echo -e "${YELLOW}Creating tables...${NC}"

# OMS tables
docker exec -i purchasing-mysql mysql -uroot -ppassword oms_db <<EOF
CREATE TABLE IF NOT EXISTS sales_history (
    id INT AUTO_INCREMENT PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    avg_daily_demand DECIMAL(10,2) NOT NULL,
    trend VARCHAR(20) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY unique_product (product_id)
);
EOF

# Inventory tables
docker exec -i purchasing-mysql mysql -uroot -ppassword inventory_db <<EOF
CREATE TABLE IF NOT EXISTS inventory_levels (
    id INT AUTO_INCREMENT PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    warehouse_id VARCHAR(50) NOT NULL,
    current_qty INT NOT NULL,
    reserved_qty INT NOT NULL,
    available_qty INT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY unique_product_warehouse (product_id, warehouse_id)
);
EOF

# Supplier tables
docker exec -i purchasing-mysql mysql -uroot -ppassword supplier_db <<EOF
CREATE TABLE IF NOT EXISTS supplier_catalog (
    id INT AUTO_INCREMENT PRIMARY KEY,
    supplier_id VARCHAR(50) NOT NULL,
    product_id VARCHAR(50) NOT NULL,
    moq INT NOT NULL,
    lead_time_days INT NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY unique_supplier_product (supplier_id, product_id)
);
EOF

# UOM tables
docker exec -i purchasing-mysql mysql -uroot -ppassword uom_db <<EOF
CREATE TABLE IF NOT EXISTS unit_conversions (
    id INT AUTO_INCREMENT PRIMARY KEY,
    from_unit VARCHAR(20) NOT NULL,
    to_unit VARCHAR(20) NOT NULL,
    factor DECIMAL(10,4) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY unique_conversion (from_unit, to_unit)
);
EOF

echo -e "${GREEN}Tables created successfully${NC}"

# Step 4: Insert test data
echo -e "\n${YELLOW}Inserting test data...${NC}"
docker exec -i purchasing-mysql mysql -uroot -ppassword < /Users/jamesvu/Documents/Personals/rust-logic-graph/case_study/microservices/scripts/setup-test-data.sql

echo -e "${GREEN}Test data inserted successfully${NC}"

# Step 5: Verify data
echo -e "\n${YELLOW}Verifying data...${NC}"
docker exec purchasing-mysql mysql -uroot -ppassword -e "
SELECT 'OMS Data:' as info;
SELECT * FROM oms_db.sales_history;
SELECT 'Inventory Data:' as info;
SELECT * FROM inventory_db.inventory_levels;
SELECT 'Supplier Data:' as info;
SELECT * FROM supplier_db.supplier_catalog;
"

echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}  Setup Complete!${NC}"
echo -e "${GREEN}========================================${NC}\n"

echo -e "${BLUE}You can now run the test suite:${NC}"
echo -e "${YELLOW}  chmod +x /Users/jamesvu/Documents/Personals/rust-logic-graph/case_study/microservices/scripts/test-purchasing-flow.sh${NC}"
echo -e "${YELLOW}  /Users/jamesvu/Documents/Personals/rust-logic-graph/case_study/microservices/scripts/test-purchasing-flow.sh${NC}"
