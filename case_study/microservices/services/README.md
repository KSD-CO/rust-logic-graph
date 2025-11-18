# Purchasing Flow Microservices

This directory contains all microservices for the Purchasing Flow application.

## Services Overview

### 1. Orchestrator Service (Port 8080)
**Role:** API Gateway and Workflow Orchestrator

**Responsibilities:**
- Exposes main API endpoint for purchasing flow
- Orchestrates communication between all services
- Aggregates data from multiple services
- Manages the workflow sequence

**Endpoints:**
- `GET /health` - Health check
- `POST /purchasing/flow` - Execute purchasing flow

**Dependencies:** All other services

---

### 2. OMS Service (Port 8081)
**Role:** Order Management System Data Provider

**Responsibilities:**
- Provides historical order data
- Returns average daily demand and trends

**Endpoints:**
- `GET /health` - Health check
- `GET /oms/history/{product_id}` - Get OMS history data

**Database:** oms_db (MySQL)

---

### 3. Inventory Service (Port 8082)
**Role:** Inventory Data Provider

**Responsibilities:**
- Provides current inventory levels
- Returns available, reserved, and current quantities

**Endpoints:**
- `GET /health` - Health check
- `GET /inventory/levels/{product_id}` - Get inventory levels

**Database:** inventory_db (MySQL)

---

### 4. Supplier Service (Port 8083)
**Role:** Supplier Information Provider

**Responsibilities:**
- Provides supplier information
- Returns MOQ, lead times, and pricing

**Endpoints:**
- `GET /health` - Health check
- `GET /supplier/info/{product_id}` - Get supplier information

**Database:** supplier_db (MySQL)

---

### 5. UOM Service (Port 8084)
**Role:** Unit of Measurement Conversion Provider

**Responsibilities:**
- Provides UOM conversion factors
- Handles unit conversions

**Endpoints:**
- `GET /health` - Health check
- `GET /uom/conversion/{product_id}` - Get UOM conversion data

**Database:** uom_db (MySQL)

---

### 6. Rule Engine Service (Port 8085)
**Role:** Business Rules Processor

**Responsibilities:**
- Evaluates GRL (Generic Rule Language) business rules
- Calculates order quantities
- Determines approval requirements
- Applies safety multipliers and trend adjustments

**Endpoints:**
- `GET /health` - Health check
- `POST /evaluate` - Evaluate business rules

**Dependencies:** GRL rules file

---

### 7. PO Service (Port 8086)
**Role:** Purchase Order Manager

**Responsibilities:**
- Creates purchase orders
- Sends POs to suppliers
- Manages PO lifecycle

**Endpoints:**
- `GET /health` - Health check
- `POST /po/create` - Create purchase order
- `POST /po/send` - Send purchase order

---

## Communication Flow

```
Client Request
     │
     ▼
Orchestrator Service (8080)
     │
     ├─► OMS Service (8081)        ──► MySQL (oms_db)
     ├─► Inventory Service (8082)  ──► MySQL (inventory_db)
     ├─► Supplier Service (8083)   ──► MySQL (supplier_db)
     └─► UOM Service (8084)        ──► MySQL (uom_db)
     │
     ▼
Rule Engine Service (8085)
     │
     ▼
PO Service (8086)
     │
     ▼
Client Response
```

## Technology Stack

- **Language:** Rust
- **Web Framework:** Axum 0.7
- **Database Driver:** SQLx 0.7 (MySQL)
- **HTTP Client:** Reqwest 0.11
- **Serialization:** Serde + serde_json
- **Logging:** Tracing + tracing-subscriber
- **Async Runtime:** Tokio
- **Rule Engine:** rust-rule-engine 0.14

## Shared Library

### purchasing-models
Located in `../shared/models/`

Provides shared data structures used across all services:
- Request/Response models
- Data models (OMS, Inventory, Supplier, UOM)
- Purchase Order models
- Health check responses

## Building Services

### Individual Service
```bash
cd services/oms-service
cargo build --release
```

### All Services (Docker)
```bash
cd ../..
./scripts/build-all.sh
```

## Running Services Locally

### Individual Service
```bash
# Set environment variables
export DB_USER=root
export DB_PASSWORD=password
export DB_HOST=localhost
export DB_PORT=3306
export OMS_DB=oms_db
export PORT=8081

# Run
cargo run
```

### All Services (Docker Compose)
```bash
cd ../..
docker-compose up
```

## Testing

### Test Individual Service
```bash
# Health check
curl http://localhost:8081/health

# Get data
curl http://localhost:8081/oms/history/PROD-001
```

### Test Complete Flow
```bash
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

## Environment Variables

### Database Services (OMS, Inventory, Supplier, UOM)
- `PORT` - Service port number
- `DB_USER` - Database username
- `DB_PASSWORD` - Database password
- `DB_HOST` - Database host
- `DB_PORT` - Database port
- `OMS_DB` / `INVENTORY_DB` / `SUPPLIER_DB` / `UOM_DB` - Database name

### Rule Engine Service
- `PORT` - Service port number

### PO Service
- `PORT` - Service port number

### Orchestrator Service
- `PORT` - Service port number
- `OMS_SERVICE_URL` - OMS service URL
- `INVENTORY_SERVICE_URL` - Inventory service URL
- `SUPPLIER_SERVICE_URL` - Supplier service URL
- `UOM_SERVICE_URL` - UOM service URL
- `RULE_ENGINE_SERVICE_URL` - Rule engine service URL
- `PO_SERVICE_URL` - PO service URL

## Development

### Adding a New Service

1. Create service directory: `mkdir -p services/new-service/src`
2. Create Cargo.toml with purchasing-models dependency
3. Implement REST API using Axum
4. Create Dockerfile
5. Add to docker-compose.yml
6. Create Kubernetes manifests
7. Update orchestrator if needed

### Modifying Shared Models

1. Edit `shared/models/src/lib.rs`
2. Rebuild all services that depend on it
3. Test API compatibility

## Monitoring

Each service exposes:
- Health check endpoint: `GET /health`
- Structured JSON logging via tracing
- HTTP request tracing via tower-http

Logs can be viewed with:
```bash
# Docker Compose
docker-compose logs -f service-name

# Kubernetes
kubectl logs -n purchasing-flow -l app=service-name -f
```

## Performance Considerations

- **Connection Pooling:** All database services use SQLx connection pools
- **Async Operations:** All I/O operations are async using Tokio
- **Horizontal Scaling:** All services are stateless and can be scaled horizontally
- **Resource Limits:** Kubernetes deployments include resource requests/limits

## Security

- Database credentials stored in Kubernetes Secrets
- Service-to-service communication within cluster network
- No sensitive data in logs
- Input validation on all endpoints

## Future Enhancements

- [ ] Add authentication/authorization (JWT)
- [ ] Implement circuit breakers (resilience4j pattern)
- [ ] Add caching layer (Redis)
- [ ] Implement event-driven architecture (Kafka/NATS)
- [ ] Add GraphQL API gateway option
- [ ] Implement distributed tracing (OpenTelemetry)
- [ ] Add rate limiting
- [ ] Implement API versioning
- [ ] Add gRPC support for inter-service communication

## License

MIT - See main project LICENSE file
