# Multi-Database Architecture

> **Distributed data with single process - Best of both worlds: Monolithic simplicity + Microservices data isolation**

## 🎯 Overview

The monolithic version uses a **multi-database architecture** where each domain has its own PostgreSQL database, similar to microservices, but runs in a single process.

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Single Process (Monolithic)               │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │      PurchasingGraphExecutor                       │    │
│  │                                                     │    │
│  │  pools: HashMap<String, DatabasePool>              │    │
│  │  ├─ "oms_db" ────────────┐                        │    │
│  │  ├─ "inventory_db" ───────┼─────┐                 │    │
│  │  ├─ "supplier_db" ────────┼─────┼─────┐           │    │
│  │  └─ "uom_db" ─────────────┼─────┼─────┼───┐       │    │
│  └─────────────────────────────────────────────────┘  │    │
│                           │     │     │     │          │    │
└───────────────────────────┼─────┼─────┼─────┼──────────┘    │
                            │     │     │     │                │
                ┌───────────┼─────┼─────┼─────┼───────────┐   │
                │ PostgreSQL Connection Pools (sqlx)      │   │
                └───────────┼─────┼─────┼─────┼───────────┘   │
                            ▼     ▼     ▼     ▼                │
                  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐        │
                  │oms_db│ │inv_db│ │sup_db│ │uom_db│        │
                  │      │ │      │ │      │ │      │        │
                  │oms_  │ │inven │ │suppl │ │uom_  │        │
                  │history│ │tory │ │iers │ │conv  │        │
                  └──────┘ └──────┘ └──────┘ └──────┘        │
```

## 🗄️ Database Separation

### Four Independent Databases

| Database | Table | Purpose | Sample Queries |
|----------|-------|---------|----------------|
| **oms_db** | `oms_history` | Order Management System | Historical demand, trends |
| **inventory_db** | `inventory` | Inventory Management | Stock levels, warehouse locations |
| **supplier_db** | `suppliers` | Supplier Management | Pricing, MOQ, lead times |
| **uom_db** | `uom_conversions` | Unit Conversions | Unit factors, conversions |

### Sample Data

```sql
-- oms_db.oms_history
| product_id | avg_daily_demand | trend      |
|------------|------------------|------------|
| PROD-001   | 150.00           | increasing |
| PROD-002   | 80.50            | stable     |
| PROD-003   | 200.00           | decreasing |

-- inventory_db.inventory
| product_id | available_qty | reserved_qty | warehouse_location |
|------------|---------------|--------------|-------------------|
| PROD-001   | 500.00        | 100.00       | WH-A-01           |
| PROD-002   | 300.00        | 50.00        | WH-B-02           |
| PROD-003   | 150.00        | 0.00         | WH-C-03           |

-- supplier_db.suppliers
| product_id | supplier_name        | unit_price | moq    | lead_time |
|------------|---------------------|------------|--------|-----------|
| PROD-001   | ABC Supplies Co.    | 15.50      | 100.00 | 7         |
| PROD-002   | XYZ Manufacturing   | 25.00      | 50.00  | 14        |
| PROD-003   | Global Parts Ltd.   | 42.75      | 200.00 | 21        |

-- uom_db.uom_conversions
| product_id | from_uom | to_uom | conversion_factor |
|------------|----------|--------|------------------|
| PROD-001   | pieces   | box    | 12.0000          |
| PROD-002   | kg       | lb     | 2.2046           |
| PROD-003   | liter    | gallon | 0.2642           |
```

## 📋 YAML Configuration

### Database Routing in YAML

Each node specifies which database to query:

```yaml
# purchasing_flow_graph.yaml
nodes:
  oms_history:
    type: DBNode
    database: "oms_db"  # ← Database specification
    query: "SELECT product_id, avg_daily_demand, trend FROM oms_history WHERE product_id = $1"
    description: "Fetch order management system history data"
  
  inventory_levels:
    type: DBNode
    database: "inventory_db"  # ← Different database
    query: "SELECT product_id, available_qty, reserved_qty, warehouse_location FROM inventory WHERE product_id = $1"
    description: "Fetch current inventory levels"
  
  supplier_info:
    type: DBNode
    database: "supplier_db"  # ← Another database
    query: "SELECT product_id, supplier_name, unit_price, moq, lead_time FROM suppliers WHERE product_id = $1"
    description: "Fetch supplier information and pricing"
  
  uom_conversion:
    type: DBNode
    database: "uom_db"  # ← Fourth database
    query: "SELECT product_id, from_uom, to_uom, conversion_factor FROM uom_conversions WHERE product_id = $1"
    description: "Fetch unit of measurement conversions"
