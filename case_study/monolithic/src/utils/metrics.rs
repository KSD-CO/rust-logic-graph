use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub oms_fetch_time_ms: u128,
    pub inventory_fetch_time_ms: u128,
    pub supplier_fetch_time_ms: u128,
    pub uom_fetch_time_ms: u128,
    pub rule_evaluation_time_ms: u128,
    pub total_time_ms: u128,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            oms_fetch_time_ms: 0,
            inventory_fetch_time_ms: 0,
            supplier_fetch_time_ms: 0,
            uom_fetch_time_ms: 0,
            rule_evaluation_time_ms: 0,
            total_time_ms: 0,
        }
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn print_summary(&self) {
        println!("\n📊 Performance Metrics:");
        println!("   OMS Fetch:       {:>6} ms", self.oms_fetch_time_ms);
        println!("   Inventory Fetch: {:>6} ms", self.inventory_fetch_time_ms);
        println!("   Supplier Fetch:  {:>6} ms", self.supplier_fetch_time_ms);
        println!("   UOM Fetch:       {:>6} ms", self.uom_fetch_time_ms);
        println!("   Rule Evaluation: {:>6} ms", self.rule_evaluation_time_ms);
        println!("   ─────────────────────────");
        println!("   Total Time:      {:>6} ms", self.total_time_ms);
    }
}

pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
