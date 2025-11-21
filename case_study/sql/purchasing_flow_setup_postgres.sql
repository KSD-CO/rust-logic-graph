-- Database setup for purchasing_flow example (PostgreSQL version)

-- Each node uses a separate database to simulate different external systems

-- ========================================
-- DATABASE 1: OMS (Order Management System)
-- ========================================
DROP DATABASE IF EXISTS oms_db;
CREATE DATABASE oms_db;

\c oms_db;

CREATE TABLE IF NOT EXISTS oms_history (
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
-- DATABASE 2: Inventory System
-- ========================================
\c postgres;
DROP DATABASE IF EXISTS inventory_db;
CREATE DATABASE inventory_db;

\c inventory_db;

CREATE TABLE IF NOT EXISTS inventory_levels (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    warehouse_id VARCHAR(50),
    available_qty DECIMAL(10,2) NOT NULL,
    reserved_qty DECIMAL(10,2) DEFAULT 0,
    warehouse_location VARCHAR(100),
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert test data for Inventory Levels
INSERT INTO inventory_levels (product_id, warehouse_id, available_qty, reserved_qty, warehouse_location) VALUES
('PROD-001', 'WH-01', 50.0, 10.0, 'Building A, Shelf 12'),
('PROD-002', 'WH-01', 120.0, 5.0, 'Building B, Shelf 5'),
('PROD-003', 'WH-02', 8.0, 2.0, 'Building C, Shelf 3');

-- ========================================
-- DATABASE 3: Supplier System
-- ========================================
\c postgres;
DROP DATABASE IF EXISTS supplier_db;
CREATE DATABASE supplier_db;

\c supplier_db;

CREATE TABLE IF NOT EXISTS supplier_info (
    id SERIAL PRIMARY KEY,
    supplier_id VARCHAR(50),
    product_id VARCHAR(50) NOT NULL,
    moq DECIMAL(10,2) NOT NULL,
    lead_time_days INT NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert test data for Supplier Info
INSERT INTO supplier_info (supplier_id, product_id, moq, lead_time_days, unit_price, is_active) VALUES
('SUPP-A', 'PROD-001', 20.0, 7, 12.50, TRUE),
('SUPP-B', 'PROD-002', 50.0, 5, 8.00, TRUE),
('SUPP-C', 'PROD-003', 30.0, 10, 15.75, TRUE);

-- ========================================
-- DATABASE 4: UOM (Unit of Measure) System
-- ========================================
\c postgres;
DROP DATABASE IF EXISTS uom_db;
CREATE DATABASE uom_db;

\c uom_db;

CREATE TABLE IF NOT EXISTS uom_conversions (
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
