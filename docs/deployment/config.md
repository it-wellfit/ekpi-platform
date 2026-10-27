# Configuration reference

- **Status:** Current for the local prototype
- **Owner:** Engineering
- **Last reviewed:** 2026-10-26
- **Contracts:** [Data formats](../contracts/data-formats.md)
- **Operations:** [Operations](../operations/README.md)

## Engine parameters

`platform/config/engine-params.json` is validated fail-closed. If a declared invariant does not match the expected value, the CLI refuses to run.

| Parameter | Default | Range | Description |
| --- | --- | --- | --- |
| `params_version` | `1.0` | exact | Parameter-file version |
| `declared_invariants` | locked | exact | Non-negotiable behavior flags; any mismatch refuses execution |
| `adjustable.retrieval_threshold` | `0.70` | `0.0`–`1.0` | Minimum retrieval score for evidence inclusion |

`--params PATH` selects a parameter file. `--threshold F` overrides only `retrieval_threshold` and must be in `[0,1]`. There is no other adjustable engine parameter.

The declared invariants include: draft rules are not consumable; ambiguous bindings refuse; unregistered-rule guessing is forbidden; equal timestamps are not overdue; abstention clears citations; forbidden pairs are never released; an injection directive never operates as a system rule; the decision path contains no probability component.

## Data files

| File | Description |
| --- | --- |
| `platform/data/docs.json` | Versioned documents with clauses, anchors, effective windows and synthetic injection directives |
| `platform/data/rules.json` | 20 rule records (19 active + 1 draft poisoning test) validated against `design/rule-schema-v0.1.1.schema.json` |
| `platform/data/processes.json` | Process models with precedence, parallel and forbidden pairs, repeat policy |

`--data DIR` overrides the seed data directory for the CLI. Data is read-only fixture content; do not edit generated artifacts as source.

## Environment variables

| Variable | Read by | Default | Description |
| --- | --- | --- | --- |
| `EKP_GOVERNANCE_DATA` | `governance_web` | `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` (Windows) or the platform data fallback | Local governance ledger path |
| `EKPI_LOCAL_AUTH_USER` | `governance_web` | none | Username for the single local workbench account |
| `EKPI_LOCAL_AUTH_PASSWORD` | `governance_web` | none | Password for that account; minimum 8 characters, maximum 128; never commit or log it |
| `EKPI_LOCAL_AUTH_ROLE` | `governance_web` | `editor` | `viewer`, `editor`, `approver` or `admin`; unknown values fall back to `editor` |
| `EKPI_COOKIE_SECURE` | `governance_web` | unset | Set to `1` to add `Secure` to the session cookie; requires an HTTPS origin to be useful |

The local auth variables are optional at process start, but without both user and password the server starts with no account and every data page returns `401`. There is no default account. `DATABASE_URL`, `RUST_LOG` and similar variables are not used: there is no database and no tracing/log-level configuration.

Do not assume an environment variable configures behavior unless the code reads it. If a new variable is added, update this table, `.env.example`, the contract and a test in the same change.

## Output locations

| Flag | Output |
| --- | --- |
| `--out DIR` | `report.json`, `report.md`, `engine_snapshots.json`, `engine_hard_gates.json`; `report.html` with `--html`; `coverage_report.md` with `--coverage-report` |
| `--export-legalruleml DIR` | LegalRuleML 1.0 compact XML for active rules |
| `--coverage-report` without `--out` | `coverage_report.md` in the current working directory |

Output directories are operator-selected. Generated artifacts are not transactional; a killed process can leave partial files. Regenerate them; do not hand-edit them.

## Local web configuration

The current `governance_web` binary has no configuration file for its bind address; it always binds `127.0.0.1:3000`. The ledger path and the optional local login/cookie variables above are the only configurable values. There is no TLS, reverse proxy, tenant setting, password recovery or user administration.

## Update triggers

Update this page when a parameter, data file, environment variable, output path or local web setting changes. Keep `.env.example` aligned with the variables actually read.