```

## 💻 Implementation

### 1. Multi-Pool Architecture

```rust
// graph_executor.rs
pub struct PurchasingGraphExecutor {
    pools: HashMap<String, DatabasePool>,  // Map: db_name → pool
    default_pool: DatabasePool,
}

impl PurchasingGraphExecutor {
    pub fn add_pool(&mut self, db_name: String, pool: DatabasePool) {
        self.pools.insert(db_name, pool);
    }
    
    fn get_pool(&self, db_name: Option<&str>) -> DatabasePool {
        db_name
            .and_then(|name| self.pools.get(name))
            .cloned()
            .unwrap_or_else(|| self.default_pool.clone())
    }
}
```

### 2. Database Configuration with Per-DB Settings

```rust
// config.rs
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DatabaseConfig {
    /// Create config from environment variables with prefix
    /// Example: prefix="OMS_DB" reads OMS_DB_HOST, OMS_DB_PORT, etc.
    pub fn from_env_prefix(prefix: &str) -> Self {
        Self {
            host: env::var(&format!("{}_HOST", prefix)).unwrap_or_else(|_| "localhost".to_string()),
            port: env::var(&format!("{}_PORT", prefix)).unwrap_or_else(|_| "5432".to_string()).parse().unwrap_or(5432),
            user: env::var(&format!("{}_USER", prefix)).unwrap_or_else(|_| "postgres".to_string()),
            password: env::var(&format!("{}_PASSWORD", prefix)).unwrap_or_else(|_| "".to_string()),
            database: env::var(&format!("{}_NAME", prefix)).unwrap_or_else(|_| "postgres".to_string()),
        }
    }
    
    pub fn connection_string(&self) -> String {
        format!("postgres://{}:{}@{}:{}/{}", self.user, self.password, self.host, self.port, self.database)
    }
}

/// Application configuration with multiple database configs
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub oms_db: DatabaseConfig,
    pub inventory_db: DatabaseConfig,
    pub supplier_db: DatabaseConfig,
    pub uom_db: DatabaseConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            oms_db: DatabaseConfig::from_env_prefix("OMS_DB"),
            inventory_db: DatabaseConfig::from_env_prefix("INVENTORY_DB"),
            supplier_db: DatabaseConfig::from_env_prefix("SUPPLIER_DB"),
            uom_db: DatabaseConfig::from_env_prefix("UOM_DB"),
        }
    }
}
```

### 3. Dynamic Database Routing

```rust
// graph_executor.rs - execute_with_config()
for (node_id, node_config) in &graph_def.nodes {
    let node: Box<dyn Node> = match node_config.node_type {
        NodeType::DBNode => {
            let query = node_config.query.clone()
                .ok_or_else(|| anyhow!("DBNode '{}' missing query", node_id))?;
            
            let node_cfg = graph_config.nodes.get(node_id);
            
            // Check if node has custom connection string
            let pool = if let Some(conn_str) = node_cfg.and_then(|nc| nc.connection.as_deref()) {
                // Create a new pool for this specific connection
                tracing::info!("📡 Node '{}' using custom connection", node_id);
                let pg_pool = utils::database::create_postgres_pool(conn_str).await?;
                DatabasePool::from_postgres(pg_pool)
            } else {
                // Use database name routing (main method)
                let db_name = node_cfg.and_then(|nc| nc.database.as_deref());
                let pool = self.get_pool(db_name);
                
                if let Some(db) = db_name {
                    tracing::info!("📦 Node '{}' will use database: {}", node_id, db);
                }
                
                pool
            };
            
            Box::new(DynamicDBNode::new(node_id.clone(), query, pool, product_id.to_string()))
        }
        // ...
    };
}
```

### 4. Main.rs Setup

```rust
// main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::from_path(".env")?;
    
    let config = AppConfig::default();
    
    tracing::info!("✅ Configuration loaded");
    tracing::info!("   OMS: {}:{}/{}", config.oms_db.host, config.oms_db.port, config.oms_db.database);
    tracing::info!("   Inventory: {}:{}/{}", config.inventory_db.host, config.inventory_db.port, config.inventory_db.database);
    tracing::info!("   Supplier: {}:{}/{}", config.supplier_db.host, config.supplier_db.port, config.supplier_db.database);
    tracing::info!("   UOM: {}:{}/{}", config.uom_db.host, config.uom_db.port, config.uom_db.database);
    
    // Create connection pool for each database
    let oms_pool = utils::database::create_postgres_pool(&config.oms_db.connection_string()).await?;
    tracing::info!("✅ OMS database pool created: {}", config.oms_db.database);
    
    let inventory_pool = utils::database::create_postgres_pool(&config.inventory_db.connection_string()).await?;
    tracing::info!("✅ Inventory database pool created: {}", config.inventory_db.database);
    
    let supplier_pool = utils::database::create_postgres_pool(&config.supplier_db.connection_string()).await?;
    tracing::info!("✅ Supplier database pool created: {}", config.supplier_db.database);
    
    let uom_pool = utils::database::create_postgres_pool(&config.uom_db.connection_string()).await?;
    tracing::info!("✅ UOM database pool created: {}", config.uom_db.database);
    
    // Create executor with multi-pool support
    let mut executor = PurchasingGraphExecutor::from_postgres(oms_pool.clone());
    executor.add_pool("oms_db".to_string(), DatabasePool::from_postgres(oms_pool));
    executor.add_pool("inventory_db".to_string(), DatabasePool::from_postgres(inventory_pool));
    executor.add_pool("supplier_db".to_string(), DatabasePool::from_postgres(supplier_pool));
    executor.add_pool("uom_db".to_string(), DatabasePool::from_postgres(uom_pool));
    
    // ... rest of setup
}
```

## 🚀 Setup & Usage

### 1. Database Setup

```bash
# Run the multi-database setup script
cd case_study/scripts
./setup_multi_databases.sh
```

This creates:
- `oms_db` with `oms_history` table
- `inventory_db` with `inventory` table
- `supplier_db` with `suppliers` table
- `uom_db` with `uom_conversions` table

### 2. Configuration

```bash
# .env file in monolithic/
# Default database connection (fallback)
DB_HOST="localhost"
DB_PORT="5432"
DB_USER="postgres"
DB_PASSWORD=""

