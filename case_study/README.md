# Purchasing Flow - Production Case Study

> **A complete, production-grade distributed purchasing system with dual architectures: Monolithic & Microservices**
>
> **Built with `rust-logic-graph`, Rete Algorithm, and Kubernetes-ready**

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-14+-blue.svg)](https://www.postgresql.org)
[![MySQL](https://img.shields.io/badge/MySQL-8.0+-blue.svg)](https://www.mysql.com)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue.svg)](https://www.docker.com)
[![Kubernetes](https://img.shields.io/badge/Kubernetes-Ready-326CE5.svg)](https://kubernetes.io)
[![Rete](https://img.shields.io/badge/Rete-Algorithm-orange.svg)](https://en.wikipedia.org/wiki/Rete_algorithm)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](../LICENSE)

**[Quick Start](#-quick-start)** • **[Architecture](#-architecture-options)** • **[Microservices](#-microservices-deployment)** • **[Documentation](#-documentation)**

</div>

---

## 📋 Table of Contents

- [What Is This?](#-what-is-this)
- [Architecture Options](#-architecture-options)
- [Quick Start](#-quick-start)
- [Directory Structure](#-directory-structure)
- [Documentation](#-documentation)
- [Use Cases](#-use-cases)

---

## 🎯 What Is This?

This case study demonstrates how to build a **real-world distributed purchasing automation system** using the `rust-logic-graph` framework with **two deployment architectures**:

### 🏛️ Dual Architecture Support

**1. Monolithic (Multi-Database YAML-Driven)**
- Single binary with distributed data
- 4 separate PostgreSQL databases
- YAML-driven database routing per node
- Fast development iteration
- Simple deployment (single process)
- Perfect for learning and prototyping
- **Data isolation like microservices**

**2. Microservices (Kubernetes-Ready)**
- 7 independent services
- Kubernetes-ready with full manifests
- Horizontal scaling per service
- Production-grade with Docker Compose
- **Rete algorithm** for rule engine

### Production Patterns
✅ **Multi-database architecture** - 4 separate PostgreSQL databases (OMS, Inventory, Supplier, UOM)
✅ **Distributed data with single process** - Monolithic benefits + microservices data isolation
✅ **YAML-driven database routing** - Each node specifies target database in config
✅ **Async/await processing** - Parallel queries with connection pooling per database
✅ **Rete rule engine** - Incremental pattern matching for business rules
✅ **Dual Protocol** - gRPC for inter-service + REST for external APIs
✅ **Clean Architecture** - Separation of concerns in monolithic version
✅ **Error handling** - Comprehensive error management
✅ **Production logging** - Structured, informative output
✅ **Container-ready** - Docker & Kubernetes deployment

---

## 🏗️ Architecture Options

Choose the architecture that fits your needs:

### Option 1: Monolithic (Multi-Database YAML-Driven) 🚀

Perfect for: Development, Testing, Learning, Single Server Deployment

**Architecture:**
```
purchasing_flow_graph.yaml (Source of Truth)
  ↓ each node specifies database
  ↓ SQL queries + business rules
GraphExecutor (Multi-Pool Manager)
  ├─ pools["oms_db"] → PostgreSQL
  ├─ pools["inventory_db"] → PostgreSQL
  ├─ pools["supplier_db"] → PostgreSQL
  └─ pools["uom_db"] → PostgreSQL
  ↓ creates dynamic nodes
DynamicDBNode + DynamicRuleNode
  ↓ routes to correct database
4 Separate PostgreSQL Databases
```

**Key Features:**
- 🎯 **100% YAML-driven** - All SQL queries + database routing in config
- 🔥 **Multi-database support** - Each node queries its own database
- ⚡ **Dynamic routing** - Nodes automatically use correct database pool
- 📋 **Distributed data** - Simulates microservices data isolation
- 🗄️ **PostgreSQL** - Production-grade database with ACID compliance

```bash
# Setup 4 separate databases (one-time)
./scripts/setup_multi_databases.sh

# Run monolithic version
cd monolithic
cargo run
```

**Benefits:**
- ✅ Single binary, single process
- ✅ < 2 seconds startup
- ✅ Easy debugging with full stack traces
- ✅ Config-driven architecture (no code changes for queries)
- ✅ Multi-database architecture (simulates microservices isolation)
- ✅ No network overhead
- ✅ PostgreSQL ACID transactions

### Option 2: Microservices (Production) 🎯

Perfect for: Production, Kubernetes, Cloud Deployment

```bash
# Navigate to case study
cd case_study

# Start with Docker Compose
docker-compose -f microservices/docker-compose.yml up -d

# Or deploy to Kubernetes
./scripts/deploy-k8s.sh

# Test the API
./scripts/test-api.sh
```

**Benefits:**
- ✅ Independent scaling
- ✅ Fault isolation
- ✅ **gRPC** for 5x faster inter-service communication
- ✅ Rete algorithm rule engine
- ✅ Cloud-native architecture
- ✅ Kubernetes-ready

---

## 🚀 Quick Start

### Monolithic (5 Minutes)

```bash
# 1. Navigate to case study directory
cd case_study

# 2. Setup 4 separate databases (one-time)
./scripts/setup_multi_databases.sh

# 3. Run the monolithic version
cd monolithic
cargo run

# 4. Test the API (in another terminal)
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

### Microservices (10 Minutes)

```bash
# 1. Navigate to case study
cd case_study

# 2. Build all services
./scripts/build-all.sh

# 3. Start with Docker Compose
cd microservices
docker-compose up -d

# 4. Test the API
cd ..
./scripts/test-api.sh

# 5. View logs
cd microservices
docker-compose logs -f orchestrator-service
```

**Expected Output**: Complete purchasing flow with real database queries, rule evaluation, and generated purchase order.

---

## 📁 Directory Structure

```
case_study/
├── README.md                         # This file - Main entry point ⭐
├── GRPC.md                           # gRPC implementation guide 🔥
├── MICROSERVICES_DEPLOYMENT.md       # Kubernetes deployment guide
│
├── monolithic/                       # 🏛️ Monolithic Architecture (Multi-DB YAML-Driven)
│   ├── Cargo.toml                    # Monolithic build config
│   ├── purchasing_flow_graph.yaml   # ⭐ Graph definition with SQL + DB routing
│   ├── .env                          # Database configuration
│   ├── src/                          # Source code
│   │   ├── main.rs                   # Entry point with multi-pool setup
│   │   ├── config.rs                 # Multi-database configuration
│   │   ├── models.rs                 # Data models
│   │   ├── graph_config.rs           # YAML parser (with database field)
│   │   ├── graph_executor.rs         # Multi-pool executor engine
│   │   ├── db_executor.rs            # Database executor implementations
│   │   └── utils/                    # Utilities (PostgreSQL pools, metrics)
│   └── shared/models/                # Shared data structures
│
├── microservices/                    # 🎯 Microservices Architecture
│   ├── docker-compose.yml            # Local development environment
│   ├── proto/                        # Protocol Buffer definitions
│   │   ├── oms.proto
│   │   ├── inventory.proto
│   │   ├── supplier.proto
│   │   └── uom.proto
│   ├── shared/models/                # Shared models for microservices
│   ├── services/                     # 7 independent services
│   │   ├── orchestrator-service/     # API Gateway (Port 8080)
│   │   ├── oms-service/              # OMS data (Port 8081, gRPC 50051)
│   │   ├── inventory-service/        # Inventory (Port 8082, gRPC 50052)
│   │   ├── supplier-service/         # Supplier (Port 8083, gRPC 50053)
│   │   ├── uom-service/              # UOM (Port 8084, gRPC 50054)
│   │   ├── rule-engine-service/      # Rete engine (Port 8085)
│   │   └── po-service/               # Purchase Orders (Port 8086)
│   └── k8s/                          # Kubernetes manifests
│       ├── namespace.yaml
│       ├── deployments/              # Deployments for all services
│       ├── services/                 # Service definitions
│       ├── configmaps/               # Configuration
│       └── secrets/                  # Secrets
│
├── rules/                            # Rule definitions (JSON format)
├── scripts/                          # Helper scripts
│   ├── setup_databases.sh            # Legacy: Single database setup
│   ├── setup_multi_databases.sh      # ⭐ NEW: Multi-database setup (4 DBs)
│   ├── run_monolithic.sh             # Run monolithic
│   ├── build-all.sh                  # Build all Docker images
│   ├── deploy-k8s.sh                 # Deploy to Kubernetes
│   └── test-api.sh                   # Test microservices API
│
├── sql/                              # Database setup SQL scripts
└── docs/                             # Historical documentation (v1.0)
```

---

## 📚 Documentation

### Core Documentation

| Document | Purpose | Audience | Time |
|----------|---------|----------|------|
| **[README.md](README.md)** | Project overview & quick start | Everyone | 10 min ⭐ START HERE |
| **[MULTI_DATABASE_ARCHITECTURE.md](MULTI_DATABASE_ARCHITECTURE.md)** | Multi-database pattern + Multi-server setup | Developers/DevOps | 20 min 🔥 |
| **[GRPC.md](GRPC.md)** | gRPC implementation guide | Developers | 15 min |
| **[MICROSERVICES_DEPLOYMENT.md](MICROSERVICES_DEPLOYMENT.md)** | Kubernetes deployment | DevOps | 30 min |

### Microservices Documentation

| Document | Purpose |
|----------|---------|
| **[microservices/services/README.md](microservices/services/README.md)** | Services overview |
| **[microservices/GRPC_IMPLEMENTATION.md](microservices/GRPC_IMPLEMENTATION.md)** | gRPC implementation details |
| **[microservices/proto/README.md](microservices/proto/README.md)** | Protocol Buffer definitions |

### Historical Documentation (v1.0)

| Document | Purpose | Status |
|----------|---------|--------|
| **[docs/QUICKSTART.md](docs/QUICKSTART.md)** | Old quick start | ⚠️ Outdated |
| **[docs/CASE_STUDY.md](docs/CASE_STUDY.md)** | Original case study | ⚠️ Historical |
| **[docs/purchasing_flow_README.md](docs/purchasing_flow_README.md)** | Full reference | ⚠️ Historical |

### By Use Case

**"I want to learn the system"**
1. Read [README.md](README.md) - Overview
2. Read [MULTI_DATABASE_ARCHITECTURE.md](MULTI_DATABASE_ARCHITECTURE.md) - Multi-DB pattern ⭐
3. Read [GRPC.md](GRPC.md) - gRPC architecture
4. Run monolithic: `cd monolithic && cargo run`

**"I want to connect to multiple database servers"**
1. Read [MULTI_DATABASE_ARCHITECTURE.md](MULTI_DATABASE_ARCHITECTURE.md) - Multi-server setup ⭐
2. Add `connection` field to YAML config
3. Run app: `cd monolithic && cargo run`
4. Test with Docker: Multiple PostgreSQL containers on different ports

**"I want to deploy to production"**
1. Read [README.md](README.md) - Choose architecture (Monolithic vs Microservices)
2. Read [MICROSERVICES_DEPLOYMENT.md](MICROSERVICES_DEPLOYMENT.md) - Deployment guide
3. Read [GRPC.md](GRPC.md) - gRPC implementation

**"I want to understand gRPC"**
1. Read [GRPC.md](GRPC.md) - Complete gRPC guide ⭐
2. Read [microservices/GRPC_IMPLEMENTATION.md](microservices/GRPC_IMPLEMENTATION.md) - Implementation details
3. Read [microservices/proto/README.md](microservices/proto/README.md) - Proto definitions

---

## 🎯 Use Cases

This case study is perfect for:

### 1. Learning
- Understanding distributed systems
- Learning async Rust patterns
- Studying workflow orchestration
- Exploring Clean Architecture

### 2. Building
- Supply chain automation
- E-commerce purchasing systems
- Inventory management
- Order processing workflows

### 3. Reference
- Production architecture patterns
- Microservices design
- Clean Architecture implementation
- Database integration strategies

### 4. Teaching
- Workshop material
- Code examples
- Architecture demonstrations
- Best practices showcase

---

## � Architecture Comparison

### Monolithic vs Microservices

| Aspect | Monolithic (YAML-Driven) | Microservices (gRPC) |
|--------|-------------------------|---------------------|
| **Deployment** | Single binary | 7 independent services |
| **Configuration** | YAML file (purchasing_flow_graph.yaml) | YAML files + gRPC proto |
| **SQL Queries** | ✅ In YAML config | ❌ In service code |
| **Communication** | Direct function calls | gRPC protocol |
| **Nodes** | DBNode + RuleNode | GrpcNode + RuleNode |
| **Startup Time** | < 1 second | ~10 seconds (all services) |
| **Scaling** | Vertical only | Horizontal per service |
| **Best For** | Development, Testing, Single server | Production, Cloud, K8s |

### Code Architecture

**Monolithic (Config-Driven):**
```yaml
# purchasing_flow_graph.yaml - Single source of truth
nodes:
  oms_history:
    type: DBNode
    query: "SELECT product_id, avg_daily_demand FROM oms_history WHERE product_id = $1"
  
  rule_engine:
    type: RuleNode
    condition: "inventory < demand * lead_time"
```

**Microservices (Service-Driven):**
```yaml
# purchasing_flow_graph.yaml - Service orchestration
nodes:
  oms_grpc:
    type: GrpcNode
    query: "http://localhost:50051#GetOrderHistory"
  
  rule_engine_grpc:
    type: RuleNode
    # Rules evaluated in separate service
```

---

## ️ Quick Commands

### Monolithic

```bash
# Setup 4 separate databases
./scripts/setup_multi_databases.sh

# Run monolithic version
cd monolithic
cargo run

# Test the API
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'

# Build for production
cd monolithic
cargo build --release

# Verify databases
psql -h localhost -p 5432 -U postgres -d oms_db -c "SELECT * FROM oms_history;"
psql -h localhost -p 5432 -U postgres -d inventory_db -c "SELECT * FROM inventory;"
```

### Microservices

```bash
# Build all images
./scripts/build-all.sh

# Local development
cd microservices
docker-compose up -d
docker-compose logs -f
docker-compose down

# Kubernetes
./scripts/deploy-k8s.sh
kubectl get pods -n purchasing-flow
./scripts/cleanup-k8s.sh
```

---

## 🗄️ Database Configuration

### Multi-Database Architecture (Monolithic)

**4 Separate PostgreSQL Databases:**

| Database | Table | Purpose | Sample Data |
|----------|-------|---------|-------------|
| `oms_db` | `oms_history` | Order history & demand trends | PROD-001: 150/day (increasing) |
| `inventory_db` | `inventory` | Current stock levels | PROD-001: 500 units available |
| `supplier_db` | `suppliers` | Supplier info & pricing | PROD-001: $15.50, MOQ 100, 7 days |
| `uom_db` | `uom_conversions` | Unit conversions | PROD-001: 12 pieces = 1 box |

**Connection Pooling:**
```rust
// Each database has its own connection pool
PurchasingGraphExecutor {
    pools: HashMap<String, DatabasePool>
    ├─ "oms_db" → PgPool (oms_db)
    ├─ "inventory_db" → PgPool (inventory_db)  
    ├─ "supplier_db" → PgPool (supplier_db)
    └─ "uom_db" → PgPool (uom_db)
}
```

**Benefits:**
- ✅ **Data isolation** - Each domain has separate database (like microservices)
- ✅ **Independent scaling** - Can optimize each DB separately
- ✅ **Clear boundaries** - Forces proper separation of concerns
- ✅ **Migration path** - Easy to split into microservices later

---

## 📜 License

MIT License - See project root for details

---

## 🎉 Get Started!

```bash
# Navigate to case study
cd case_study

# Setup 4 separate databases (PostgreSQL)
./scripts/setup_multi_databases.sh

# Run monolithic version (multi-database)
cd monolithic
cargo run

# Test the purchasing flow
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'

# Expected response:
# {
#   "status": "success",
#   "message": "Purchase order created successfully",
#   "purchase_order": {
#     "product_id": "PROD-001",
#     "order_qty": 550.0,
#     "supplier_id": "SUPP-PROD-001",
#     "total_cost": 8525.0
#   }
# }

# Or run microservices (7 services with MySQL)
cd ../
./scripts/build-all.sh
cd microservices && docker-compose up -d
```

**Next Steps:**
- Read [GRPC.md](GRPC.md) for microservices architecture
- Check [monolithic/purchasing_flow_graph.yaml](monolithic/purchasing_flow_graph.yaml) to see YAML config
- Explore [docs/](docs/) for detailed documentation

**Happy Learning! 🚀**

---

<div align="center">

**[Monolithic](monolithic/)** • **[Microservices](microservices/)** • **[Documentation](docs/)** • **[Scripts](scripts/)**

*Rust Logic Graph v0.7.0 - Production-Grade Workflow Orchestration*

</div>
