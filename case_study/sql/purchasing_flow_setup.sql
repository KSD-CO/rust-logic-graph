-- Database setup for purchasing_flow example

-- Each node uses a separate database to simulate different external systems

-- ========================================
-- DATABASE 1: OMS (Order Management System)
-- ========================================
CREATE DATABASE IF NOT EXISTS oms_db;
USE oms_db;

CREATE TABLE IF NOT EXISTS oms_history (
    id INT AUTO_INCREMENT PRIMARY KEY,
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
CREATE DATABASE IF NOT EXISTS inventory_db;
USE inventory_db;
CREATE TABLE IF NOT EXISTS inventory_levels (
    id INT AUTO_INCREMENT PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    warehouse_id VARCHAR(50) NOT NULL,
    current_qty INT NOT NULL,
    reserved_qty INT DEFAULT 0,
    available_qty INT GENERATED ALWAYS AS (current_qty - reserved_qty) STORED,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert test data for Inventory Levels
INSERT INTO inventory_levels (product_id, warehouse_id, current_qty, reserved_qty) VALUES
('PROD-001', 'WH-001', 25, 5),
('PROD-002', 'WH-001', 50, 10),
('PROD-003', 'WH-001', 10, 3);

-- ========================================
-- DATABASE 3: Supplier Management System
-- ========================================
CREATE DATABASE IF NOT EXISTS supplier_db;
USE supplier_db;
CREATE TABLE IF NOT EXISTS supplier_info (
    id INT AUTO_INCREMENT PRIMARY KEY,
    supplier_id VARCHAR(50) NOT NULL,
    product_id VARCHAR(50) NOT NULL,
    moq INT NOT NULL, -- Minimum Order Quantity
    lead_time_days INT NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert test data for Supplier Info
INSERT INTO supplier_info (supplier_id, product_id, moq, lead_time_days, unit_price, is_active) VALUES
('SUP-001', 'PROD-001', 20, 7, 15.99, TRUE),
('SUP-002', 'PROD-002', 50, 14, 8.50, TRUE),
('SUP-003', 'PROD-003', 10, 5, 25.00, TRUE);

-- ========================================
-- DATABASE 4: UOM (Unit of Measure) System
-- ========================================
CREATE DATABASE IF NOT EXISTS uom_db;
USE uom_db;
CREATE TABLE IF NOT EXISTS uom_conversion (
    id INT AUTO_INCREMENT PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    from_uom VARCHAR(20) NOT NULL,
    to_uom VARCHAR(20) NOT NULL,
    conversion_factor DECIMAL(10,4) NOT NULL,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert test data for UOM Conversion
INSERT INTO uom_conversion (product_id, from_uom, to_uom, conversion_factor) VALUES
('PROD-001', 'CASE', 'PIECE', 12.0000),
('PROD-002', 'CASE', 'PIECE', 24.0000),
('PROD-003', 'PALLET', 'CASE', 48.0000);

-- ========================================
-- Summary and Verification
-- ========================================

-- Display data counts from each database
USE oms_db;
SELECT 'OMS Database - oms_history' as info, COUNT(*) as row_count FROM oms_history;

USE inventory_db;
SELECT 'Inventory Database - inventory_levels' as info, COUNT(*) as row_count FROM inventory_levels;

USE supplier_db;
SELECT 'Supplier Database - supplier_info' as info, COUNT(*) as row_count FROM supplier_info;

USE uom_db;
SELECT 'UOM Database - uom_conversion' as info, COUNT(*) as row_count FROM uom_conversion;
