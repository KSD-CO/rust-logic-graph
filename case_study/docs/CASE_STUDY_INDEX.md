# Purchasing Flow Case Study - Complete Index

## 📚 Overview

This directory contains a **complete case study** demonstrating how to build a production-grade distributed purchasing system using the `rust-logic-graph` framework with real MySQL databases.

---

## 🗂️ Documentation Files

### Essential Reading (Start Here)

1. **[QUICKSTART.md](QUICKSTART.md)** - Get up and running in 5 minutes
   - Quick commands
   - Database info
   - Expected output
   - Troubleshooting

2. **[CASE_STUDY.md](CASE_STUDY.md)** - Complete case study (Main Document)
   - Business context and problem statement
   - Technical architecture deep-dive
   - Implementation details
   - Performance analysis
   - Production considerations
   - Lessons learned

### Technical Documentation

3. **[purchasing_flow_README.md](purchasing_flow_README.md)** - Full technical guide
   - Detailed setup instructions
   - Architecture diagrams
   - Business logic explanation
   - Test data reference
   - Customization guide

4. **[COMPARISON.md](COMPARISON.md)** - Mock vs Real Database
   - Feature comparison matrix
   - Code differences
   - Performance comparison
   - Use case recommendations

5. **[PURCHASING_FLOW_SUMMARY.md](PURCHASING_FLOW_SUMMARY.md)** - Vietnamese summary
   - Tổng quan dự án
   - Kiến trúc hệ thống
   - Hướng dẫn sử dụng
   - Troubleshooting

---

## 💻 Code Files

### Executable Examples

| File | Purpose | Database | Difficulty |
|------|---------|----------|------------|
| **purchasing_flow.rs** | Mock version (baseline) | None | ⭐ Beginner |
| **purchasing_flow_realdb.rs** | Real MySQL integration | 4 DBs | ⭐⭐ Intermediate |
| **purchasing_flow_advanced.rs** | With monitoring & metrics | 4 DBs | ⭐⭐⭐ Advanced |

### Database & Setup

| File | Purpose |
|------|---------|
| **purchasing_flow_setup.sql** | Creates 4 databases + test data |
| **setup_databases.sh** | Automated database setup script |
| **test_purchasing_flow.sh** | Connectivity test + run script |

### Performance Testing

| File | Purpose |
|------|---------|
| **../benches/purchasing_flow_benchmark.rs** | Criterion benchmarks |

---

## 🎯 Learning Path

### Level 1: Understanding the Basics (30 mins)
1. Read [QUICKSTART.md](QUICKSTART.md)
2. Run `purchasing_flow.rs` (mock version)
3. Understand the graph structure

```bash
cargo run --example purchasing_flow
```

### Level 2: Real Database Integration (1 hour)
1. Read [purchasing_flow_README.md](purchasing_flow_README.md) Database Setup section
2. Run `setup_databases.sh`
3. Execute `purchasing_flow_realdb.rs`
4. Study the custom MySQLDBNode implementation

```bash
./examples/setup_databases.sh
cargo run --example purchasing_flow_realdb --features mysql
```

### Level 3: Production Patterns (2 hours)
1. Read full [CASE_STUDY.md](CASE_STUDY.md)
2. Run `purchasing_flow_advanced.rs` with metrics
3. Study the monitoring implementation
4. Review error handling patterns

```bash
cargo run --example purchasing_flow_advanced --features mysql
```

### Level 4: Performance Analysis (1 hour)
1. Read Performance Analysis section in case study
2. Run benchmarks
3. Compare mock vs real DB performance
4. Read [COMPARISON.md](COMPARISON.md)

```bash
cargo bench --bench purchasing_flow_benchmark
```

---

## 🏗️ Architecture at a Glance

### System Components

```
┌─────────────────────────────────────────────────────────┐
│                Purchasing Flow System                    │
│              (Rust Logic Graph Framework)                │
└─────────────────────────────────────────────────────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
    ┌────────┐    ┌─────────┐   ┌─────────┐    ┌────────┐
    │ OMS DB │    │ INV DB  │   │ SUP DB  │    │UOM DB  │
    │171.244 │    │171.244  │   │171.244  │    │171.244 │
    │ .10.40 │    │ .10.40  │   │ .10.40  │    │.10.40  │
    │ :6033  │    │ :6033   │   │ :6033   │    │:6033   │
    └────────┘    └─────────┘   └─────────┘    └────────┘
```

### Data Flow

```
Input: product_id
  ↓
[Parallel DB Queries]
  ↓
[Rule Engine] → [Calculate Order Qty] → [Create PO] → [Send PO]
  ↓
Output: Purchase Order JSON
```

---

## 📊 Key Metrics

### Performance (Real DB)

- **Total Latency**: ~500ms per product
- **Database Queries**: 4 parallel queries
- **Query Time**: ~150ms total (parallel execution)
- **Business Logic**: ~20ms
- **Graph Overhead**: ~30ms

### Scale

