//! E-commerce order flow with Saga pattern
//! Demonstrates distributed transaction with compensation and timeout

use anyhow::Result;
use rust_logic_graph::saga::{SagaCoordinator, SagaStep, SagaStepStatus};
use std::time::Duration;

fn main() -> Result<()> {
    let mut saga = SagaCoordinator::new(Some(Duration::from_secs(10)));

    // Step 1: Reserve inventory
    saga.add_step(SagaStep {
        id: "reserve_inventory".to_string(),
        action: Box::new(|ctx| {
            println!("[Inventory] Reserving items...");
            ctx.data
                .insert("inventory_reserved".to_string(), serde_json::json!(true));
            Ok(())
        }),
        compensation: Some(Box::new(|ctx| {
            println!("[Inventory] Releasing reserved items...");
            ctx.data
                .insert("inventory_reserved".to_string(), serde_json::json!(false));
            Ok(())
        })),
        status: SagaStepStatus::Pending,
        timeout: Some(Duration::from_secs(3)),
    });

    // Step 2: Charge payment
    saga.add_step(SagaStep {
        id: "charge_payment".to_string(),
        action: Box::new(|ctx| {
            println!("[Payment] Charging customer...");
            // Simulate success
            ctx.data
                .insert("payment_charged".to_string(), serde_json::json!(true));
            Ok(())
        }),
        compensation: Some(Box::new(|ctx| {
            println!("[Payment] Refunding customer...");
            ctx.data
                .insert("payment_charged".to_string(), serde_json::json!(false));
            Ok(())
        })),
        status: SagaStepStatus::Pending,
        timeout: Some(Duration::from_secs(3)),
    });

    // Step 3: Create shipment (simulate failure)
    saga.add_step(SagaStep {
        id: "create_shipment".to_string(),
        action: Box::new(|ctx| {
            println!("[Shipping] Creating shipment...");
            Err(anyhow::anyhow!("Shipping service unavailable"))
        }),
        compensation: Some(Box::new(|ctx| {
            println!("[Shipping] Cancelling shipment...");
            Ok(())
        })),
        status: SagaStepStatus::Pending,
        timeout: Some(Duration::from_secs(3)),
    });

    // Step 4: Send confirmation (should not run)
    saga.add_step(SagaStep {
        id: "send_confirmation".to_string(),
        action: Box::new(|ctx| {
            println!("[Notification] Sending order confirmation...");
            Ok(())
        }),
        compensation: Some(Box::new(|ctx| {
            println!("[Notification] Cancelling confirmation...");
            Ok(())
        })),
        status: SagaStepStatus::Pending,
        timeout: Some(Duration::from_secs(2)),
    });

    let result = saga.execute();
    println!("\nSaga result: {:?}", result);
    println!("Saga state: {:?}", saga.state);
    println!("Saga context: {:#?}", saga.context.data);
    Ok(())
}
