# Contracts

- **Status:** Active
- **Owner:** Engineering
- **Last reviewed:** 2026-10-26

A contract is any interface another person, process or test depends on. This repository has CLI, HTML-form, JSON-data and XML-export contracts. It does **not** have an approved production JSON API.

## Contract inventory

| ID | Contract | Canonical document | Version | Consumers | Verification |
| --- | --- | --- | --- | --- | --- |
| CTR-CLI-001 | `ekp_platform` CLI | [CLI](cli.md#ekp_platform) | 0.1.0 | Operators, CI, release checks | `--selftest`, schema check, argument-contract tests, smoke gate |
| CTR-CLI-002 | `ekp_eval` CLI | [CLI](cli.md#ekp_eval) | 0.1.0 | Evaluation runs, CI | `--selftest`, tests |
| CTR-WEB-001 | Local governance HTML routes and forms | [Web routes](web-routes.md) | Local prototype | Browser operator, route tests | `platform/tests/governance_web.rs` |
| CTR-DATA-001 | `platform/data/docs.json` | [Data formats](data-formats.md#documents) | Fixture v1 | Engine, tests | Integration tests; SHA-256 binding |
| CTR-DATA-002 | `platform/data/rules.json` | [Data formats](data-formats.md#rules) | Schema v0.1.1 | Engine, export, coverage | `--declared-rules-schema` |
| CTR-DATA-003 | `platform/data/processes.json` | [Data formats](data-formats.md#processes) | Fixture v1 | Process replay, tests | Integration tests |
| CTR-DATA-004 | `platform/config/engine-params.json` | [Data formats](data-formats.md#engine-parameters) | 1.0 | CLI fail-closed checks | `--params`, invalid-params tests |
| CTR-DATA-005 | Report and snapshot artifacts | [Data formats](data-formats.md#report-and-snapshot-artifacts) | De facto v1 | Operators, release evidence | Smoke gate; derivation rule |
| CTR-DATA-006 | Governance ledger JSONL | [Data formats](data-formats.md#governance-ledger) | Local v1 | `governance_web`, audit view | Hash-chain verification; route tests |
| CTR-DATA-007 | Search evaluation set | [Data formats](data-formats.md#search-evaluation-set) | 1.0 | `--eval`, integration tests | Loader validation; fixed-set gate; `platform/tests/search_eval.rs` |
| CTR-EXP-001 | LegalRuleML 1.0 compact export | [Data formats](data-formats.md#legalruleml-export) | 1.0 | External legal-rule consumers | Smoke gate structural checks; exporter self-check |
| — | JSON HTTP API | Not applicable | — | — | `platform/src/api.rs` is an unwired placeholder; see [backend](../backend/README.md#api-surface) |

## Compatibility policy

1. **Additive changes are preferred.** Adding an optional field or a new command flag is backward-compatible; document it and add a test.
2. **Breaking changes require a decision.** Removing or renaming a flag, field, route, event type or required artifact requires an ADR or an accepted spec, a migration/compatibility note, and updated consumers and tests.
3. **Schema versions are explicit.** Rule data carries a schema version; the schema file is `design/rule-schema-v0.1.1.schema.json`. A new schema version needs a new file, validation tests and fixture migration.
4. **Artifacts are evidence, not a database.** Report and snapshot formats may evolve, but public numbers must remain derivable from named sources. Never silently change a metric definition; update the register and report together.
5. **HTML routes are a local prototype surface.** They may change with the UI, but a route or form change still needs a test and a traceability update.
6. **LegalRuleML is an external standard.** Preserve the official namespace and required structure; do not substitute a stub or a private approximation.
7. **No unversioned API promise.** The placeholder `api.rs` is not a contract and must not be exposed or documented as stable.

## Change process

For any contract change:

1. Link the accepted issue/spec and risk tier.
2. Identify affected consumers and update their tests.
3. Update the contract document and the schema/version if needed.
4. Run the focused contract check and the full baseline.
5. Update the traceability register with the contract ID and evidence.
6. If the change is breaking, record the migration/rollback in the release template.

## Contract tests

| Contract | Minimum check |
| --- | --- |
| CLI | `cargo run --locked --bin ekp_platform -- --selftest`; `--declared-rules-schema`; `--eval --test-set data/eval/search_eval.json`; `platform/tests/cli_contract.rs` (exit-code and no-write rejection paths); smoke gate with report/export/coverage/HTML |
| Eval CLI | `cargo test --locked`; `cargo run --locked -- --selftest` |
| Search evaluation set | `platform/tests/search_eval.rs`; gate rejects wrong expectations and a missing data source |
| Web routes/forms | `cargo test --locked` (governance route tests) |
| Rule schema | `--declared-rules-schema`; schema validation of all 20 records |
| Report artifacts | `gate.ps1 -IncludeSmoke`; artifact presence and zero-script HTML |
| LegalRuleML | Root/namespace, size, 19/19/19/16 structural counts |
| Ledger | Route tests; audit page chain verification |

## Out of scope

- OpenAPI/Swagger: no production HTTP API exists.
- Event bus or message schemas: no event bus exists.
- Database migration contracts: no database exists.
- Authentication/authorization contracts: not implemented in the served app.

These become required only when the corresponding Phase B capability is approved. Do not create empty OpenAPI or event-schema files to look complete.

## Update triggers

Update this index and the affected contract document when a flag, route, form field, schema, artifact, export or compatibility rule changes. Update the traceability register in the same change.
