# Purchasing Flow Example - Quick Start

> ⚠️ **OUTDATED DOCUMENTATION - v1.0**
>
> This document references the old monolithic structure and removed files.
>
> **For current quick start, see [Main README](../README.md)**
>
> **Current commands:**
> ```bash
> # Monolithic
> cd case_study
> ./scripts/setup_databases.sh
> ./scripts/run_monolithic.sh
>
> # Microservices
> cd case_study/microservices
> docker-compose up -d
> ```

---

## ⚠️ Historical Content Below (For Reference Only)

## TL;DR (OLD - No Longer Works)

```bash
# 0. Configure environment (first-time only)
cd case_study
cp .env.example .env
# Edit .env with your database credentials if needed

# 1. Setup databases (one-time)
./scripts/setup_databases.sh

# 2. Run the example (OLD - no longer exists)
./scripts/run_realdb.sh  # ❌ REMOVED - Use ./scripts/run_monolithic.sh instead
```

## Environment Setup (IMPORTANT)

**Database credentials are now stored in `.env` file for security.**

### Step 1: Create .env file

```bash
cd case_study
cp .env.example .env
```

### Step 2: Edit credentials (if needed)

The `.env.example` already contains demo database credentials. For custom database:

```bash
# Edit .env with your credentials
DB_HOST=your-host
DB_PORT=your-port
DB_USER=your-user
DB_PASSWORD=your-password
```

---

**Note:** For current documentation, see [../README.md](../README.md)
