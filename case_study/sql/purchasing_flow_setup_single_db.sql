-- Database setup for purchasing_flow example (PostgreSQL - Single Database)
-- Run this with: psql -h 127.0.0.1 -U rustfire -d purchasing_flow -f purchasing_flow_setup_single_db.sql

-- ========================================
-- OMS (Order Management System)
-- ========================================
DROP TABLE IF EXISTS oms_history CASCADE;

CREATE TABLE oms_history (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    avg_daily_demand DECIMAL(10,2) NOT NULL,
    trend VARCHAR(20),
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert test data for OMS History
INSERT INTO oms_history (product_id, avg_daily_demand, trend) VALUES
('PROD-001', 15.5, 'increasing'),
('PROD-002', 8.3, 'stable'),
('PROD-003', 22.0, 'increasing');

-- ========================================
-- Inventory System
-- ========================================
DROP TABLE IF EXISTS inventory CASCADE;

CREATE TABLE inventory (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    warehouse_id VARCHAR(50),
    available_qty DECIMAL(10,2) NOT NULL,
    reserved_qty DECIMAL(10,2) DEFAULT 0,
    warehouse_location VARCHAR(100),
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert test data for Inventory Levels
INSERT INTO inventory (product_id, warehouse_id, available_qty, reserved_qty, warehouse_location) VALUES
('PROD-001', 'WH-01', 50.0, 10.0, 'Building A, Shelf 12'),
('PROD-002', 'WH-01', 120.0, 5.0, 'Building B, Shelf 5'),
('PROD-003', 'WH-02', 8.0, 2.0, 'Building C, Shelf 3');

-- ========================================
-- Supplier System
-- ========================================
DROP TABLE IF EXISTS suppliers CASCADE;

CREATE TABLE suppliers (
    id SERIAL PRIMARY KEY,
    supplier_id VARCHAR(50),
    product_id VARCHAR(50) NOT NULL,
    moq DECIMAL(10,2) NOT NULL,
    lead_time INT NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    supplier_name VARCHAR(100),
    is_active BOOLEAN DEFAULT TRUE,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert test data for Supplier Info
INSERT INTO suppliers (supplier_id, product_id, moq, lead_time, unit_price, supplier_name, is_active) VALUES
('SUPP-A', 'PROD-001', 20.0, 7, 12.50, 'Supplier Alpha', TRUE),
('SUPP-B', 'PROD-002', 50.0, 5, 8.00, 'Supplier Beta', TRUE),
('SUPP-C', 'PROD-003', 30.0, 10, 15.75, 'Supplier Gamma', TRUE);

-- ========================================
-- UOM (Unit of Measure) System
-- ========================================
DROP TABLE IF EXISTS uom_conversions CASCADE;

CREATE TABLE uom_conversions (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    from_uom VARCHAR(20) NOT NULL,
    to_uom VARCHAR(20) NOT NULL,
    conversion_factor DECIMAL(10,4) NOT NULL,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert test data for UOM Conversions
INSERT INTO uom_conversions (product_id, from_uom, to_uom, conversion_factor) VALUES
('PROD-001', 'pieces', 'boxes', 12.0000),
('PROD-002', 'kg', 'g', 1000.0000),
('PROD-003', 'liters', 'ml', 1000.0000);

-- Verify data
SELECT 'OMS History' as table_name, COUNT(*) as row_count FROM oms_history
UNION ALL
SELECT 'Inventory', COUNT(*) FROM inventory
UNION ALL
SELECT 'Suppliers', COUNT(*) FROM suppliers
UNION ALL
SELECT 'UOM Conversions', COUNT(*) FROM uom_conversions;
