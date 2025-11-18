# Protocol Buffers Definitions

> **gRPC service definitions for microservices**

## 📁 Proto Files

| File | Service | Description |
|------|---------|-------------|
| [oms.proto](oms.proto) | OmsService | Order Management System data |
| [inventory.proto](inventory.proto) | InventoryService | Inventory levels |
| [supplier.proto](supplier.proto) | SupplierService | Supplier information |
| [uom.proto](uom.proto) | UomService | Unit of Measurement conversions |
| [rule_engine.proto](rule_engine.proto) | RuleEngineService | Business rules evaluation |
| [po.proto](po.proto) | PoService | Purchase Order operations |

## 🔧 Usage

### 1. Generate Rust Code

Proto files are automatically compiled via `build.rs`:

```rust
// build.rs in each service
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../proto/SERVICE_NAME.proto")?;
    Ok(())
}
```

### 2. Include in Code

```rust
// In your service
pub mod oms {
    tonic::include_proto!("oms");
}

use oms::oms_service_server::{OmsService, OmsServiceServer};
```

### 3. Implement Service

```rust
#[tonic::async_trait]
impl OmsService for OmsServiceImpl {
    async fn get_history(
        &self,
        request: Request<HistoryRequest>,
    ) -> Result<Response<HistoryResponse>, Status> {
        // Implementation...
    }
}
```

## 📊 Service APIs

### OMS Service (Port 50051)

```protobuf
service OmsService {
  rpc GetHistory(HistoryRequest) returns (HistoryResponse);
  rpc HealthCheck(HealthRequest) returns (HealthResponse);
}
```

**Request:**
```json
{
  "product_id": "PROD-001"
}
```

**Response:**
```json
{
  "product_id": "PROD-001",
  "avg_daily_demand": 50.0,
  "trend": "stable"
}
```

### Inventory Service (Port 50052)

```protobuf
service InventoryService {
  rpc GetLevels(LevelsRequest) returns (LevelsResponse);
  rpc HealthCheck(HealthRequest) returns (HealthResponse);
}
```

### Supplier Service (Port 50053)

```protobuf
service SupplierService {
  rpc GetInfo(InfoRequest) returns (InfoResponse);
  rpc HealthCheck(HealthRequest) returns (HealthResponse);
}
```

### UOM Service (Port 50054)

```protobuf
service UomService {
  rpc GetConversion(ConversionRequest) returns (ConversionResponse);
  rpc HealthCheck(HealthRequest) returns (HealthResponse);
}
```

### Rule Engine Service (Port 50055)

```protobuf
service RuleEngineService {
  rpc Evaluate(EvaluateRequest) returns (EvaluateResponse);
  rpc HealthCheck(HealthRequest) returns (HealthResponse);
}
```

### PO Service (Port 50056)

```protobuf
service PoService {
  rpc Create(CreateRequest) returns (CreateResponse);
  rpc Send(SendRequest) returns (SendResponse);
  rpc HealthCheck(HealthRequest) returns (HealthResponse);
}
```

## 🧪 Testing with grpcurl

### Install grpcurl

```bash
# macOS
brew install grpcurl

# Linux
apt install grpcurl

# Or download from https://github.com/fullstorydev/grpcurl
```

### Test OMS Service

```bash
# Get history
grpcurl -plaintext -d '{"product_id": "PROD-001"}' \
  localhost:50051 oms.OmsService/GetHistory

# Health check
grpcurl -plaintext \
  localhost:50051 oms.OmsService/HealthCheck
```

### Test Inventory Service

```bash
grpcurl -plaintext -d '{"product_id": "PROD-001"}' \
  localhost:50052 inventory.InventoryService/GetLevels
```

### Test Rule Engine Service

```bash
grpcurl -plaintext -d '{
  "oms_data": {
    "product_id": "PROD-001",
    "avg_daily_demand": 50.0,
    "trend": "stable"
  },
  "inventory_data": {
    "product_id": "PROD-001",
    "available_qty": 100,
    "reserved_qty": 20,
    "on_order_qty": 0
  },
  "supplier_data": {
    "product_id": "PROD-001",
    "moq": 100,
    "lead_time_days": 7,
    "unit_price": 15.99
  }
}' localhost:50055 rule_engine.RuleEngineService/Evaluate
```

## 📋 Common Patterns

### Health Check (All Services)

Every service implements health check:

```protobuf
message HealthRequest {}

message HealthResponse {
  string status = 1;
  string service = 2;
}
```

Usage:
```bash
grpcurl -plaintext localhost:PORT SERVICE.ServiceName/HealthCheck
```

### Error Handling

Use standard gRPC status codes:

```rust
use tonic::{Code, Status};

// Not found
return Err(Status::new(Code::NotFound, "Product not found"));

// Invalid argument
return Err(Status::new(Code::InvalidArgument, "Invalid product ID"));

// Internal error
return Err(Status::new(Code::Internal, "Database error"));
```

## 🔄 Updating Proto Files

1. **Edit proto file** (e.g., `oms.proto`)
2. **Rebuild service:** `cargo build`
3. **Auto-generates** new Rust code
4. **Update implementation** if needed

## 📚 Best Practices

### 1. Versioning

When making breaking changes:

```protobuf
syntax = "proto3";

package oms.v2;  // New version

service OmsServiceV2 {
  // New API
}
```

### 2. Field Numbers

Never reuse field numbers:

```protobuf
message HistoryResponse {
  string product_id = 1;
  double avg_daily_demand = 2;
  string trend = 3;
  // reserved 4;  // If field removed, reserve the number
  string new_field = 5;  // Use next available number
}
```

### 3. Backwards Compatibility

- Don't change field numbers
- Don't change field types
- Mark fields as optional if they might not exist
- Use reserved for removed fields

## 🔗 Resources

- **Implementation Guide:** [../GRPC_IMPLEMENTATION.md](../GRPC_IMPLEMENTATION.md)
- **Tonic Docs:** https://docs.rs/tonic/
- **Protocol Buffers:** https://developers.google.com/protocol-buffers
- **gRPC Best Practices:** https://grpc.io/docs/guides/performance/

---

**Generated Code:** Auto-generated via `tonic-build`
**Language:** Rust (via Protocol Buffers)
**Transport:** HTTP/2
