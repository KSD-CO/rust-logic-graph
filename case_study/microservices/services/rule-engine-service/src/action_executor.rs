// Action Executor Module
// Custom action handlers callable from GRL rules using register_action_handler

use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use tracing::info;

/// Register all custom action handlers with the rule engine
/// This allows GRL rules to call Rust functions directly
pub fn register_action_handlers(engine: &mut rust_rule_engine::engine::RustRuleEngine) {
    info!("🎯 Registering custom action handlers for GRL rules");
    
    // 1. CreatePurchaseOrder - Creates PO and stores in facts
    engine.register_action_handler("CreatePurchaseOrder", |params, facts| {
        let product_id = get_param_string(params, "0", facts).unwrap_or_else(|| "unknown".to_string());
        let supplier_id = get_param_string(params, "1", facts).unwrap_or_else(|| "unknown".to_string());
        let quantity = get_param_number(params, "2", facts).unwrap_or(0.0);
        let unit_price = get_param_number(params, "3", facts).unwrap_or(0.0);
        let total_amount = get_param_number(params, "4", facts).unwrap_or(quantity * unit_price);
        
        info!("📝 CREATE PURCHASE ORDER:");
        info!("   ├── Product ID: {}", product_id);
        info!("   ├── Supplier ID: {}", supplier_id);
        info!("   ├── Quantity: {}", quantity);
        info!("   ├── Unit Price: ${:.2}", unit_price);
        info!("   ├── Total Amount: ${:.2}", total_amount);
        info!("   └── Status: ✅ PO created successfully");
        
        // Store PO details in facts for later use
        let mut po_data = HashMap::new();
        po_data.insert("product_id".to_string(), Value::String(product_id));
        po_data.insert("supplier_id".to_string(), Value::String(supplier_id));
        po_data.insert("quantity".to_string(), Value::Number(quantity));
        po_data.insert("unit_price".to_string(), Value::Number(unit_price));
        po_data.insert("total_amount".to_string(), Value::Number(total_amount));
        po_data.insert("status".to_string(), Value::String("created".to_string()));
        po_data.insert("created_at".to_string(), Value::String(chrono::Utc::now().to_string()));
        
        facts.add_value("PurchaseOrder", Value::Object(po_data)).unwrap();
        
        Ok(())
    });
    
    // 2. SendPurchaseOrder - Sends PO to supplier
    engine.register_action_handler("SendPurchaseOrder", |params, facts| {
        let po_id = get_param_string(params, "0", facts).unwrap_or_else(|| "unknown".to_string());
        let supplier_id = get_param_string(params, "1", facts).unwrap_or_else(|| "unknown".to_string());
        let send_method = get_param_string(params, "2", facts).unwrap_or_else(|| "email".to_string());
        
        info!("📤 SEND PURCHASE ORDER:");
        info!("   ├── PO ID: {}", po_id);
        info!("   ├── Supplier ID: {}", supplier_id);
        info!("   ├── Method: {}", send_method);
        
        // Business logic based on send method
        match send_method.to_lowercase().as_str() {
            "email" => {
                info!("   ├── 📧 Sending via Email");
                info!("   ├── To: supplier-{}@example.com", supplier_id);
            },
            "fax" => {
                info!("   ├── 📠 Sending via Fax");
            },
            "edi" => {
                info!("   ├── 🔄 Sending via EDI");
            },
            _ => {
                info!("   ├── 📮 Sending via {}", send_method);
            }
        }
        
        info!("   └── Status: ✅ PO sent successfully");
        
        // Update PO status
        if let Some(Value::Object(po_obj)) = facts.get("PurchaseOrder") {
            let mut updated_po = po_obj.clone();
            updated_po.insert("status".to_string(), Value::String("sent".to_string()));
            updated_po.insert("sent_at".to_string(), Value::String(chrono::Utc::now().to_string()));
            updated_po.insert("send_method".to_string(), Value::String(send_method));
            facts.add_value("PurchaseOrder", Value::Object(updated_po)).unwrap();
        }
        
        Ok(())
    });
    
    // 3. SendAlert - Multi-level alert system
    engine.register_action_handler("SendAlert", |params, facts| {
        let level = get_param_string(params, "0", facts).unwrap_or_else(|| "INFO".to_string());
        let message = get_param_string(params, "1", facts).unwrap_or_else(|| "Alert".to_string());
        
        let (emoji, priority) = match level.to_uppercase().as_str() {
            "CRITICAL" => ("🚨", "URGENT"),
            "HIGH" => ("⚠️", "HIGH"),
            "MEDIUM" => ("🔔", "NORMAL"),
            "LOW" => ("ℹ️", "LOW"),
            _ => ("ℹ️", "INFO"),
        };
        
        info!("{} ALERT [{}]:", emoji, level.to_uppercase());
        info!("   ├── Priority: {}", priority);
        info!("   ├── Message: {}", message);
        info!("   ├── Timestamp: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
        
        // Add context from facts
        if let Some(product_id) = facts.get("product_id") {
            info!("   ├── Product: {}", product_id.to_string());
        }
        
        if level.to_uppercase() == "CRITICAL" {
            info!("   ├── 🚨 IMMEDIATE ACTION REQUIRED");
            info!("   └── 📞 Notifying management");
        } else {
            info!("   └── 📋 Alert logged");
        }
        
        Ok(())
    });
    
    // 4. LogToDatabase - Database logging with context
    engine.register_action_handler("LogToDatabase", |params, facts| {
        let table = get_param_string(params, "0", facts).unwrap_or_else(|| "events".to_string());
        let event = get_param_string(params, "1", facts).unwrap_or_else(|| "unknown".to_string());
        let category = get_param_string(params, "2", facts).unwrap_or_else(|| "general".to_string());
        
        info!("🗄️ DATABASE LOG:");
        info!("   ├── Table: {}", table);
        info!("   ├── Event: {}", event);
        info!("   ├── Category: {}", category);
        info!("   ├── Timestamp: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
        
        // Add context from facts
        if let Some(product_id) = facts.get("product_id") {
            info!("   ├── Product: {}", product_id.to_string());
        }
        
        if let Some(Value::Object(po_obj)) = facts.get("PurchaseOrder") {
            if let Some(total) = po_obj.get("total_amount") {
                info!("   ├── PO Total: ${}", total.to_string());
            }
        }
        
        info!("   └── Status: ✅ Logged to database");
        
        Ok(())
    });
    
    // 5. ReserveInventory - Reserve stock for order
    engine.register_action_handler("ReserveInventory", |params, facts| {
        let product_id = get_param_string(params, "0", facts).unwrap_or_else(|| "unknown".to_string());
        let quantity = get_param_number(params, "1", facts).unwrap_or(0.0);
        
        info!("📦 RESERVE INVENTORY:");
        info!("   ├── Product ID: {}", product_id);
        info!("   ├── Quantity: {}", quantity);
        info!("   └── Status: ✅ Reserved successfully");
        
        Ok(())
    });
    
    info!("✅ Registered 5 custom action handlers");
}

// Helper functions to extract parameters from action calls

fn get_param_string(params: &HashMap<String, Value>, index: &str, facts: &Facts) -> Option<String> {
    if let Some(arg) = params.get(index) {
        match arg {
            Value::String(s) => {
                // Try to resolve from facts if it looks like a reference
                if let Some(resolved) = facts.get_nested(s) {
                    Some(resolved.to_string())
                } else {
                    Some(s.clone())
                }
            }
            _ => Some(arg.to_string()),
        }
    } else {
        None
    }
}

fn get_param_number(params: &HashMap<String, Value>, index: &str, facts: &Facts) -> Option<f64> {
    if let Some(arg) = params.get(index) {
        match arg {
            Value::Number(n) => Some(*n),
            Value::Integer(i) => Some(*i as f64),
            Value::String(s) => {
                // Try to resolve from facts first
                if let Some(fact_value) = facts.get_nested(s) {
                    match fact_value {
                        Value::Number(n) => Some(n),
                        Value::Integer(i) => Some(i as f64),
                        _ => s.parse::<f64>().ok(),
                    }
                } else {
                    s.parse::<f64>().ok()
                }
            }
            _ => None,
        }
    } else {
        None
    }
}
