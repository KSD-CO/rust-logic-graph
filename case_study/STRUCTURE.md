# Case Study Directory Structure

## Overview

This directory contains the complete **Purchasing Flow Production Case Study** with organized structure for easy navigation.

## Directory Tree

```
case_study/
│
├── README.md                           # Main entry point (START HERE)
├── STRUCTURE.md                        # This file
├── .gitignore                         # Git ignore rules
│
├── docs/                              # Documentation (7 files)
│   ├── QUICKSTART.md                  # 5-minute quick start ⭐
│   ├── CASE_STUDY.md                  # Complete technical deep-dive (30 pages)
│   ├── purchasing_flow_README.md      # Full technical reference
│   ├── COMPARISON.md                  # Mock vs Real DB analysis
│   ├── PURCHASING_FLOW_SUMMARY.md     # Vietnamese summary
│   ├── CASE_STUDY_INDEX.md            # Navigation guide
│   └── PRESENTATION.md                # Slide-style presentation
│
├── src/                               # Source code (2 files)
│   ├── purchasing_flow_realdb.rs      # Real DB version ⭐
│   └── purchasing_flow_advanced.rs    # With monitoring & metrics
│
├── sql/                               # Database setup (1 file)
│   └── purchasing_flow_setup.sql      # Creates 4 databases + test data
│
└── scripts/                           # Helper scripts (2 files)
    ├── setup_databases.sh             # Automated database setup
    └── test_purchasing_flow.sh        # Connectivity test + run
```

## File Descriptions

### Root Files

#### README.md
- **Purpose**: Main entry point for the case study
- **Content**: Overview, quick start, navigation
- **Audience**: Everyone
- **Read First**: Yes

#### STRUCTURE.md
- **Purpose**: Directory structure documentation
- **Content**: File organization and descriptions
- **Audience**: Developers exploring the codebase
- **Read First**: Optional

### docs/ Directory

All documentation files live here. Organized by purpose:

#### QUICKSTART.md ⭐
- **Purpose**: Get running in 5 minutes
- **Content**: Quick commands, database info, troubleshooting
- **Length**: 2 pages
- **Read if**: You want to start immediately

#### CASE_STUDY.md
- **Purpose**: Complete technical analysis
- **Content**: Business context, architecture, implementation, performance
- **Length**: 30+ pages
- **Read if**: You want deep understanding

#### purchasing_flow_README.md
- **Purpose**: Technical reference documentation
- **Content**: Setup instructions, architecture, business logic
- **Length**: 15 pages
- **Read if**: You need detailed technical reference

#### COMPARISON.md
- **Purpose**: Mock vs Real DB comparison
- **Content**: Feature matrix, code differences, performance
- **Length**: 10 pages
- **Read if**: You want to understand trade-offs

#### PURCHASING_FLOW_SUMMARY.md
- **Purpose**: Vietnamese language summary
- **Content**: Tổng quan, kiến trúc, hướng dẫn (tiếng Việt)
- **Length**: 10 pages
- **Read if**: Vietnamese speaker wanting overview

#### CASE_STUDY_INDEX.md
- **Purpose**: Navigation and file index
- **Content**: File listing, learning paths, cross-references
- **Length**: 8 pages
- **Read if**: You need navigation help

#### PRESENTATION.md
- **Purpose**: Slide-style presentation
- **Content**: Overview in presentation format
- **Length**: 20+ slides
- **Read if**: Preparing presentation or demo

### src/ Directory

Source code examples:

#### purchasing_flow_realdb.rs ⭐
- **Purpose**: Real database version
- **Features**:
  - 4 MySQL database connections
  - Connection pooling with sqlx
  - Async/await queries
  - Production patterns
- **Lines**: ~400 lines
- **Complexity**: Intermediate
- **Run**: `cargo run --example purchasing_flow_realdb --features mysql`

#### purchasing_flow_advanced.rs
- **Purpose**: Advanced version with monitoring
- **Features**:
  - All features from purchasing_flow_realdb.rs
  - Performance metrics collection
  - Real-time monitoring
  - Detailed timing
  - Production logging
- **Lines**: ~450 lines
- **Complexity**: Advanced
- **Run**: `cargo run --example purchasing_flow_advanced --features mysql`

**Note**: The mock version (`purchasing_flow.rs`) remains in `../examples/` for simplicity.

### sql/ Directory

Database setup files:

#### purchasing_flow_setup.sql
- **Purpose**: Create all databases and test data
- **Creates**:
  - 4 databases (oms_db, inventory_db, supplier_db, uom_db)
  - 4 tables (one per database)
  - Test data for 3 products (PROD-001, PROD-002, PROD-003)
- **Lines**: ~110 lines
- **Run**: Via `scripts/setup_databases.sh` or manually with mysql client

### scripts/ Directory

Helper automation scripts:

#### setup_databases.sh
- **Purpose**: Automated database setup
- **Does**:
  - Checks for mysql client
  - Creates all 4 databases
  - Creates tables
  - Inserts test data
  - Verifies setup
- **Run**: `./scripts/setup_databases.sh`
- **Required**: Once before running examples

#### test_purchasing_flow.sh
- **Purpose**: Test connectivity and run example
- **Does**:
  - Builds example if needed
  - Tests database connectivity
  - Runs purchasing flow
  - Shows helpful error messages
- **Run**: `./scripts/test_purchasing_flow.sh`
- **Required**: No (but helpful)

## Navigation Guide