# OMS Database Server
OMS_DB_HOST="localhost"
OMS_DB_PORT="5432"
OMS_DB_USER="postgres"
OMS_DB_PASSWORD=""
OMS_DB_NAME="oms_db"

# Inventory Database Server
INVENTORY_DB_HOST="localhost"
INVENTORY_DB_PORT="5432"
INVENTORY_DB_USER="postgres"
INVENTORY_DB_PASSWORD=""
INVENTORY_DB_NAME="inventory_db"

# Supplier Database Server
SUPPLIER_DB_HOST="localhost"
SUPPLIER_DB_PORT="5432"
SUPPLIER_DB_USER="postgres"
SUPPLIER_DB_PASSWORD=""
SUPPLIER_DB_NAME="supplier_db"

# UOM Database Server
UOM_DB_HOST="localhost"
UOM_DB_PORT="5432"
UOM_DB_USER="postgres"
UOM_DB_PASSWORD=""
UOM_DB_NAME="uom_db"
```

**Note:** Each database can be on a different server! Just change the HOST/PORT per database.

### Multi-Server Setup Example

For true distributed architecture across different servers/clouds:

```bash
# .env - Multi-Cloud Configuration
# OMS on AWS RDS
OMS_DB_HOST="oms-prod.c9akciq32.us-west-2.rds.amazonaws.com"
OMS_DB_PORT="5432"
OMS_DB_USER="oms_admin"
OMS_DB_PASSWORD="${AWS_DB_PASSWORD}"
OMS_DB_NAME="oms_production"

# Inventory on Google Cloud SQL
INVENTORY_DB_HOST="34.82.145.92"
INVENTORY_DB_PORT="5432"
INVENTORY_DB_USER="inventory_service"
INVENTORY_DB_PASSWORD="${GCP_DB_PASSWORD}"
INVENTORY_DB_NAME="inventory_prod"

# Supplier on Azure Database
SUPPLIER_DB_HOST="supplier-db.postgres.database.azure.com"
SUPPLIER_DB_PORT="5432"
SUPPLIER_DB_USER="supplier_admin@supplier-db"
SUPPLIER_DB_PASSWORD="${AZURE_DB_PASSWORD}"
SUPPLIER_DB_NAME="suppliers"

# UOM on local datacenter
UOM_DB_HOST="10.0.1.50"
UOM_DB_PORT="5432"
UOM_DB_USER="uom_readonly"
UOM_DB_PASSWORD="${LOCAL_DB_PASSWORD}"
UOM_DB_NAME="reference_data"
```

Architecture becomes:
```
┌─────────────────────────────────────┐
│    Monolithic Application           │
│    (Single Process)                 │
└─────┬────┬────┬────┬────────────────┘
      │    │    │    │
      ▼    ▼    ▼    ▼
   ┌────┐┌────┐┌────┐┌────┐
   │AWS ││GCP ││Azure││Local│
   │RDS ││SQL ││DB  ││DC  │
   └────┘└────┘└────┘└────┘
