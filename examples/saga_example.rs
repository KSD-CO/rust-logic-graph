//! Example: Saga Pattern Implementation
//! Demonstrates transaction coordinator, compensation, state persistence, timeout/deadline

use anyhow::Result;
use rust_logic_graph::saga::{SagaContext, SagaCoordinator, SagaStep, SagaStepStatus};
use std::time::Duration;

fn main() -> Result<()> {
    let mut saga = SagaCoordinator::new(Some(Duration::from_secs(5)));

    // Step 1: Reserve inventory
    saga.add_step(SagaStep {
        id: "reserve_inventory".to_string(),
        action: Box::new(|ctx| {
            println!("Reserving inventory...");
            ctx.data
                .insert("inventory_reserved".to_string(), serde_json::json!(true));
            Ok(())
        }),
        compensation: Some(Box::new(|ctx| {
            println!("Releasing inventory...");
            ctx.data
                .insert("inventory_reserved".to_string(), serde_json::json!(false));
            Ok(())
        })),
        status: SagaStepStatus::Pending,
        timeout: Some(Duration::from_secs(2)),
    });

    // Step 2: Charge payment (simulate failure)
    saga.add_step(SagaStep {
        id: "charge_payment".to_string(),
        action: Box::new(|ctx| {
            println!("Charging payment...");
            Err(anyhow::anyhow!("Payment failed"))
        }),
        compensation: Some(Box::new(|ctx| {
            println!("Refunding payment...");
            Ok(())
        })),
        status: SagaStepStatus::Pending,
        timeout: Some(Duration::from_secs(2)),
    });

    // Step 3: Ship order (should not run)
    saga.add_step(SagaStep {
        id: "ship_order".to_string(),
        action: Box::new(|ctx| {
            println!("Shipping order...");
            Ok(())
        }),
        compensation: Some(Box::new(|ctx| {
            println!("Cancel shipment...");
            Ok(())
        })),
        status: SagaStepStatus::Pending,
        timeout: Some(Duration::from_secs(2)),
    });

    let result = saga.execute();
    println!("Saga result: {:?}", result);
    println!("Saga state: {:?}", saga.state);
    Ok(())
}
