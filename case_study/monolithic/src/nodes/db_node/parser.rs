use serde_json::{json, Value};
use sqlx::Row;

/// Parse MySQL query results into JSON
pub fn parse_mysql_row(node_id: &str, row: sqlx::mysql::MySqlRow) -> Result<Value, rust_logic_graph::rule::RuleError> {
    let result = match node_id {
        "oms_history" => json!({
            "product_id": row.try_get::<String, _>("product_id").unwrap_or_default(),
            "avg_daily_demand": row.try_get::<f64, _>("avg_daily_demand").unwrap_or(0.0),
            "trend": row.try_get::<String, _>("trend").unwrap_or_default(),
        }),
        "inventory_levels" => json!({
            "product_id": row.try_get::<String, _>("product_id").unwrap_or_default(),
            "available_qty": row.try_get::<f64, _>("available_qty").unwrap_or(0.0),
            "reserved_qty": row.try_get::<f64, _>("reserved_qty").unwrap_or(0.0),
            "warehouse_location": row.try_get::<String, _>("warehouse_location").unwrap_or_default(),
        }),
        "supplier_info" => json!({
            "product_id": row.try_get::<String, _>("product_id").unwrap_or_default(),
            "supplier_name": row.try_get::<String, _>("supplier_name").unwrap_or_default(),
            "unit_price": row.try_get::<f64, _>("unit_price").unwrap_or(0.0),
            "moq": row.try_get::<f64, _>("moq").unwrap_or(0.0),
            "lead_time": row.try_get::<i32, _>("lead_time").unwrap_or(0),
        }),
        "uom_conversion" => json!({
            "product_id": row.try_get::<String, _>("product_id").unwrap_or_default(),
            "from_uom": row.try_get::<String, _>("from_uom").unwrap_or_default(),
            "to_uom": row.try_get::<String, _>("to_uom").unwrap_or_default(),
            "conversion_factor": row.try_get::<f64, _>("conversion_factor").unwrap_or(1.0),
        }),
        _ => return Err(rust_logic_graph::rule::RuleError::Eval(format!("Unknown DB node: {}", node_id))),
    };
    Ok(result)
}

/// Parse Postgres query results into JSON
pub fn parse_postgres_row(node_id: &str, row: sqlx::postgres::PgRow) -> Result<Value, rust_logic_graph::rule::RuleError> {
    let result = match node_id {
        "oms_history" => {
            let product_id: String = row.get("product_id");
            let avg_daily_demand: f64 = row.get("avg_daily_demand");
            let trend: String = row.get("trend");
            tracing::debug!("🔍 OMS History: product_id={}, avg_daily_demand={}, trend={}", 
                product_id, avg_daily_demand, trend);
            json!({
                "product_id": product_id,
                "avg_daily_demand": avg_daily_demand,
                "trend": trend,
            })
        },
        "inventory_levels" => {
            let product_id: String = row.get("product_id");
            let available_qty: f64 = row.get("available_qty");
            let reserved_qty: f64 = row.get("reserved_qty");
            let warehouse_location: String = row.get("warehouse_location");
            let safety_stock: f64 = row.try_get("safety_stock").unwrap_or(0.0);
            let reorder_point: f64 = row.try_get("reorder_point").unwrap_or(0.0);
            tracing::debug!("🔍 Inventory: product_id={}, available={}, reserved={}, location={}, safety_stock={}, reorder_point={}", 
                product_id, available_qty, reserved_qty, warehouse_location, safety_stock, reorder_point);
            json!({
                "product_id": product_id,
                "available_qty": available_qty,
                "reserved_qty": reserved_qty,
                "warehouse_location": warehouse_location,
                "safety_stock": safety_stock,
                "reorder_point": reorder_point,
            })
        },
        "supplier_info" => {
            let product_id: String = row.get("product_id");
            let supplier_name: String = row.get("supplier_name");
            let unit_price: f64 = row.get("unit_price");
            let moq: f64 = row.get("moq");
            let lead_time: i32 = row.get("lead_time");
            tracing::debug!("🔍 Supplier: product_id={}, supplier={}, price={}, moq={}, lead_time={}", 
                product_id, supplier_name, unit_price, moq, lead_time);
            json!({
                "product_id": product_id,
                "supplier_name": supplier_name,
                "unit_price": unit_price,
                "moq": moq,
                "lead_time": lead_time,
            })
        },
        "uom_conversion" => {
            let product_id: String = row.get("product_id");
            let from_uom: String = row.get("from_uom");
            let to_uom: String = row.get("to_uom");
            let conversion_factor: f64 = row.get("conversion_factor");
            tracing::debug!("🔍 UOM: product_id={}, from={}, to={}, factor={}", 
                product_id, from_uom, to_uom, conversion_factor);
            json!({
                "product_id": product_id,
                "from_uom": from_uom,
                "to_uom": to_uom,
                "conversion_factor": conversion_factor,
            })
        },
        _ => return Err(rust_logic_graph::rule::RuleError::Eval(format!("Unknown DB node: {}", node_id))),
    };
    Ok(result)
}
