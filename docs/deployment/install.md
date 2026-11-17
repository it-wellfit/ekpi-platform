# Installation guide (local prototype)

- **Status:** Current for the local prototype; **not** a production installation guide
- **Owner:** Engineering
- **Last reviewed:** 2026-11-17
- **Canonical operating procedures:** [Operations](../operations/README.md)

## Prerequisites

- Rust toolchain pinned by `rust-toolchain.toml` (currently 1.99.1) with Cargo.
- A supported local OS (Windows or Linux) with Git.
- No database, search engine, message queue, container runtime or cloud account is required.

PostgreSQL, pgvector, Tantivy, a message queue and a separate search service are **not** current dependencies. They appear only as research or target-product options in [tech selection](../tech-selection-round1.md); do not install them for this prototype.

## Build

The repository is a single Cargo workspace with one root `Cargo.toml` and lockfile. Build and verify from the repository root; select a package with `-p` when needed.

```powershell
cargo build --locked --release
```

## Verify

```powershell
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run --locked -p ekp_eval -- --selftest
cargo run --locked -p ekp_platform --bin ekp_platform -- --selftest
cargo run --locked -p ekp_platform --bin ekp_platform -- --declared-rules-schema
```

From the repository root, the fixed entry points are:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
powershell -ExecutionPolicy Bypass -File gate.ps1 -IncludeSmoke
```

```bash
./scripts/check.sh
```

## Run

```powershell
cd platform

# Engine integration self-test
cargo run --locked --bin ekp_platform -- --selftest

# Ablation/evaluation report, snapshots and hard gates
cargo run --locked --bin ekp_platform -- --out target/report

# Coverage report
cargo run --locked --bin ekp_platform -- --coverage-report

# LegalRuleML export
cargo run --locked --bin ekp_platform -- --export-legalruleml target/lrml
```

The local governance workbench is a separate binary. It has no default account, so set the login variables before starting it. Stage 1 uses a manual foreground start in a dedicated non-administrator console session:

```powershell
cd platform
$env:EKPI_LOCAL_AUTH_USER = "operator"
$env:EKPI_LOCAL_AUTH_PASSWORD = "choose-a-long-local-password"
$env:EKPI_LOCAL_AUTH_ROLE = "approver"   # viewer | editor | approver | admin
$env:EKP_GOVERNANCE_DATA = "D:\EKPI\stage1\governance-ledger.jsonl"
$env:EKP_APPROVAL_DATA = "D:\EKPI\stage1\approvals.jsonl"
$env:EKPI_DATA_DIR = "D:\EKPI\stage1\data"
$env:EKPI_BIND_IP = "127.0.0.1"
$env:EKPI_WEB_PORT = "3000"
cargo run --release --locked --bin governance_web
```

It binds **`127.0.0.1:3000` by default**. `EKPI_BIND_IP` accepts loopback addresses only and refuses non-loopback values in Stage 1; `EKPI_DATA_DIR` points at the synthetic `platform/data` fixtures used by `/search`, and when unset the compile-time path is used. Use synthetic or authorized non-sensitive data. Do not register a Windows service: the console binary has no SCM `ServiceMain`, so `sc.exe create` would fail with error 1053. The login is local process-memory authentication: there is no IdP, SSO, MFA, account recovery, persistent user database or tenant isolation. Without the two credential variables the server still starts, but data pages return `401`. Do not expose it beyond the local machine. See the [staging checklist](../operations/staging-checklist-draft.md), [operations](../operations/README.md), [configuration](config.md) and the [threat model](../architecture/threat-model.md).

## Data locations

| Path | Content | Notes |
| --- | --- | --- |
| `platform/data/docs.json` | Synthetic source documents and versions | Fixture; not customer content |
| `platform/data/rules.json` | 19 active + 1 draft rule record | Validated against `design/rule-schema-v0.1.1.schema.json` |
| `platform/data/processes.json` | 3 process models | Synthetic fixture |
| `platform/config/engine-params.json` | Engine parameters and locked invariants | Fail-closed |
| `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` or `EKP_GOVERNANCE_DATA` | Local governance ledger | Append-only hash chain; not WORM |
| `platform/out/`, `platform/report_out/`, `eval/report_out/` | Generated artifacts | Regenerate; do not treat as source |

## Local-first workflow

Git is the source of truth and works offline. A remote is an opt-in upgrade governed by [remote collaboration criteria](../remote-collaboration-criteria.md). See [local-first workflow](../operations/local-first-workflow.md) and [local backup](../operations/local-backup.md).

## Update triggers

Update this guide when a dependency, toolchain, command, bind address, data path or supported environment changes. Do not add a prerequisite that the code does not actually require.