```

### 3. Run

```bash
cd case_study/monolithic

# Development (with debug logs)
RUST_LOG=info cargo run --bin purchasing_flow

# Production (optimized, faster startup)
cargo build --release
RUST_LOG=info ./target/release/purchasing_flow
```

**Performance Tip:** Release build reduces GRL parsing from ~2.7s → ~0.3s!

### 4. Test

```bash
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

## ✅ Benefits

### 1. Data Isolation
- ✅ Each domain has separate database
- ✅ No cross-domain data leakage
- ✅ Clear boundaries between contexts

### 2. Development Benefits
- ✅ Single process - easy debugging
- ✅ No network calls - fast execution
- ✅ Simple deployment - one binary
- ✅ Full stack traces across all databases

### 3. Microservices Preparation
- ✅ Database schema already separated
- ✅ Easy migration path to microservices
- ✅ Can test data isolation early
- ✅ Same database structure as microservices

### 4. Operational Benefits
- ✅ Independent database backups
- ✅ Per-database optimization (indexes, caching)
- ✅ Can scale databases independently
- ✅ Clear ownership of data

### 5. Multi-Server Flexibility
- ✅ **Geographic Distribution**: Place databases near data sources
- ✅ **Cloud Provider Flexibility**: Use best service from each provider (AWS RDS, GCP SQL, Azure DB)
- ✅ **Security Isolation**: Different credentials per database, network segmentation
- ✅ **Performance Optimization**: Dedicated resources per domain, independent scaling
- ✅ **Hybrid Cloud**: Mix cloud and on-premise databases
- ✅ **Data Residency**: Comply with regional data requirements

## 🔄 Comparison: Single DB vs Multi-DB

| Aspect | Single Database | Multi-Database (This Approach) |
|--------|----------------|-------------------------------|
| **Data Isolation** | ❌ Shared tables | ✅ Separate databases |
| **Schema Changes** | ⚠️ Affects all | ✅ Isolated to domain |
| **Database Optimization** | ❌ One-size-fits-all | ✅ Per-database tuning |
| **Backup Strategy** | ❌ All-or-nothing | ✅ Per-domain backups |
| **Migration to Microservices** | ⚠️ Complex data split | ✅ Already separated |
| **Development** | ✅ Simpler | ⚠️ More setup |
| **Deployment** | ✅ Single process | ✅ Single process |

## 🎯 Use Cases

### When to Use Multi-Database Monolith

✅ **Best For:**
- Learning distributed systems concepts
- Prototyping microservices architecture
- Development/staging environments
- Projects planning to scale to microservices
- Need data isolation without network overhead

❌ **Not Ideal For:**
- Very simple applications (single DB is fine)
- Legacy migrations (gradual approach better)
- Teams unfamiliar with distributed data

## 📊 Performance Characteristics

### Connection Pooling
- Each database has min 5, max 20 connections
- Pools created once at startup
- Reused across requests

### Query Execution
- Parallel execution via async/await
- No network serialization overhead
- Direct database driver calls (sqlx)

### Startup Time
- Debug build: ~2-3 seconds (4 pools + GRL parsing ~2.7s)
- Release build: ~1 second (4 pools + GRL parsing ~0.3s)
- vs ~10 seconds for microservices (7 services)

### Request Latency
- First request (debug): ~2.7s (GRL parsing)
- First request (release): ~330ms (GRL parsing)
- Subsequent requests: ~3-10ms (cached GRL engine)
- vs ~50-100ms for microservices (network + gRPC)

### GRL Rule Engine Performance
- **Debug build**: 2.715s parse time (first request)
- **Release build**: 0.321s parse time (first request)
- **Cached**: 0.020ms (subsequent requests with OnceLock)
- **Improvement**: 100x faster after caching, 8x faster with release build

## 🔍 Troubleshooting

### Connection Issues

```bash
# Test database connectivity (same server)
psql -h localhost -p 5432 -U postgres -d oms_db -c "SELECT 1;"
psql -h localhost -p 5432 -U postgres -d inventory_db -c "SELECT 1;"
psql -h localhost -p 5432 -U postgres -d supplier_db -c "SELECT 1;"
psql -h localhost -p 5432 -U postgres -d uom_db -c "SELECT 1;"

# Test multi-server connectivity
pg_isready -h oms-server.company.com -p 5432
pg_isready -h inventory-cluster.aws.com -p 5432
psql "postgres://user:pass@oms-server:5432/oms_db" -c "SELECT 1;"
```

### Check Database Existence

