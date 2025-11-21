-- UOM Database (Unit of Measure)

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