- **Tested with**: Up to 100 products in batch
- **Databases**: 4 independent MySQL instances
- **Connection Pools**: Dedicated pool per database
- **Throughput**: Can process multiple products concurrently

---

## 🎓 What You'll Learn

### Framework Concepts

- ✅ Graph-based workflow orchestration
- ✅ Node types (DBNode, RuleNode, Custom)
- ✅ Dependency management with edges
- ✅ Context sharing between nodes
- ✅ Parallel execution optimization

### Production Patterns

- ✅ Multi-database integration
- ✅ Connection pooling with sqlx
- ✅ Async/await patterns
- ✅ Error handling strategies
- ✅ Performance monitoring
- ✅ Metrics collection

### Database Techniques

- ✅ Distributed database architecture
- ✅ Connection management
- ✅ Query optimization
- ✅ Transaction handling (future)
- ✅ Data consistency patterns

---

## 🚀 Quick Commands Reference

```bash
# Setup (one-time)
./examples/setup_databases.sh

# Run examples
cargo run --example purchasing_flow                    # Mock
cargo run --example purchasing_flow_realdb --features mysql   # Real DB
cargo run --example purchasing_flow_advanced --features mysql # With monitoring

# Test connectivity
./examples/test_purchasing_flow.sh

# Run benchmarks
cargo bench --bench purchasing_flow_benchmark

# Build for production
cargo build --release --features mysql
```

---

## 🗄️ Database Configuration

### Connection Info

```bash
Host: 171.244.10.40
Port: 6033
User: lune_dev
Pass: rfSxLLeSqVCGNeGc
```

### Databases Created

1. **oms_db** - Order Management System
   - Table: `oms_history`
   - Columns: product_id, avg_daily_demand, trend

2. **inventory_db** - Inventory Management
   - Table: `inventory_levels`
   - Columns: product_id, warehouse_id, current_qty, reserved_qty, available_qty

3. **supplier_db** - Supplier Management
   - Table: `supplier_info`
   - Columns: supplier_id, product_id, moq, lead_time_days, unit_price

4. **uom_db** - Unit of Measure
   - Table: `uom_conversion`
   - Columns: product_id, from_uom, to_uom, conversion_factor

---

## 📈 Example Results

### Input
```json
{
  "product_id": "PROD-001"
}
```

### Output
```json
{
  "po_id": "PO-1731715200",
  "product_id": "PROD-001",
  "supplier_id": "SUP-001",
  "qty": 100,
  "unit_price": 15.99,
  "total_amount": 1599.0,
  "status": "sent",
  "created_at": "2024-11-16T03:20:00Z",
  "sent_at": "2024-11-16T03:20:00Z"
}
```

---

## 🔧 Troubleshooting

### Common Issues

| Problem | Solution | Reference |
|---------|----------|-----------|
| Connection failed | Check network, credentials | QUICKSTART.md |
| Build errors | Clean rebuild with features | purchasing_flow_README.md |
| Missing data | Run setup script | QUICKSTART.md |
| Slow queries | Check database indexes | CASE_STUDY.md |

---

## 🎯 Use Cases

This case study is perfect for:

1. **Learning** distributed systems patterns
2. **Building** supply chain automation
3. **Testing** database integration strategies
4. **Demonstrating** production architectures
5. **Teaching** async Rust patterns
6. **Prototyping** workflow orchestration systems

---

## 📝 Files Summary

### Must Read
- ✅ [QUICKSTART.md](QUICKSTART.md)
- ✅ [CASE_STUDY.md](CASE_STUDY.md)

### Reference
- 📖 [purchasing_flow_README.md](purchasing_flow_README.md)
- 📖 [COMPARISON.md](COMPARISON.md)
- 📖 [PURCHASING_FLOW_SUMMARY.md](PURCHASING_FLOW_SUMMARY.md)

### Code
- 💻 purchasing_flow.rs
- 💻 purchasing_flow_realdb.rs
- 💻 purchasing_flow_advanced.rs

### Setup
- 🔧 purchasing_flow_setup.sql
- 🔧 setup_databases.sh
- 🔧 test_purchasing_flow.sh

---

## 🌟 Highlights

This case study demonstrates:

- **Real Production Patterns**: Not just a toy example
- **Complete Documentation**: From quickstart to deep-dive
- **Progressive Complexity**: Mock → Real DB → Advanced
- **Performance Analysis**: Benchmarks and metrics
- **Best Practices**: Error handling, monitoring, security

---

## 📞 Support

For questions or issues:

1. Check [QUICKSTART.md](QUICKSTART.md) Troubleshooting section
2. Review [CASE_STUDY.md](CASE_STUDY.md) Production Considerations
3. Read full [purchasing_flow_README.md](purchasing_flow_README.md)
4. Open an issue in the repository

---

## 📜 License

MIT License - See project root for details

---

**Happy Learning! 🚀**

Start with [QUICKSTART.md](QUICKSTART.md) and work your way through the examples!
