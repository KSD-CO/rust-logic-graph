-- OMS Database (Order Management System)

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
