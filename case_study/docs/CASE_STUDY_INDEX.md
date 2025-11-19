# Purchasing Flow Case Study - Documentation Index

> **📍 You Are Here:** `/case_study/docs/` - Case study documentation directory

---

## 📚 Current Documentation (v4.0 - November 2024)

**Architecture:** 7 Microservices with gRPC + REST, Flag-based execution

### Start Here

1. **[README.md](README.md)** ✅ **MAIN INDEX**
   - Central navigation hub for all documentation
   - Quick start commands
   - System status and version info
   - Links to all current and historical docs

2. **[purchasing_flow_README.md](purchasing_flow_README.md)** ✅ **IMPLEMENTATION GUIDE**
   - Complete v4.0 microservices architecture
   - 7 services with detailed descriptions
   - Setup and running instructions
   - GRL rules explanation (15 business rules)
   - Testing and troubleshooting

3. **[PURCHASING_FLOW_SUMMARY.md](PURCHASING_FLOW_SUMMARY.md)** ✅ **TECHNICAL SUMMARY**
   - Architecture overview with diagrams
   - Workflow execution flow
   - Performance metrics (30-50ms E2E)
   - Monitoring and debugging
   - Future enhancements

---

## 📚 Historical Documentation (v1.0-v3.0)

**Preserved for reference - shows architectural evolution**

### Monolithic Architecture (v1.0-v2.0)

4. **[CASE_STUDY.md](CASE_STUDY.md)** 📚 **Historical**
   - Original monolithic architecture with MySQL DBNodes
   - Graph-based execution patterns
   - Performance analysis (sub-500ms)
   - Production considerations
   - Evolution timeline (v1.0 → v4.0)

5. **[QUICKSTART.md](QUICKSTART.md)** 📚 **Historical**
   - Quick commands for monolithic version
   - Mock vs real database comparison
   - Original troubleshooting tips

6. **[COMPARISON.md](COMPARISON.md)** 📚 **Historical**
   - Mock vs Real Database analysis
   - Performance comparisons
   - Use case recommendations

---

## 🗺️ Directory Structure

```
case_study/docs/
├── README.md                       # ✅ Main navigation hub (START HERE)
├── purchasing_flow_README.md       # ✅ v4.0 implementation guide
├── PURCHASING_FLOW_SUMMARY.md      # ✅ v4.0 technical summary
├── CASE_STUDY_INDEX.md             # 📍 This file
├── CASE_STUDY.md                   # 📚 Historical (v1.0-v2.0)
├── QUICKSTART.md                   # 📚 Historical
└── COMPARISON.md                   # 📚 Historical
```

---

## 🎯 Quick Navigation

### For New Users
→ Start with **[README.md](README.md)** for system overview
→ Then read **[purchasing_flow_README.md](purchasing_flow_README.md)** for implementation

### For Developers
→ **[PURCHASING_FLOW_SUMMARY.md](PURCHASING_FLOW_SUMMARY.md)** - Technical details
→ Check GRL rules in `/case_study/microservices/services/rule-engine-service/rules/purchasing_rules.grl`

### For Historical Context
→ **[CASE_STUDY.md](CASE_STUDY.md)** - Evolution from monolithic to microservices
→ See how architecture evolved from DBNodes to gRPC services

---

## 🔄 Version History

| Version | Date | Architecture | Documentation |
|---------|------|--------------|---------------|
| **v4.0** | Nov 2024 | Microservices (gRPC) | ✅ Current (purchasing_flow_README.md) |
| **v3.0** | Oct 2024 | Hybrid | 📚 Historical |
| **v2.0** | Sep 2024 | Monolithic (Graph + GRL) | 📚 Historical |
| **v1.0** | Aug 2024 | Monolithic (Graph only) | 📚 Historical (CASE_STUDY.md) |

---

## 🚀 Quick Start

```bash
# 1. Navigate to microservices directory
cd case_study/microservices

# 2. Setup databases (one-time)
./scripts/setup_databases.sh

# 3. Start all services (use 7 terminals or tmux)
# See purchasing_flow_README.md for detailed commands

# 4. Test the flow
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

For complete instructions, see **[purchasing_flow_README.md](purchasing_flow_README.md)**

---

**Maintained by:** James Vu  
**Last Updated:** November 2024  
**Status:** Active Development (v4.0)
