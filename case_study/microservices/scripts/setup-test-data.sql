-- Setup Test Data for Purchasing Flow Test Cases

-- =====================================================
-- OMS Database - Order Management Data
-- =====================================================
USE oms_db;

-- Clear existing data
DELETE FROM oms_history where id > 0;

-- Test Case 1: PROD-001 - No reorder needed (sufficient inventory)
INSERT INTO oms_history (product_id, avg_daily_demand, trend) VALUES
('PROD-001', 10.0, 'stable');

-- Test Case 2: PROD-002 - Reorder needed (standard case)
INSERT INTO oms_history (product_id, avg_daily_demand, trend) VALUES
('PROD-002', 50.0, 'stable');

-- Test Case 3: PROD-003 - High value order requiring approval
INSERT INTO oms_history (product_id, avg_daily_demand, trend) VALUES
('PROD-003', 200.0, 'stable');

-- Test Case 4: PROD-004 - Increasing demand trend
INSERT INTO oms_history (product_id, avg_daily_demand, trend) VALUES
('PROD-004', 80.0, 'increasing');

-- Test Case 5: PROD-005 - Zero inventory
INSERT INTO oms_history (product_id, avg_daily_demand, trend) VALUES
('PROD-005', 30.0, 'stable');

SELECT 'OMS data inserted' as status;

-- =====================================================
-- Inventory Database
-- =====================================================
USE inventory_db;

-- Clear existing data
DELETE FROM inventory_levels where id > 0;

-- Test Case 1: PROD-001 - Sufficient inventory (100 units available)
INSERT INTO inventory_levels (product_id, warehouse_id, current_qty, reserved_qty) VALUES
('PROD-001', 'WH-001', 100, 0);

-- Test Case 2: PROD-002 - Low inventory (5 units, needs reorder)
INSERT INTO inventory_levels (product_id, warehouse_id, current_qty, reserved_qty) VALUES
('PROD-002', 'WH-001', 5, 0);

-- Test Case 3: PROD-003 - Very low inventory (1 unit, high demand = expensive order)
INSERT INTO inventory_levels (product_id, warehouse_id, current_qty, reserved_qty) VALUES
('PROD-003', 'WH-001', 1, 0);

-- Test Case 4: PROD-004 - Low inventory with increasing trend
INSERT INTO inventory_levels (product_id, warehouse_id, current_qty, reserved_qty) VALUES
('PROD-004', 'WH-001', 10, 0);

-- Test Case 5: PROD-005 - Zero inventory
INSERT INTO inventory_levels (product_id, warehouse_id, current_qty, reserved_qty) VALUES
('PROD-005', 'WH-001', 0, 0);

SELECT 'Inventory data inserted' as status;

-- =====================================================
-- Supplier Database
-- =====================================================
USE supplier_db;

-- Clear existing data
DELETE FROM supplier_info where id > 0;

-- Test Case 1: PROD-001 - Standard supplier
INSERT INTO supplier_info (supplier_id, product_id, moq, lead_time_days, unit_price, is_active) VALUES
('SUP-001', 'PROD-001', 50, 7, 25.00, true);

-- Test Case 2: PROD-002 - Standard supplier
INSERT INTO supplier_info (supplier_id, product_id, moq, lead_time_days, unit_price, is_active) VALUES
('SUP-002', 'PROD-002', 100, 5, 15.50, true);

-- Test Case 3: PROD-003 - Expensive product for approval test
INSERT INTO supplier_info (supplier_id, product_id, moq, lead_time_days, unit_price, is_active) VALUES
('SUP-003', 'PROD-003', 50, 10, 120.00, true);

-- Test Case 4: PROD-004 - Medium price
INSERT INTO supplier_info (supplier_id, product_id, moq, lead_time_days, unit_price, is_active) VALUES
('SUP-004', 'PROD-004', 75, 7, 35.00, true);

-- Test Case 5: PROD-005 - Standard supplier
INSERT INTO supplier_info (supplier_id, product_id, moq, lead_time_days, unit_price, is_active) VALUES
('SUP-005', 'PROD-005', 60, 5, 22.50, true);

SELECT 'Supplier data inserted' as status;

-- =====================================================
-- UOM Database (shared across all products)
-- =====================================================
USE uom_db;

-- Clear existing data
DELETE FROM uom_conversion where id > 0;

-- Standard UOM conversions
INSERT INTO uom_conversion (from_uom, to_uom, conversion_factor, product_id) VALUES
('pieces', 'box', 0.1,'PROD-001'),    -- 10 pieces = 1 box
('box', 'pieces', 10.0,'PROD-001'),   -- 1 box = 10 pieces
('box', 'case', 0.2,'PROD-001'),      -- 5 boxes = 1 case
('case', 'box', 5.0,'PROD-001');      -- 1 case = 5 boxes

SELECT 'UOM data inserted' as status;

-- =====================================================
-- Verification Queries
-- =====================================================
SELECT '=== OMS Data ===' as section;
SELECT * FROM oms_db.oms_history ORDER BY product_id;

SELECT '=== Inventory Data ===' as section;
SELECT * FROM inventory_db.inventory_levels ORDER BY product_id;

SELECT '=== Supplier Data ===' as section;
SELECT * FROM supplier_db.supplier_info ORDER BY product_id;
																		
SELECT '=== UOM Data ===' as section;
SELECT * FROM uom_db.uom_conversion;

SELECT 'Test data setup completed!' as status;