```bash
# Check all databases exist
psql -h localhost -p 5432 -U postgres -l | grep -E "(oms_db|inventory_db|supplier_db|uom_db)"

# Or with custom user
psql -h localhost -p 5432 -U jamesvu -l | grep -E "(oms_db|inventory_db|supplier_db|uom_db)"
```

### Verify Data

```bash
# Check each database has data
PGPASSWORD=postgres psql -h localhost -p 5432 -U postgres -d oms_db \
  -c "SELECT COUNT(*) FROM oms_history;"

PGPASSWORD=postgres psql -h localhost -p 5432 -U postgres -d inventory_db \
  -c "SELECT COUNT(*) FROM inventory;"
```

### Performance Issues

**Problem: First request takes 2-3 seconds**
- **Cause**: GRL parsing in debug build
- **Solution**: Use release build
  ```bash
  cargo build --release
  RUST_LOG=info ./target/release/purchasing_flow
  ```
- **Result**: First request ~330ms (8x faster)

**Problem: Every request is slow**
- **Cause**: RuleEngineService not cached
- **Solution**: Already implemented with OnceLock in production code
- **Verify**: Check logs for "⚡ [Cache] Using cached RuleEngineService"
- **Result**: Subsequent requests ~3-10ms (100x faster)

**Problem: Connection timeout to remote servers**
- **Cause**: Firewall rules, network issues
- **Solution**: 
  ```bash
  # Check network connectivity
  ping oms-server.company.com
  telnet inventory-cluster.aws.com 5432
  # Check firewall rules allow your IP
  ```
- **Common issues**: Security groups (AWS), firewall rules (GCP/Azure), VPN required

**Problem: Authentication failed on cloud databases**
- **Cause**: Wrong credentials format
- **Azure**: User format must be `username@servername`
- **AWS RDS**: Check security group allows your IP
- **GCP SQL**: May need to whitelist IP or use Cloud SQL Proxy
- **Solution**: Verify connection string format per cloud provider

## 📚 Related Documentation

- [README.md](README.md) - Main project overview
- [monolithic/purchasing_flow_graph.yaml](monolithic/purchasing_flow_graph.yaml) - YAML config example
- [scripts/setup_multi_databases.sh](scripts/setup_multi_databases.sh) - Database setup script
- [GRPC.md](GRPC.md) - Microservices with gRPC
- [MICROSERVICES_DEPLOYMENT.md](MICROSERVICES_DEPLOYMENT.md) - K8s deployment

## 🔐 Security Best Practices

### 1. Never Hardcode Credentials
```bash
# ✅ GOOD - Use environment variables
OMS_DB_PASSWORD="${AWS_DB_PASSWORD}"
INVENTORY_DB_USER="service_account"

# ❌ BAD - Hardcoded in YAML or code
connection: "postgres://admin:password123@server:5432/db"
```

### 2. Use Read-Only Users Where Possible
```bash
# .env
SUPPLIER_DB_USER="readonly_user"
UOM_DB_USER="uom_readonly"
```

### 3. Enable SSL/TLS for Production
```bash
# Add to connection string (future enhancement)
OMS_DB_HOST="oms-server?sslmode=require"
```

### 4. Use Secrets Management
```bash
# AWS Secrets Manager example
OMS_DB_PASSWORD="$(aws secretsmanager get-secret-value --secret-id oms-db-password --query SecretString --output text)"

# GCP Secret Manager
INVENTORY_DB_PASSWORD="$(gcloud secrets versions access latest --secret=inventory-db-pass)"
```

### 5. Network Security
- Use VPN or VPC peering for cloud databases
- Whitelist application server IPs
- Use security groups/firewall rules
- Consider database proxies (Cloud SQL Proxy, RDS Proxy)

## 🎉 Summary

The multi-database monolithic architecture provides:

1. **Distributed data** like microservices
2. **Single process** simplicity of monolithic
3. **YAML-driven routing** - no hardcoded database logic
4. **Easy migration path** to true microservices
5. **Per-database configuration** - each DB can be on different server/port
6. **GRL rule engine caching** - 100x performance improvement with OnceLock
7. **Release build optimization** - 8x faster GRL parsing

**Architecture Highlights:**
- ✅ Clean separation: `nodes/` module with `db_node/` and `rule_node/`
- ✅ Models colocated with nodes for better organization
- ✅ Per-database environment variables (e.g., `OMS_DB_HOST`, `INVENTORY_DB_PORT`)
- ✅ Smart caching: RuleEngineService cached globally, parse once, reuse forever
- ✅ Performance: <10ms latency after first request (release build)

**Perfect for:** Learning, prototyping, and projects planning to scale to microservices later! 🚀