### I'm New Here
1. Read [`README.md`](README.md)
2. Then [`docs/QUICKSTART.md`](docs/QUICKSTART.md)
3. Run mock version: `cd .. && cargo run --example purchasing_flow`
4. Come back and setup databases

### I Want to Run Real DB Version
1. Read [`docs/QUICKSTART.md`](docs/QUICKSTART.md)
2. Run `scripts/setup_databases.sh`
3. Run `cd .. && cargo run --example purchasing_flow_realdb --features mysql`

### I Want to Understand Everything
1. Start with [`README.md`](README.md)
2. Read [`docs/CASE_STUDY.md`](docs/CASE_STUDY.md) completely
3. Study [`src/purchasing_flow_realdb.rs`](src/purchasing_flow_realdb.rs)
4. Try modifying and running

### I Want to Present This
1. Read [`docs/PRESENTATION.md`](docs/PRESENTATION.md)
2. Review [`docs/CASE_STUDY.md`](docs/CASE_STUDY.md) for details
3. Use diagrams from documentation

### I Want Vietnamese Documentation
1. Read [`docs/PURCHASING_FLOW_SUMMARY.md`](docs/PURCHASING_FLOW_SUMMARY.md)
2. Also check [`README.md`](README.md) for bilingual sections

## File Sizes

| File | Lines | Size |
|------|-------|------|
| README.md | ~500 | ~20 KB |
| docs/QUICKSTART.md | ~150 | ~6 KB |
| docs/CASE_STUDY.md | ~900 | ~50 KB |
| docs/purchasing_flow_README.md | ~450 | ~25 KB |
| docs/COMPARISON.md | ~350 | ~18 KB |
| docs/PURCHASING_FLOW_SUMMARY.md | ~300 | ~15 KB |
| docs/CASE_STUDY_INDEX.md | ~250 | ~12 KB |
| docs/PRESENTATION.md | ~600 | ~28 KB |
| src/purchasing_flow_realdb.rs | ~400 | ~15 KB |
| src/purchasing_flow_advanced.rs | ~450 | ~17 KB |
| sql/purchasing_flow_setup.sql | ~110 | ~4 KB |
| **Total** | **~4,460** | **~210 KB** |

## Quick Commands Reference

```bash
# Navigate to case study
cd case_study

# View structure
ls -R

# Read main README
cat README.md

# Setup databases
./scripts/setup_databases.sh

# Test connectivity
./scripts/test_purchasing_flow.sh

# Run examples (from project root)
cd ..
cargo run --example purchasing_flow                          # Mock
cargo run --example purchasing_flow_realdb --features mysql  # Real DB
cargo run --example purchasing_flow_advanced --features mysql # Advanced
```

## Dependencies

### Documentation Files
- All documentation is self-contained
- Cross-references use relative links
- Markdown format (GitHub-flavored)

### Source Files
- Require Rust 1.70+
- Depend on main crate: `rust-logic-graph`
- Compiled as examples from project root
- Feature flag: `--features mysql`

### Database Files
- Require MySQL 8.0+
- Connection to 171.244.10.40:6033
- Credentials in SQL file

### Scripts
- Require bash shell
- `setup_databases.sh`: Requires mysql client
- `test_purchasing_flow.sh`: Requires cargo

## Integration with Main Project

This case study integrates with the main `rust-logic-graph` project:

```
rust-logic-graph/                  # Main project
├── Cargo.toml                    # Defines examples
├── examples/                     # Examples directory
│   ├── purchasing_flow.rs        # Mock version (baseline)
│   ├── purchasing_flow_realdb.rs # Symlink/copy from case_study/src/
│   └── purchasing_flow_advanced.rs # Symlink/copy from case_study/src/
├── benches/
│   └── purchasing_flow_benchmark.rs # Benchmarks
└── case_study/                   # THIS DIRECTORY
    └── ...
```

**Note**: The `src/` examples are also available in `../examples/` for cargo to find.

## Maintenance

### Adding New Documentation
1. Create file in `docs/`
2. Update this STRUCTURE.md
3. Add link to README.md
4. Cross-reference in CASE_STUDY_INDEX.md

### Adding New Code Examples
1. Create file in `src/`
2. Copy/symlink to `../examples/`
3. Update Cargo.toml if needed
4. Document in README.md

### Updating Database Schema
1. Edit `sql/purchasing_flow_setup.sql`
2. Update docs to reflect changes
3. Test with `scripts/setup_databases.sh`

### Updating Scripts
1. Edit script in `scripts/`
2. Make executable: `chmod +x scripts/<script>.sh`
3. Test thoroughly
4. Document in README.md

## Best Practices

### For Readers
1. Start with README.md
2. Follow suggested learning paths
3. Run examples progressively (mock → real DB → advanced)
4. Refer to QUICKSTART.md for quick help

### For Contributors
1. Keep documentation in sync
2. Use relative links
3. Test all examples before committing
4. Update STRUCTURE.md when adding files

### For Maintainers
1. Keep examples working with latest dependencies
2. Update documentation for API changes
3. Maintain cross-references
4. Test all scripts periodically

---

**For navigation help, see [`docs/CASE_STUDY_INDEX.md`](docs/CASE_STUDY_INDEX.md)**

**For quick start, see [`docs/QUICKSTART.md`](docs/QUICKSTART.md)**

**For complete guide, see [`docs/CASE_STUDY.md`](docs/CASE_STUDY.md)**
