# 🚀 GRL (Grule Rule Language) Integration

Rust Logic Graph now includes **rust-rule-engine** for advanced business rule management using GRL (Grule Rule Language) syntax!

## 🎯 Why GRL?

- **📖 Readable Syntax** - Business-friendly rule definitions
- **🔥 High Performance** - RETE algorithm for efficient rule matching  
- **💪 Powerful Features** - Salience, activation groups, method calls
- **🎨 Flexible** - Simple expressions to complex business logic

---

## 🚀 Quick Start with GRL

### Simple GRL Rule

```rust
use rust_logic_graph::{RuleEngine, GrlRule};
use std::collections::HashMap;
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let mut context = HashMap::new();
    context.insert("age".to_string(), json!(25));
    
    let grl = r#"
rule "AgeVerification" {
    when
        age >= 18
    then
        verified = true;
}
"#;
    
    let mut engine = RuleEngine::new();
    engine.add_grl_rule(grl)?;
    engine.evaluate(&context)?;
    
    Ok(())
}
```

### Business Rules Example

```rust
let discount_rules = r#"
rule "MemberDiscount" salience 10 {
    when
        is_member == true && cart_total >= 100.0
    then
        discount = 0.15;
}

rule "RegularDiscount" salience 5 {
    when
        cart_total >= 100.0 && discount == 0.0
    then
        discount = 0.10;
}
"#;

let mut engine = RuleEngine::new();
engine.add_grl_rule(discount_rules)?;
```

---

## 📚 Examples

### 1. Simple GRL Rules

```bash
cargo run --example grl_rules
```

Demonstrates:
- Age verification rules
- E-commerce discount logic
- Temperature alerts
- Loan approval criteria

### 2. GRL + Graph Integration

```bash
cargo run --example grl_graph_flow
```

Shows how to integrate GRL rules into graph workflows:
- Input validation
- Risk assessment
- Fraud detection
- Approval decisions

---

## 🎨 GRL Helper Functions

### from_simple()

Convert simple conditions to GRL:

```rust
let rule = GrlRule::from_simple(
    "temperature_alert",
    "temperature > 30.0",
    "alert = true"
);
```

Generates:
```grl
rule "temperature_alert" {
    when
        temperature > 30.0
    then
        alert = true;
}
```

---

## 🔥 Advanced Features

### Salience (Priority)

```grl
rule "HighPriority" salience 100 {
    when
        amount > 10000
    then
        requires_approval = true;
}

rule "NormalPriority" salience 50 {
    when
        amount > 1000
    then
        notify_manager = true;
}
```

### Complex Conditions

```grl
rule "LoanApproval" {
    when
        credit_score >= 700 &&
        income >= 50000 &&
        debt_ratio < 0.4
    then
        approved = true;
        interest_rate = 3.5;
}
```

### Multiple Actions

```grl
rule "PremiumUpgrade" {
    when
        purchases > 10 && total_spent > 1000.0
    then
        tier = "premium";
        discount = 0.20;
        free_shipping = true;
}
```

---

## 🏗️ Integration with Logic Graph

### Use GRL in RuleNodes

```rust
let mut executor = Executor::new();

// Simple rule for node
executor.register_node(Box::new(RuleNode::new(
    "validate",
    "amount > 0 && amount <= 1000000"
)));
```

### Custom GRL Engine per Node

```rust
// Create advanced rule engine for specific node
let mut grl_engine = RuleEngine::new();
grl_engine.add_grl_rule(complex_rules)?;

// Use in custom node implementation
```

---

## 📊 Performance

rust-rule-engine provides:
- **RETE Algorithm**: Efficient pattern matching
- **97% Drools Compatibility**: Easy migration
- **2-24x Faster**: Than alternatives at scale (50+ rules)

---

## 📖 GRL Syntax Reference

### Basic Structure

```grl
rule "RuleName" salience PRIORITY {
    when
        CONDITION
    then
        ACTION;
}
```

### Operators

- **Comparison**: `>`, `<`, `>=`, `<=`, `==`, `!=`
- **Logical**: `&&`, `||`, `!`
- **Arithmetic**: `+`, `-`, `*`, `/`, `%`

### Data Types

- **Boolean**: `true`, `false`
- **Number**: `42`, `3.14`, `-10`
- **String**: `"hello"`, `"world"`

---

## 🔗 Resources

- [rust-rule-engine on crates.io](https://crates.io/crates/rust-rule-engine)
- [GRL Syntax Documentation](https://github.com/hyperjumptech/grule-rule-engine/blob/master/docs/en/GRL_en.md)
- [rust-rule-engine GitHub](https://github.com/KSD-CO/rust-rule-engine)

---

## ✅ Testing

All GRL features are tested:

```bash
cargo test
```

Tests include:
- ✓ RuleEngine creation
- ✓ GRL rule parsing
- ✓ Simple rule helpers
- ✓ Context evaluation
- ✓ Integration with graph

---

## 🎓 Learn More

Check out the examples:
- `examples/grl_rules.rs` - Standalone GRL examples
- `examples/grl_graph_flow.rs` - Integration with graphs
- `EXTENDING.md` - How to create custom nodes with GRL

---

**Ready to build powerful rule-based systems!** 🚀
