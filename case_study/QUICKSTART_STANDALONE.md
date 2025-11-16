# Quick Start - Standalone Case Study Project

This case study is now a **complete, standalone Rust project** with its own `Cargo.toml`!

## 🚀 Quick Start

### 1. Navigate to Case Study
```bash
cd case_study
```

### 2. Setup Environment (Required for Real Database)

For the real database and advanced versions, you need to configure your database credentials:

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env with your database credentials
vim .env  # or use your preferred editor
```

The `.env` file contains:
```bash
# Database Connection Settings
DB_USER=lune_dev
DB_PASSWORD=rfSxLLeSqVCGNeGc
DB_HOST=171.244.10.40
DB_PORT=6033

# Database Names
OMS_DB=oms_db
INVENTORY_DB=inventory_db
SUPPLIER_DB=supplier_db
UOM_DB=uom_db
```

**Note:** The `.env` file is gitignored for security. Never commit database credentials to version control.

### 3. Run Examples

#### Mock Version (No Database Required)
```bash
cargo run --bin purchasing_flow_mock

# OR use helper script
./scripts/run_mock.sh
```

#### Real Database Version
```bash
# First-time: Setup environment and databases
cp .env.example .env   # Configure your credentials
./scripts/setup_databases.sh

# Run with real MySQL databases
cargo run --bin purchasing_flow_realdb --features mysql

# OR use helper script (checks for .env automatically)
./scripts/run_realdb.sh
```

#### Advanced Version (with Monitoring)
```bash
# Requires .env file and databases to be set up
cp .env.example .env   # If not already done
./scripts/setup_databases.sh  # If not already done

cargo run --bin purchasing_flow_advanced --features mysql

# OR use helper script (checks for .env automatically)
./scripts/run_advanced.sh
```

## 📦 Build Commands

```bash
# Build all binaries
cargo build --all

# Build with MySQL support
cargo build --features mysql

# Build specific binary
cargo build --bin purchasing_flow_realdb --features mysql

# Release build (optimized)
cargo build --release --features mysql
```

## 🗂️ Project Structure

```
case_study/                    # Standalone Rust project
├── Cargo.toml                # Project configuration
├── Cargo.lock                # Dependency lock file
├── .env.example              # Environment template
├── .env                      # Your credentials (gitignored)
│
├── src/                      # Source code (3 binaries)
│   ├── purchasing_flow_mock.rs        # Mock version
│   ├── purchasing_flow_realdb.rs      # Real DB version
│   └── purchasing_flow_advanced.rs    # Advanced monitoring
│
├── scripts/                  # Helper scripts
│   ├── run_mock.sh          # Run mock version
│   ├── run_realdb.sh        # Run real DB version
│   ├── run_advanced.sh      # Run advanced version
│   ├── setup_databases.sh   # Setup MySQL databases
│   └── test_purchasing_flow.sh # Test script
│
├── sql/                      # Database setup
│   └── purchasing_flow_setup.sql
│
├── docs/                     # Documentation
│   └── ... (7 comprehensive docs)
│
└── target/                   # Build output (gitignored)
```

## 🎯 Available Binaries

| Binary | Description | Database | Command |
|--------|-------------|----------|---------|
| `purchasing_flow_mock` | Mock version | None | `cargo run --bin purchasing_flow_mock` |
| `purchasing_flow_realdb` | Real DB version | MySQL | `cargo run --bin purchasing_flow_realdb --features mysql` |
| `purchasing_flow_advanced` | With monitoring | MySQL | `cargo run --bin purchasing_flow_advanced --features mysql` |

## 🔧 Development

### Run Tests
```bash
cargo test
```

### Check Code
```bash
cargo check --all --features mysql
```

### Format Code
```bash
cargo fmt
```

### Clean Build
```bash
cargo clean
cargo build --features mysql
```

## 📊 Features

- `mysql` - Enable MySQL database support (required for realdb and advanced)
- `all` - Enable all features

## 🌟 Key Advantages

✅ **Self-Contained** - Complete Rust project, no parent dependencies
✅ **Independent Build** - Own `Cargo.toml` and build system
✅ **Easy to Run** - Simple `cargo run` commands
✅ **Helper Scripts** - Convenient shell scripts for common tasks
✅ **Professional Structure** - Production-ready organization
✅ **Ready to Share** - Can be extracted and shared independently

## 📝 Notes

- This project depends on `rust-logic-graph` from the parent directory via `path = ".."`
- For production deployment, change the dependency to use crates.io version
- **Database credentials are now stored in `.env` file** (not in source code)
- The `.env` file is gitignored and should never be committed to version control
- See `docs/` folder for comprehensive documentation

## 🔐 Security Best Practices

1. **Never commit `.env` file** - It's already in `.gitignore`
2. **Use different credentials for production** - The example credentials are for demo only
3. **Rotate credentials regularly** - Update `.env` file as needed
4. **Limit database permissions** - Use read-only accounts where possible

## 🚀 Next Steps

1. **Setup Environment**: `cp .env.example .env` and configure your credentials
2. **Learn**: Read `docs/QUICKSTART.md`
3. **Explore**: Run `./scripts/run_mock.sh` (no database needed)
4. **Setup DB**: Run `./scripts/setup_databases.sh` (creates tables and data)
5. **Test Real DB**: Run `./scripts/run_realdb.sh` (uses your .env config)
6. **Study**: Read `docs/CASE_STUDY.md` for deep dive

---

**This is now a complete, standalone Rust project!** 🎉
