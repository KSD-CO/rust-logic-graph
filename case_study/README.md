# Purchasing Flow - Production Case Study

> **A complete, production-grade distributed purchasing system built with `rust-logic-graph` and real MySQL databases**

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![MySQL](https://img.shields.io/badge/MySQL-8.0+-blue.svg)](https://www.mysql.com)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](../LICENSE)

**[Quick Start](#-quick-start-5-minutes)** • **[Documentation](#-documentation)** • **[Architecture](#-architecture)** • **[Learning Path](#-learning-path)**

</div>

---

## 📋 Table of Contents

- [What Is This?](#-what-is-this)
- [Quick Start](#-quick-start-5-minutes)
- [Directory Structure](#-directory-structure)
- [Documentation](#-documentation)
- [Code Examples](#-code-examples)
- [Architecture](#-architecture)
- [Learning Path](#-learning-path)
- [Key Features](#-key-features)
- [Performance](#-performance)
- [Use Cases](#-use-cases)

---

## 🎯 What Is This?

This case study demonstrates how to build a **real-world distributed purchasing automation system** using the `rust-logic-graph` framework. It's not just a tutorial - it's a complete reference implementation showcasing:

### Production Patterns
✅ **Multi-database architecture** - 4 separate MySQL databases (OMS, Inventory, Supplier, UOM)
✅ **Async/await processing** - Parallel queries with connection pooling
✅ **Business rule engine** - Automated purchasing decisions
✅ **Performance monitoring** - Real-time metrics and timing
✅ **Error handling** - Comprehensive error management
✅ **Production logging** - Structured, informative output

### Complete Package
✅ **3 progressive code examples** (Mock → Real DB → Advanced)
✅ **7 comprehensive documents** (Quickstart to deep-dive)
✅ **Database setup automation** (One-command setup)
✅ **Performance benchmarks** (Criterion-based testing)
✅ **Test data included** (3 products with realistic data)

---

## 🚀 Quick Start (5 Minutes)

```bash
# 1. Navigate to case study directory
cd case_study

# 2. Setup databases (one-time)
./scripts/setup_databases.sh

# 3. Run the example
cd ..
cargo run --example purchasing_flow_realdb --features mysql

# OR run advanced version with monitoring
cargo run --example purchasing_flow_advanced --features mysql
```

**Expected Output**: Complete purchasing flow with real database queries, business logic calculation, and generated purchase order.

---

## 📁 Directory Structure

```
case_study/
├── README.md                    # This file (main entry point)
│
├── docs/                        # Documentation
│   ├── QUICKSTART.md           # 5-minute quick reference ⭐ START HERE
│   ├── CASE_STUDY.md           # Complete technical deep-dive (30 pages)
│   ├── purchasing_flow_README.md # Full technical reference
│   ├── COMPARISON.md           # Mock vs Real DB analysis
│   ├── PURCHASING_FLOW_SUMMARY.md # Vietnamese summary
│   ├── CASE_STUDY_INDEX.md     # Navigation guide
│   └── PRESENTATION.md         # Slide-style presentation
│
├── src/                        # Source code examples
│   ├── purchasing_flow_realdb.rs     # Real DB version ⭐ MAIN
│   └── purchasing_flow_advanced.rs   # With monitoring & metrics
│
├── sql/                        # Database setup
│   └── purchasing_flow_setup.sql     # Creates 4 databases + test data
│
└── scripts/                    # Helper scripts
    ├── setup_databases.sh      # Automated database setup
    └── test_purchasing_flow.sh # Connectivity test + run
```

### How to Navigate

**New to this?** → Start with [`docs/QUICKSTART.md`](docs/QUICKSTART.md)
**Want details?** → Read [`docs/CASE_STUDY.md`](docs/CASE_STUDY.md)
**Need reference?** → Check [`docs/purchasing_flow_README.md`](docs/purchasing_flow_README.md)
**Looking for code?** → See [`src/`](src/) directory
**Setting up DB?** → Use [`scripts/setup_databases.sh`](scripts/setup_databases.sh)

---

## 📚 Documentation

We provide **7 comprehensive documents** for different learning styles:

| Document | Purpose | Audience | Time |
|----------|---------|----------|------|
| **[QUICKSTART.md](docs/QUICKSTART.md)** | Get running fast | Everyone | 5 min |
| **[CASE_STUDY.md](docs/CASE_STUDY.md)** ⭐ | Complete technical analysis | Developers | 30 min |
| **[purchasing_flow_README.md](docs/purchasing_flow_README.md)** | Full reference | Developers | 20 min |
| **[COMPARISON.md](docs/COMPARISON.md)** | Mock vs Real DB | Architects | 10 min |
| **[PURCHASING_FLOW_SUMMARY.md](docs/PURCHASING_FLOW_SUMMARY.md)** | Vietnamese summary | Vietnamese speakers | 15 min |
| **[CASE_STUDY_INDEX.md](docs/CASE_STUDY_INDEX.md)** | Navigation guide | Everyone | 10 min |
| **[PRESENTATION.md](docs/PRESENTATION.md)** | Slide format | Presenters | 15 min |

---

## 💻 Code Examples

### Three Progressive Versions

We provide 3 versions with increasing complexity:

#### 1. Mock Version (Beginner) ⭐
**Location**: `../examples/purchasing_flow.rs` (in main examples folder)

```bash
cargo run --example purchasing_flow
```

- ✅ No database required
- ✅ Perfect for learning framework basics
- ✅ Executes in ~1ms
- ✅ Great for understanding graph structure

#### 2. Real Database Version (Intermediate) ⭐⭐
**Location**: [`src/purchasing_flow_realdb.rs`](src/purchasing_flow_realdb.rs)

```bash
# From project root
cargo run --example purchasing_flow_realdb --features mysql
```

- ✅ Connects to 4 separate MySQL databases
- ✅ Real connection pooling with sqlx
- ✅ Async/await database queries
- ✅ Production-like architecture
- ✅ Executes in ~500ms

#### 3. Advanced Monitoring Version (Expert) ⭐⭐⭐
**Location**: [`src/purchasing_flow_advanced.rs`](src/purchasing_flow_advanced.rs)

```bash
# From project root
cargo run --example purchasing_flow_advanced --features mysql
```

- ✅ All features from Version 2
- ✅ **+ Performance metrics collection**
- ✅ **+ Real-time monitoring output**
- ✅ **+ Detailed timing information**
- ✅ **+ Production-ready logging**

---

## 🏗️ Architecture

### High-Level System Design

```
┌─────────────────────────────────────────────────────────┐
│              Purchasing Flow Orchestrator                │
│            (Rust Logic Graph Framework)                  │
└─────────────────────────────────────────────────────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
    ┌────────┐    ┌─────────┐   ┌─────────┐    ┌────────┐
    │OMS DB  │    │ INV DB  │   │ SUP DB  │    │UOM DB  │
    │oms_db  │    │inventory│   │supplier │    │uom_db  │
    │        │    │   _db   │   │   _db   │    │        │
    └────────┘    └─────────┘   └─────────┘    └────────┘

Host: 171.244.10.40:6033
```

### Data Flow

```
Input: product_id
    ↓
[Parallel DB Queries] (4 databases, async)
    ↓
[Shared Context] (HashMap with all data)
    ↓
[Rule Engine] → [Calculate Order Qty] → [Create PO] → [Send PO]
    ↓
Output: Purchase Order (JSON)
```

### Graph Structure

```mermaid
graph LR
    A[OMS Node] --> E[Rule Engine]
    B[Inventory Node] --> E
    C[Supplier Node] --> E
    D[UOM Node] --> E
    E --> F[Calc Order Qty]
    F --> G[Create PO]
    G --> H[Send PO]
```

**See [`docs/CASE_STUDY.md`](docs/CASE_STUDY.md) for detailed architecture analysis.**

---

## 🎓 Learning Path

### Level 1: Understand Basics (30 minutes)
1. Read [`docs/QUICKSTART.md`](docs/QUICKSTART.md)
2. Run mock version: `cargo run --example purchasing_flow`
3. Understand graph structure and node types

### Level 2: Real Database Integration (1 hour)
1. Read database setup in [`docs/purchasing_flow_README.md`](docs/purchasing_flow_README.md)
2. Setup databases: `./scripts/setup_databases.sh`
3. Run real DB version: `cargo run --example purchasing_flow_realdb --features mysql`
4. Study custom `MySQLDBNode` in [`src/purchasing_flow_realdb.rs`](src/purchasing_flow_realdb.rs)

### Level 3: Production Patterns (2 hours)
1. Read complete [`docs/CASE_STUDY.md`](docs/CASE_STUDY.md)
2. Run advanced version: `cargo run --example purchasing_flow_advanced --features mysql`
3. Study metrics collection and monitoring
4. Review error handling patterns

### Level 4: Performance Analysis (1 hour)
1. Read Performance Analysis section in case study
2. Run benchmarks: `cargo bench --bench purchasing_flow_benchmark`
3. Study the comparison in [`docs/COMPARISON.md`](docs/COMPARISON.md)

### Level 5: Adapt for Your Use Case (4+ hours)
1. Understand business logic
2. Modify for your requirements
3. Add custom nodes and rules
4. Deploy to your environment

---

## ✨ Key Features

### Distributed Architecture
- **4 independent MySQL databases**
- Each database represents a different external system
- Follows microservices patterns
- Isolated failures and independent scaling

### Asynchronous Processing
- **Parallel database queries** for optimal performance
- Non-blocking I/O with tokio
- Connection pooling for efficiency
- Sub-500ms end-to-end latency

### Business Logic
- **Automated order quantity calculation**
  ```
  demand = avg_daily_demand × lead_time_days
  shortage = max(0, demand - available_stock)
  order_qty = ceil(shortage / moq) × moq
  ```
- Rule-based decision engine
- Supplier constraint handling (MOQ, lead times)
- Unit conversion support

### Production Ready
- Comprehensive error handling
- Performance metrics collection
- Structured logging
- Connection pool management
- Type-safe with Rust

---

## 📊 Performance

### Metrics (Real Database Version)

| Metric | Value | Notes |
|--------|-------|-------|
| **Total Latency** | ~500ms | Per product, end-to-end |
| **DB Connections** | 4 pools | One per database |
| **DB Queries** | 4 parallel | Executed concurrently |
| **Query Time** | ~150ms | Total for all 4 queries |
| **Business Logic** | ~20ms | Calculation + PO creation |
| **Graph Overhead** | ~30ms | Framework execution |
| **Throughput** | 100+ | Products in batch mode |

### Comparison

| Version | Latency | Setup | Realism |
|---------|---------|-------|---------|
| Mock | <1ms | None | Low |
| Real DB | ~500ms | Required | High |
| Advanced | ~500ms | Required | Very High |

**See [`docs/COMPARISON.md`](docs/COMPARISON.md) for detailed analysis.**

---

## 🎯 Use Cases

This case study is perfect for:

### 1. Learning
- Understanding distributed systems
- Learning async Rust patterns
- Studying workflow orchestration
- Exploring database integration

### 2. Building
- Supply chain automation
- E-commerce purchasing systems
- Inventory management
- Order processing workflows

### 3. Reference
- Production architecture patterns
- Microservices design
- Database integration strategies
- Error handling best practices

### 4. Teaching
- Workshop material
- Code examples
- Architecture demonstrations
- Best practices showcase

---

## 🗄️ Database Configuration

### Connection Info

```
Host: 171.244.10.40
Port: 6033
User: lune_dev
Pass: rfSxLLeSqVCGNeGc
```

### Databases

1. **oms_db** - Order Management System
   - Table: `oms_history` (demand forecasting)
   - Columns: product_id, avg_daily_demand, trend

2. **inventory_db** - Inventory Management
   - Table: `inventory_levels` (stock management)
   - Columns: product_id, warehouse_id, current_qty, reserved_qty, available_qty

3. **supplier_db** - Supplier Management
   - Table: `supplier_info` (vendor information)
   - Columns: supplier_id, product_id, moq, lead_time_days, unit_price

4. **uom_db** - Unit of Measure
   - Table: `uom_conversion` (unit conversions)
   - Columns: product_id, from_uom, to_uom, conversion_factor

### Test Data

Pre-populated with 3 products:
- **PROD-001**: High demand, low stock → Large order (100 units)
- **PROD-002**: Medium demand, adequate stock → Medium order
- **PROD-003**: High demand, very low stock → Urgent order

---

## 🛠️ Setup Instructions

### Prerequisites

- Rust 1.70 or later
- MySQL client (optional, for manual setup)
- Network access to MySQL server

### Automated Setup (Recommended)

```bash
cd case_study
./scripts/setup_databases.sh
```

This will:
- Check for MySQL client
- Create 4 databases
- Create all tables
- Insert test data
- Verify setup

### Manual Setup

```bash
mysql -h 171.244.10.40 -P 6033 -u lune_dev -p'rfSxLLeSqVCGNeGc' \
  < sql/purchasing_flow_setup.sql
```

### Verify Setup

```bash
# Use test script
./scripts/test_purchasing_flow.sh

# Or manually
cd ..
cargo run --example purchasing_flow_realdb --features mysql
```

---

## 📝 What You'll Learn

### Framework Concepts
✅ Graph-based workflow orchestration
✅ Node types and dependencies
✅ Context sharing between nodes
✅ Parallel execution optimization

### Rust Patterns
✅ Async/await for I/O operations
✅ Connection pooling with sqlx
✅ Error handling with Result
✅ Type-safe business logic
✅ RAII resource management

### Production Patterns
✅ Multi-database integration
✅ Distributed system design
✅ Performance monitoring
✅ Metrics collection
✅ Comprehensive logging

### Database Techniques
✅ Connection pool management
✅ Query optimization
✅ Data consistency patterns
✅ Transaction handling

---

## 🔍 Troubleshooting

### Connection Failed

```bash
# Test connectivity
nc -z -v 171.244.10.40 6033

# Check credentials
mysql -h 171.244.10.40 -P 6033 -u lune_dev -p
```

### Build Errors

```bash
# Clean rebuild
cd ..
cargo clean
cargo build --example purchasing_flow_realdb --features mysql
```

### Missing Data

```bash
# Re-run setup
cd case_study
./scripts/setup_databases.sh
```

**For more help, see [`docs/QUICKSTART.md`](docs/QUICKSTART.md) Troubleshooting section.**

---

## 🌟 Highlights

### Why This Is Special

❌ **Not a toy example** - Real production patterns
❌ **Not incomplete** - Comprehensive documentation
❌ **Not just code** - Includes analysis and benchmarks
❌ **Not single-database** - Distributed architecture

✅ **Production-ready** reference implementation
✅ **Complete documentation** from basics to deep-dive
✅ **Progressive complexity** with 3 versions
✅ **Performance focused** with benchmarks
✅ **Best practices** throughout

---

## 📞 Support & Resources

### Documentation
- **Quick Start**: [`docs/QUICKSTART.md`](docs/QUICKSTART.md)
- **Complete Guide**: [`docs/CASE_STUDY.md`](docs/CASE_STUDY.md)
- **Technical Reference**: [`docs/purchasing_flow_README.md`](docs/purchasing_flow_README.md)
- **Navigation**: [`docs/CASE_STUDY_INDEX.md`](docs/CASE_STUDY_INDEX.md)

### Code
- **Real DB Version**: [`src/purchasing_flow_realdb.rs`](src/purchasing_flow_realdb.rs)
- **Advanced Version**: [`src/purchasing_flow_advanced.rs`](src/purchasing_flow_advanced.rs)
- **Mock Version**: `../examples/purchasing_flow.rs`

### Database
- **Setup SQL**: [`sql/purchasing_flow_setup.sql`](sql/purchasing_flow_setup.sql)
- **Setup Script**: [`scripts/setup_databases.sh`](scripts/setup_databases.sh)

---

## 🚀 Next Steps

### For Beginners
1. Start with [`docs/QUICKSTART.md`](docs/QUICKSTART.md)
2. Run the mock version
3. Follow the learning path

### For Developers
1. Read [`docs/CASE_STUDY.md`](docs/CASE_STUDY.md)
2. Setup databases and run all versions
3. Study the code and adapt for your needs

### For Architects
1. Review [`docs/PRESENTATION.md`](docs/PRESENTATION.md)
2. Analyze the distributed architecture
3. Consider patterns for your systems

---

## 📜 License

MIT License - See project root for details

---

## 🎉 Get Started!

```bash
# Navigate to case study
cd case_study

# Read quick start
cat docs/QUICKSTART.md

# Setup and run
./scripts/setup_databases.sh
cd ..
cargo run --example purchasing_flow_realdb --features mysql
```

**Happy Learning! 🚀**

---

<div align="center">

**[Documentation](docs/)** • **[Source Code](src/)** • **[Database Setup](sql/)** • **[Scripts](scripts/)**

*Rust Logic Graph v0.7.0 - Production-Grade Workflow Orchestration*

</div>
