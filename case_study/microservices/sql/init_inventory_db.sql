-- Inventory Database

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
