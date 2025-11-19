# Case Study Documentation

## 📚 Documentation Index

This directory contains complete documentation for the **Purchasing Flow Microservices** implementation.

### Current Documentation (v4.0 - Microservices with GRL)

| File | Purpose | Status |
|------|---------|--------|
| **[purchasing_flow_README.md](purchasing_flow_README.md)** | Complete implementation guide | ✅ Current |
| **[PURCHASING_FLOW_SUMMARY.md](PURCHASING_FLOW_SUMMARY.md)** | Technical summary & architecture | ✅ Current |
| [CASE_STUDY.md](CASE_STUDY.md) | Historical technical analysis | 📚 Reference |
| [CASE_STUDY_INDEX.md](CASE_STUDY_INDEX.md) | Old navigation guide | 📚 Reference |

---

## 🎯 Quick Start

### For New Users

1. **Read**: [purchasing_flow_README.md](purchasing_flow_README.md) - Full implementation guide
2. **Quick Reference**: [PURCHASING_FLOW_SUMMARY.md](PURCHASING_FLOW_SUMMARY.md) - Technical overview
3. **Parent Docs**: [../README.md](../README.md) - Main case study overview

### For Existing Users

If you were using older versions:
- **v1.0**: Monolithic with mock data → See [CASE_STUDY.md](CASE_STUDY.md)
- **v2.0**: Monolithic with MySQL → Historical content in docs
- **v3.0**: First gRPC implementation → Replaced by v4.0
- **v4.0**: Current microservices + GRL rules (this version)

---

## �️ Current Architecture (v4.0)

### Microservices with gRPC + GRL Rules

```
Client (HTTP)
    ↓
Orchestrator Service (:8080)
    ↓ (parallel gRPC)
    ├→ OMS Service (:50051)
    ├→ Inventory Service (:50052)
    ├→ Supplier Service (:50053)
    └→ UOM Service (:50054)
    ↓
Rule Engine Service (:50055, :8085)
    └→ GRL Rules (15 rules)
    ↓
Orchestrator executes based on flags
    └→ PO Service (:50056)
```

**Key Features:**
- ✅ 7 independent microservices
- ✅ gRPC for high-performance communication
- ✅ GRL (Generic Rule Language) for business rules
- ✅ Separation: Rule Engine calculates, Orchestrator executes
- ✅ Real MySQL databases per service
- ✅ Flag-based workflow execution

---

## 📖 Documentation Structure

### Main Files

1. **[purchasing_flow_README.md](purchasing_flow_README.md)**
   - Complete implementation guide
   - Architecture diagrams
   - Setup instructions
   - GRL rules explanation
   - Testing guide
   - Troubleshooting

2. **[PURCHASING_FLOW_SUMMARY.md](PURCHASING_FLOW_SUMMARY.md)**
   - Technical summary
   - Workflow execution details
   - Performance metrics
   - Future enhancements

### Related Documentation

- **[../GRPC.md](../GRPC.md)** - gRPC implementation details (if exists)
- **[../microservices/README.md](../microservices/README.md)** - Service-specific docs
- **[../README.md](../README.md)** - Main case study overview

---

## 🚀 Quick Commands

### Setup

```bash
# Database setup
cd case_study
./scripts/setup_databases.sh

# Build all services
cd microservices
./scripts/build-all.sh
```

### Run Services

```bash
cd case_study/microservices/services

# Start all (in background)
./oms-service/target/release/oms-service &
./inventory-service/target/release/inventory-service &
./supplier-service/target/release/supplier-service &
./uom-service/target/release/uom-service &
./po-service/target/release/po-service &
./rule-engine-service/target/release/rule-engine-service &
./orchestrator-service/target/release/orchestrator-service &
```

### Test

```bash
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-002"}'
```

---

## 🔍 What's New in v4.0

### Major Changes

1. **GRL Rule Engine**
   - 15 business rules in declarative format
   - Expression evaluation (arithmetic, comparisons)
   - Salience-based priority
   - No-loop flags

2. **Flag-Based Execution**
   - Rules return decision flags (`should_create_po`, `should_send_po`)
   - Orchestrator reads flags and executes
   - Clear separation of concerns

3. **Updated Proto**
   - Added workflow flags to `EvaluateResponse`
   - `should_create_po`, `should_send_po`, `po_status`, `send_method`

4. **Calculation Mode**
   - Rule engine in pure calculation mode
   - No action execution in rules
   - All execution in orchestrator

### Breaking Changes

- Action handlers removed from GRL rules
- Orchestrator now handles all PO creation/sending
- Proto messages updated with new fields

---

## 📊 System Status

### Services

| Service | Port | Status | Database |
|---------|------|--------|----------|
| Orchestrator | 8080 (HTTP) | ✅ Active | None |
| OMS | 50051 (gRPC) | ✅ Active | oms_db |
| Inventory | 50052 (gRPC) | ✅ Active | inventory_db |
| Supplier | 50053 (gRPC) | ✅ Active | supplier_db |
| UOM | 50054 (gRPC) | ✅ Active | uom_db |
| Rule Engine | 50055 (gRPC), 8085 (HTTP) | ✅ Active | None |
| PO | 50056 (gRPC) | ✅ Active | po_db |

### Test Data

- ✅ PROD-001: 100 units order expected
- ✅ PROD-002: 245 units order expected  
- ✅ PROD-003: 110 units order expected

---

## 🆘 Support

### Common Issues

1. **Services not starting** → Check ports with `lsof -i :8080`
2. **Database errors** → Verify `.env` file and run setup script
3. **gRPC errors** → Ensure all services are running
4. **Rules not firing** → Check rule engine logs for initial values

### Logs

```bash
tail -f /tmp/orchestrator.log
tail -f /tmp/rule-engine.log
tail -f /tmp/po.log
```

### Documentation

- Full guide: [purchasing_flow_README.md](purchasing_flow_README.md)
- Quick reference: [PURCHASING_FLOW_SUMMARY.md](PURCHASING_FLOW_SUMMARY.md)

---

**Version:** v4.0 (Microservices + GRL)  
**Last Updated:** November 2025  
**Status:** ✅ Production Ready (with additional hardening needed)
