# Purchasing Flow - Production Case Study

> **A complete, production-grade distributed purchasing system with dual architectures: Monolithic & Microservices**
>
> **Built with `rust-logic-graph`, Rete Algorithm, and Kubernetes-ready**

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
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

**1. Monolithic (Clean Architecture)**
- Single binary with modular design
- Fast development iteration
- Simple deployment
- Perfect for learning and prototyping
- Clean separation of concerns

**2. Microservices (Kubernetes-Ready)**
- 7 independent services
- Kubernetes-ready with full manifests
- Horizontal scaling per service
- Production-grade with Docker Compose
- **Rete algorithm** for rule engine

### Production Patterns
✅ **Multi-database architecture** - 4 separate MySQL databases (OMS, Inventory, Supplier, UOM)
✅ **Async/await processing** - Parallel queries with connection pooling
✅ **Rete rule engine** - Incremental pattern matching for business rules
✅ **Dual Protocol** - gRPC for inter-service + REST for external APIs
✅ **Clean Architecture** - Separation of concerns in monolithic version
✅ **Error handling** - Comprehensive error management
✅ **Production logging** - Structured, informative output
✅ **Container-ready** - Docker & Kubernetes deployment

---

## 🏗️ Architecture Options

Choose the architecture that fits your needs:

### Option 1: Monolithic (Clean Architecture) 🚀

Perfect for: Development, Testing, Learning, Single Server Deployment

```bash
# Navigate to case study
cd case_study

# Setup databases (one-time)
./scripts/setup_databases.sh

# Run monolithic version
./scripts/run_monolithic.sh
```

**Benefits:**
- ✅ Single binary
- ✅ < 1 second startup
- ✅ Easy debugging
- ✅ Clean Architecture pattern
- ✅ No container overhead

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

# 2. Setup databases (one-time)
./scripts/setup_databases.sh

# 3. Run the monolithic version
./scripts/run_monolithic.sh
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
├── monolithic/                       # 🏛️ Monolithic Architecture
│   ├── Cargo.toml                    # Monolithic build config
│   ├── src/                          # Clean Architecture source code
│   │   ├── main.rs                   # Entry point
│   │   ├── config.rs                 # Configuration
│   │   ├── models.rs                 # Data models
│   │   ├── handlers/                 # Request handlers
│   │   ├── services/                 # Business logic services
│   │   └── utils/                    # Utilities (DB, metrics, timer)
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
│   ├── setup_databases.sh            # Database setup
│   ├── run_monolithic.sh             # Run monolithic ⭐
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
| **[GRPC.md](GRPC.md)** | gRPC implementation guide | Developers | 15 min 🔥 NEW |
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
2. Read [GRPC.md](GRPC.md) - gRPC architecture
3. Run monolithic: `./scripts/run_monolithic.sh`

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

## 🛠️ Quick Commands

### Monolithic

```bash
# Run monolithic version
./scripts/run_monolithic.sh

# Build
cd monolithic
cargo build --features mysql

# Run tests
cargo test --features mysql
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


### Databases

1. **oms_db** - Order Management System
2. **inventory_db** - Inventory Management
3. **supplier_db** - Supplier Management
4. **uom_db** - Unit of Measure

---

## 📜 License

MIT License - See project root for details

---

## 🎉 Get Started!

```bash
# Navigate to case study
cd case_study

# Setup databases
./scripts/setup_databases.sh

# Run monolithic version
./scripts/run_monolithic.sh

# Or run microservices
./scripts/build-all.sh
cd microservices && docker-compose up -d
```

**Happy Learning! 🚀**

---

<div align="center">

**[Monolithic](monolithic/)** • **[Microservices](microservices/)** • **[Documentation](docs/)** • **[Scripts](scripts/)**

*Rust Logic Graph v0.7.0 - Production-Grade Workflow Orchestration*

</div>
