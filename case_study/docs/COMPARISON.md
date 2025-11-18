# Purchasing Flow: Mock vs Real Database Comparison

> ⚠️ **OUTDATED DOCUMENTATION - v1.0**
>
> This document references the old monolithic structure with mock vs real database versions.
>
> **For current documentation:**
> - **[Main README](../README.md)** - Current project overview
> - **[GRPC.md](../GRPC.md)** - gRPC implementation
> - **[MICROSERVICES_DEPLOYMENT.md](../MICROSERVICES_DEPLOYMENT.md)** - Production deployment
>
> **Current structure:**
> - Monolithic: Uses real MySQL databases with Clean Architecture
> - Microservices: Production-ready with gRPC + REST dual protocol

---

## ⚠️ Historical Content Below (For Reference Only)

## Overview

| Feature | Mock Version | Real DB Version |
|---------|--------------|-----------------|
| File | `purchasing_flow.rs` | `purchasing_flow_realdb.rs` |
| Data Source | Hardcoded JSON | MySQL Database |
| Databases | None | 4 separate databases |
| Dependencies | Basic | `sqlx`, `chrono` |
| Cargo Features | None | `--features mysql` |
| Setup Required | None | Database setup needed |
| Realistic | Demo only | Production-like |

## Architecture Comparison

### Mock Version (purchasing_flow.rs)

```
┌─────────────┐
│   DBNode    │  Returns: json!({"avg": 10})
│ (mock data) │  Connection: None
└─────────────┘
```

**Characteristics:**
- No external dependencies
- Instant execution
- Predictable results
- Good for demos/testing
- No setup required

### Real DB Version (purchasing_flow_realdb.rs)

```
┌─────────────┐
│ MySQLDBNode │  Query: SELECT * FROM oms_history...
│  (oms_db)   │  Connection: Pool<MySql>
└─────────────┘
       │
       ▼
┌─────────────┐
│  MySQL DB   │  Host: 171.244.10.40:6033
│   oms_db    │  User: user
└─────────────┘
```

**Characteristics:**
- Real database connections
- Network I/O involved
- Production-like behavior
- Requires setup
- Multiple databases

---

**Note:** This comparison is historical. The current project uses only real databases in both monolithic and microservices architectures.
