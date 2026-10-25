# Installation Guide

## Prerequisites

- Rust toolchain 1.99.1 (via rust-toolchain.toml)
- PostgreSQL 16+ with pgvector extension
- Tantivy (bundled as crate dependency)

## Build

```bash
# Platform crate
cd platform
cargo build --locked --release

# Eval crate
cd ../eval
cargo build --locked --release
```

## Run

```bash
# Self-test
cargo run --locked --bin ekp_platform -- --selftest

# Ablation report
cargo run --locked --bin ekp_platform -- --out report_out

# Coverage report
cargo run --locked --bin ekp_platform -- --coverage-report

# LegalRuleML export
cargo run --locked --bin ekp_platform -- --export-legalruleml export_out
```

## Local-First Workflow

1. All data in `platform/data/` (JSON files)
2. All tests in `platform/tests/` and `platform/src/*/tests`
3. All docs in `docs/`
4. Git version control (local, no remote required)
5. Check scripts: `scripts/check.ps1` and `scripts/check.sh`
