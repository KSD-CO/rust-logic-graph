# Environment Configuration Guide

## Quick Start

1. **Copy the example file:**
   ```bash
   cp .env.example .env
   ```

2. **For local development (default):**
   
   The `.env` file is already configured for local PostgreSQL:
   ```bash
   # All databases on localhost:5432
   OMS_DB_HOST="localhost"
   INVENTORY_DB_HOST="localhost"
   SUPPLIER_DB_HOST="localhost"
   UOM_DB_HOST="localhost"
   ```

3. **Setup databases:**
   ```bash
   cd ../scripts
   ./setup_multi_databases.sh
   ```

4. **Run the application:**
   ```bash
   cargo run
   ```

## Configuration Structure

The application reads database configuration from environment variables:

```
┌─────────────────────────────────┐
│        .env file                │
├─────────────────────────────────┤
│ OMS_DB_HOST=localhost           │
│ OMS_DB_PORT=5432                │
│ OMS_DB_USER=postgres            │
│ OMS_DB_PASSWORD=postgres        │
│ OMS_DB_NAME=oms_db              │
├─────────────────────────────────┤
│ INVENTORY_DB_HOST=localhost     │
│ INVENTORY_DB_PORT=5432          │
│ ... (for each database)         │
└─────────────────────────────────┘
          ↓
┌─────────────────────────────────┐
│    config.rs                    │
│    DatabaseConfig::from_env()   │
└─────────────────────────────────┘
          ↓
┌─────────────────────────────────┐
│    main.rs                      │
│    Creates PgPool per database  │
└─────────────────────────────────┘
```

## Environment Variables

### Required for Each Database

| Variable Pattern | Description | Example |
|-----------------|-------------|---------|
| `{PREFIX}_DB_HOST` | Database server hostname | `localhost`, `db.example.com` |
| `{PREFIX}_DB_PORT` | PostgreSQL port | `5432`, `5433` |
| `{PREFIX}_DB_USER` | Database username | `postgres`, `app_user` |
| `{PREFIX}_DB_PASSWORD` | Database password | `your_password` |
| `{PREFIX}_DB_NAME` | Database name | `oms_db`, `inventory_db` |

### Prefixes

- `OMS_DB_*` - Order Management System database
- `INVENTORY_DB_*` - Inventory database
- `SUPPLIER_DB_*` - Supplier database
- `UOM_DB_*` - Unit of Measurement database

## Configuration Scenarios

### 1. Local Development (Single Server)

All databases on localhost:

```bash
# .env
OMS_DB_HOST="localhost"
OMS_DB_PORT="5432"
OMS_DB_USER="postgres"
OMS_DB_PASSWORD="postgres"
OMS_DB_NAME="oms_db"

INVENTORY_DB_HOST="localhost"
INVENTORY_DB_PORT="5432"
INVENTORY_DB_USER="postgres"
INVENTORY_DB_PASSWORD="postgres"
INVENTORY_DB_NAME="inventory_db"

# ... repeat for SUPPLIER_DB and UOM_DB
```

### 2. Multi-Container Docker

Each database in separate container:

```bash
# .env
OMS_DB_HOST="oms-postgres"          # Docker service name
OMS_DB_PORT="5432"
OMS_DB_USER="postgres"
OMS_DB_PASSWORD="postgres"
OMS_DB_NAME="oms_db"

INVENTORY_DB_HOST="inventory-postgres"
INVENTORY_DB_PORT="5432"
# ...
```

### 3. Different Ports (Same Server)

Multiple PostgreSQL instances on one server:

```bash
# .env
OMS_DB_HOST="localhost"
OMS_DB_PORT="5432"                  # First instance
# ...

INVENTORY_DB_HOST="localhost"
INVENTORY_DB_PORT="5433"            # Second instance
# ...

SUPPLIER_DB_HOST="localhost"
SUPPLIER_DB_PORT="5434"             # Third instance
# ...
```

### 4. Multi-Cloud Production

Databases across different cloud providers:

```bash
# .env
# OMS on AWS RDS
OMS_DB_HOST="oms-prod.c9akciq32.us-west-2.rds.amazonaws.com"
OMS_DB_PORT="5432"
OMS_DB_USER="oms_admin"
OMS_DB_PASSWORD="${AWS_DB_PASSWORD}"    # From secrets manager
OMS_DB_NAME="oms_production"

# Inventory on GCP Cloud SQL
INVENTORY_DB_HOST="34.82.145.92"
INVENTORY_DB_PORT="5432"
INVENTORY_DB_USER="inventory_service"
INVENTORY_DB_PASSWORD="${GCP_DB_PASSWORD}"
INVENTORY_DB_NAME="inventory_prod"

# Supplier on Azure
SUPPLIER_DB_HOST="supplier-db.postgres.database.azure.com"
SUPPLIER_DB_PORT="5432"
SUPPLIER_DB_USER="supplier_admin@supplier-db"
SUPPLIER_DB_PASSWORD="${AZURE_DB_PASSWORD}"
SUPPLIER_DB_NAME="suppliers"
```

## Testing Configuration

### Verify Environment Variables

```bash
# Check if .env is loaded
cd monolithic
cargo run 2>&1 | grep "Configuration loaded"
```

Should show output like:
```
✅ Configuration loaded
   OMS: localhost:5432/oms_db
   Inventory: localhost:5432/inventory_db
   Supplier: localhost:5432/supplier_db
   UOM: localhost:5432/uom_db
```

### Test Database Connections

```bash
# Test each database individually
psql -h localhost -p 5432 -U postgres -d oms_db -c "SELECT 1;"
psql -h localhost -p 5432 -U postgres -d inventory_db -c "SELECT 1;"
psql -h localhost -p 5432 -U postgres -d supplier_db -c "SELECT 1;"
psql -h localhost -p 5432 -U postgres -d uom_db -c "SELECT 1;"
```

## Troubleshooting

### Issue: "Connection refused"

Check if database server is running:
```bash
pg_isready -h localhost -p 5432
```

### Issue: "Authentication failed"

Verify credentials in `.env` match database:
```bash
# Test connection manually
psql "postgres://postgres:postgres@localhost:5432/oms_db"
```

### Issue: "Database does not exist"

Create databases:
```bash
cd ../scripts
./setup_multi_databases.sh
```

### Issue: Environment variables not loaded

Make sure `.env` file exists in the monolithic directory:
```bash
ls -la .env
```

If missing:
```bash
cp .env.example .env
```

## Security Best Practices

### 1. Never Commit `.env` to Git

Already in `.gitignore`:
```
.env
```

### 2. Use Strong Passwords

```bash
# Generate random password
openssl rand -base64 32
```

### 3. Use Read-Only Users

```bash
# Create read-only user in PostgreSQL
CREATE USER readonly_user WITH PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE oms_db TO readonly_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly_user;
```

### 4. Production Secrets

Use external secrets management:

**AWS:**
```bash
OMS_DB_PASSWORD="$(aws secretsmanager get-secret-value \
  --secret-id oms-db-password \
  --query SecretString \
  --output text)"
```

**HashiCorp Vault:**
```bash
OMS_DB_PASSWORD="$(vault kv get -field=password secret/database/oms)"
```

## See Also

- [MULTI_DATABASE_ARCHITECTURE.md](../MULTI_DATABASE_ARCHITECTURE.md) - Multi-database pattern and multi-server setup
- [.env.example](.env.example) - Complete configuration template
- [../scripts/setup_multi_databases.sh](../scripts/setup_multi_databases.sh) - Database setup script
