-- Supplier Database

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
