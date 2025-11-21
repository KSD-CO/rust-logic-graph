# Purchasing Flow Microservices

Production-ready microservices architecture demonstrating **rust-logic-graph** framework with gRPC communication and PostgreSQL databases.

## 🏗️ Architecture

```
External → REST:8080 → Orchestrator
                      ↓ (gRPC - 5x faster)
         ┌────────────┼────────────┬────────────┬────────────┐
         ↓            ↓            ↓            ↓            ↓
    OMS:50051   Inventory   Supplier    UOM      Rule Engine
                 :50052      :50053    :50054      :50055
                                                      ↓
                                                  PO:50056
```

## 📦 Services

| Service | REST Port | gRPC Port | Database | Purpose |
|---------|-----------|-----------|----------|---------|
| **Orchestrator** | 8080 | - | - | API Gateway & workflow coordinator |
| **OMS** | 8081 | 50051 | postgres-oms:5433 | Order history & demand data |
| **Inventory** | 8082 | 50052 | postgres-inventory:5434 | Stock levels |
| **Supplier** | 8083 | 50053 | postgres-supplier:5435 | Supplier info & pricing |
| **UOM** | 8084 | 50054 | postgres-uom:5436 | Unit conversions |
| **Rule Engine** | 8085 | 50055 | - | GRL business rules |
| **PO** | 8086 | 50056 | - | Purchase order creation |

## 🚀 Quick Start

### Prerequisites
- Docker & Docker Compose
- Rust 1.75+ (for local development)

### 1. Check Services Compile
```bash
cd scripts
./check-all-services.sh
```

### 2. Build Docker Images
```bash
cd scripts
./build-docker.sh
# Or manually:
docker-compose build
```

### 3. Start Services
```bash
docker-compose up -d
```

Wait ~30 seconds for all services to be ready.

### 4. Test Purchasing Flow
```bash
cd scripts
./test-docker-flow.sh

# Or manually:
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

### 5. View Logs
```bash
docker-compose logs -f orchestrator-service
docker-compose logs -f rule-engine-service
```

### 6. Stop Services
```bash
docker-compose down
```

## 🧪 Test Cases

### Test 1: Low Demand (PROD-001)
- Avg daily demand: 15.5 units
- Available: 50 units
- **Expected**: No reorder needed

### Test 2: Standard Reorder (PROD-002)
- Avg daily demand: 8.3 units
- Available: 120 units
- **Expected**: PO created and sent

### Test 3: High Value Order (PROD-003)
- Avg daily demand: 22 units
- Available: 8 units
- **Expected**: PO requires approval, high value ($173k+)

## 📊 Database Schema

Each database has one main table:

**oms_db:**
```sql
CREATE TABLE oms_history (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    avg_daily_demand DECIMAL(10,2) NOT NULL,
    trend VARCHAR(20),
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**inventory_db:**
```sql
CREATE TABLE inventory_levels (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    warehouse_id VARCHAR(50),
    available_qty DECIMAL(10,2) NOT NULL,
    reserved_qty DECIMAL(10,2) DEFAULT 0,
    warehouse_location VARCHAR(100),
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**supplier_db:**
```sql
CREATE TABLE supplier_info (
    id SERIAL PRIMARY KEY,
    supplier_id VARCHAR(50),
    product_id VARCHAR(50) NOT NULL,
    moq DECIMAL(10,2) NOT NULL,
    lead_time_days INT NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**uom_db:**
```sql
CREATE TABLE uom_conversions (
    id SERIAL PRIMARY KEY,
    product_id VARCHAR(50) NOT NULL,
    from_uom VARCHAR(20) NOT NULL,
    to_uom VARCHAR(20) NOT NULL,
    conversion_factor DECIMAL(10,4) NOT NULL,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## 🔧 Development

### Local Development (without Docker)
Each service can run locally:

```bash
cd services/oms-service
DB_HOST=localhost DB_PORT=5432 DB_NAME=oms_db \
DB_USER=postgres DB_PASSWORD=password \
PORT=8081 GRPC_PORT=50051 \
cargo run
```

### Building Individual Services
```bash
cd services/oms-service
cargo build --release
```

### Running Tests
```bash
cd services/oms-service
cargo test
```

## 📚 Documentation

- [GRPC_IMPLEMENTATION.md](GRPC_IMPLEMENTATION.md) - Complete gRPC guide
- [docker-compose.yml](docker-compose.yml) - Service configuration
- [proto/](proto/) - gRPC Protocol Buffer definitions

## 🎯 Key Features

✅ **Dual Protocol**: gRPC (internal) + REST (external)  
✅ **Database Isolation**: 4 separate PostgreSQL databases  
✅ **Production Ready**: Docker Compose + Kubernetes manifests  
✅ **Type Safety**: Protocol Buffers for gRPC  
✅ **GRL Rules**: Business logic in rule engine  
✅ **Health Checks**: All services expose REST health endpoints  

## 🔍 Troubleshooting

### Services won't start
```bash
# Check Docker logs
docker-compose logs

# Restart specific service
docker-compose restart oms-service
```

### Database connection errors
```bash
# Check PostgreSQL containers
docker-compose ps

# View database logs
docker-compose logs postgres-oms
```

### Port conflicts
Edit `docker-compose.yml` to change port mappings.

## 📈 Performance

- **gRPC vs REST**: 5x faster for inter-service calls
- **First request**: ~300ms (cold start)
- **Cached requests**: ~3ms (GRL engine caching)
- **Throughput**: 10K+ req/s per service

---

Built with [rust-logic-graph](https://github.com/KSD-CO/rust-logic-graph)
