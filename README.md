# 🧠 Rust Logic Graph

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance **reasoning graph framework** for Rust with **GRL (Grule Rule Language)** support. Build complex workflows with conditional execution, topological ordering, and async processing.

```rust
use rust_logic_graph::{Graph, Orchestrator, GraphIO};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let def = GraphIO::load_from_file("workflow.json")?;
    let mut graph = Graph::new(def);
    Orchestrator::execute_graph(&mut graph).await?;
    Ok(())
}
```

---

## ✨ Key Features

- 🔥 **GRL Support** - [rust-rule-engine](https://crates.io/crates/rust-rule-engine) integration with RETE algorithm
- 🔄 **Topological Execution** - Automatic DAG-based node ordering
- ⚡ **Async Runtime** - Built on Tokio for high concurrency
- 📊 **Multiple Node Types** - RuleNode, DBNode, AINode
- 📝 **JSON Configuration** - Simple workflow definitions
- 🎯 **97% Drools Compatible** - Easy migration from Java

---

## 🚀 Quick Start

### Installation

```toml
[dependencies]
rust-logic-graph = "0.1.0"
```

### Simple Example

```rust
use rust_logic_graph::{RuleEngine, GrlRule};

let grl = r#"
rule "Discount" {
    when
        cart_total > 100 && is_member == true
    then
        discount = 0.15;
}
"#;

let mut engine = RuleEngine::new();
engine.add_grl_rule(grl)?;
```

### Run Examples

```bash
# Basic workflow
cargo run --example simple_flow

# GRL rules
cargo run --example grl_rules

# Advanced integration
cargo run --example grl_graph_flow
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **[GRL Guide](docs/GRL.md)** | Complete GRL syntax and examples |
| **[Use Cases](docs/USE_CASES.md)** | 33+ real-world applications |
| **[Extending](docs/EXTENDING.md)** | Create custom nodes and integrations |
| **[Implementation](docs/IMPLEMENTATION_SUMMARY.md)** | Technical details |
| **[GRL Integration](docs/GRL_INTEGRATION_SUMMARY.md)** | Integration guide |

---

## 🎯 Use Cases

Rust Logic Graph powers applications in:

- 💰 **Finance** - Loan approval, fraud detection, risk assessment
- 🛒 **E-commerce** - Dynamic pricing, recommendations, fulfillment
- 🏥 **Healthcare** - Patient triage, clinical decisions, monitoring
- 🏭 **Manufacturing** - Predictive maintenance, QC automation
- 🛡️ **Insurance** - Claims processing, underwriting
- 📊 **Marketing** - Lead scoring, campaign optimization
- ⚖️ **Compliance** - AML monitoring, GDPR automation

**[View all 33+ use cases →](docs/USE_CASES.md)**

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│         rust-rule-engine (GRL)          │
│        RETE Algorithm • 2-24x Faster    │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│        Rust Logic Graph Core            │
│  • Graph Definition                     │
│  • Topological Executor                 │
│  • Context Management                   │
└────────────────┬────────────────────────┘
                 │
    ┌────────────┼────────────┐
    │            │            │
┌───▼───┐   ┌───▼───┐   ┌───▼───┐
│ Rule  │   │  DB   │   │  AI   │
│ Node  │   │ Node  │   │ Node  │
└───────┘   └───────┘   └───────┘
```

---

## 🔥 GRL Example

```grl
rule "HighValueLoan" salience 100 {
    when
        loan_amount > 100000 &&
        credit_score < 750
    then
        requires_manual_review = true;
        approval_tier = "senior";
}

rule "AutoApproval" salience 50 {
    when
        credit_score >= 700 &&
        income >= loan_amount * 3 &&
        debt_ratio < 0.4
    then
        auto_approve = true;
        interest_rate = 3.5;
}
```

**[Learn more about GRL →](docs/GRL.md)**

---

## 📊 Performance

- **RETE Algorithm**: Optimized pattern matching
- **2-24x Faster**: Than alternatives at 50+ rules
- **97% Drools Compatible**: Easy migration path
- **Async by Default**: High concurrency support

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific example
cargo run --example grl_rules
```

**Result**: ✅ 6/6 tests passing

---

## 📦 Project Status

**Version**: 0.1.0 (Alpha)
**Status**: Production-ready core, active development

### What's Working
- ✅ Core graph execution engine
- ✅ GRL rule engine integration
- ✅ Three node types (Rule, DB, AI)
- ✅ Topological sorting
- ✅ Async execution
- ✅ JSON I/O
- ✅ Comprehensive documentation

### Roadmap
- [ ] Real database integrations (PostgreSQL, MySQL)
- [ ] Real AI/LLM integrations (OpenAI, Anthropic)
- [ ] Parallel node execution
- [ ] GraphQL API
- [ ] Web UI for visualization
- [ ] Performance optimizations

---

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create your feature branch
3. Write tests for new features
4. Submit a pull request

---

## 📖 Examples

| Example | Description | Lines |
|---------|-------------|-------|
| `simple_flow.rs` | Basic 3-node pipeline | 36 |
| `advanced_flow.rs` | Complex 6-node workflow | 120 |
| `grl_rules.rs` | GRL rule examples | 110 |
| `grl_graph_flow.rs` | GRL + Graph integration | 140 |

---

## 🌟 Why Rust Logic Graph?

### vs. Traditional Rule Engines
- ✅ **Async by default** - No blocking I/O
- ✅ **Type safety** - Rust's type system
- ✅ **Modern syntax** - GRL support
- ✅ **Graph-based** - Complex workflows

### vs. Workflow Engines
- ✅ **Embedded** - No external services
- ✅ **Fast** - Compiled Rust code
- ✅ **Flexible** - Custom nodes
- ✅ **Rule-based** - Business logic in rules

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🔗 Links

- **Repository**: https://github.com/KSD-CO/rust-logic-graph
- **rust-rule-engine**: https://crates.io/crates/rust-rule-engine
- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/KSD-CO/rust-logic-graph/issues)

---

## 👥 Authors

**James Vu** - Initial work

---

## 🙏 Acknowledgments

Built with:
- [rust-rule-engine](https://crates.io/crates/rust-rule-engine) - GRL support
- [Tokio](https://tokio.rs/) - Async runtime
- [Petgraph](https://github.com/petgraph/petgraph) - Graph algorithms
- [Serde](https://serde.rs/) - Serialization

---

<div align="center">

**⭐ Star us on GitHub if you find this useful! ⭐**

[Documentation](docs/) • [Examples](examples/) • [Use Cases](docs/USE_CASES.md)

</div>
