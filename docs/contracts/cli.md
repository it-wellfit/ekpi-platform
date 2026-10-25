# CLI contracts

- **Status:** Active
- **Owner:** Engineering
- **Last reviewed:** 2026-10-26
- **Contract IDs:** `CTR-CLI-001` (`ekp_platform`), `CTR-CLI-002` (`ekp_eval`)

Both CLIs are local, offline tools. Commands must be run from the crate directory because relative data paths resolve from the process working directory or the crate manifest, and the crates are independent (no root Cargo workspace).

## `ekp_platform`

### Usage

```text
usage: ekp_platform [--selftest | --out DIR [--cases PATH] [--data DIR] [--threshold F]
        [--params PATH] [--declared-rules-schema] [--export-legalruleml DIR]
        [--coverage-report] [--html] | --search QUERY | --eval --test-set PATH]
```

Run from `platform/`:

```powershell
cargo run --locked --bin ekp_platform -- --selftest
```

### Flags

| Flag | Argument | Behavior | Default |
| --- | --- | --- | --- |
| `--selftest` | none | Runs the engine integration self-test and exits. Takes precedence over other flags. | — |
| `--out` | directory | Generates the evaluation report and snapshots in the directory. | — |
| `--cases` | file path | Overrides the case file used by the report runner. | Seed cases |
| `--data` | directory | Overrides the seed data directory. | `platform/data` |
| `--threshold` | float in `[0,1]` | Overrides `retrieval_threshold`; invalid values are refused. | Loaded config or engine default |
| `--params` | file path | Loads the governance parameter file; invalid/tampered invariants fail closed. | `config/engine-params.json` when present |
| `--declared-rules-schema` | none | Validates all declared rules against the rule schema and exits. | — |
| `--export-legalruleml` | directory | Exports active rules as LegalRuleML 1.0 compact XML. | — |
| `--coverage-report` | none | Runs seed + casebook coverage and writes `coverage_report.md`; with `--out`, writes it inside the output directory. | — |
| `--html` | none | With `--out`, renders `report.md` to `report.html`. Without `--out`, refused with exit code 2. | — |
| `--search` | query string | Lexical search over every document's latest version. Prints up to five verified verbatim citations, or `insufficient evidence` with a reason. This mode does not receive a user identity and therefore applies **no permission filter**. | — |
| `--eval` | none | Runs a fixed retrieval regression set. Requires `--test-set`. Exits `1` if any case or coverage gate fails. | — |
| `--test-set` | file path | JSON evaluation set (`{ "version", "threshold", "cases": [...] }` or a bare array). The set may declare its own operating threshold; an explicit `--threshold` overrides it. | — |

`--selftest` is the primary integration check. `--declared-rules-schema` may be run alone and returns success after validating all records. `--html` is not a standalone mode. `--search` and `--eval` are standalone modes; combining them with `--out` does not generate a report.

### Search evaluation set

The fixed set is [platform/data/eval/search_eval.json](../../platform/data/eval/search_eval.json) (`CTR-DATA-007`). It contains 24 cases across `general`, `ambiguous`, `adversarial`, `wrong_source` and `insufficient_context`; six cases expect an honest refusal. `tool_failure` is not a case category because retrieval calls no external tool; the CLI's missing-data path is covered by a fail-closed integration test instead.

The gate is `metrics.gate_failures()` in `platform/src/eval.rs`: at least 10 cases, zero failed cases, permission isolation 1.00, and at least one case in every required category. A green run at threshold `0.42` is retrieval regression evidence, not an LLM or production-quality claim.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success; requested checks/artifacts completed |
| `1` | Fail-closed failure: invalid/tampered parameters, invariant mismatch, schema rejection, export/report IO failure, self-test failure, missing/unreadable data source, failed evaluation gate |
| `2` | Usage or argument error: unknown flag, missing value, invalid threshold, `--html` without `--out`, `--eval` without `--test-set` |

An exit code of `0` means the requested operation completed; it does not mean the target product is validated or production-ready.

### Artifacts with `--out`

| Artifact | Content | Evidence rule |
| --- | --- | --- |
| `report.json` | Per-system metrics, confusion matrix, coverage curve, per-case results | Every number must be derivable from snapshots, hard gates or persisted case counts |
| `report.md` | Human-readable report, including per-case failure detail | Must agree with `report.json` |
| `report.html` | CSS-only rendering of `report.md` | Must contain zero `<script>` elements |
| `engine_snapshots.json` | Engine decisions and replay inputs | Replaying the same snapshot must be identical |
| `engine_hard_gates.json` | H-1–H-6 measurements | Hard gates must not be relaxed |
| `coverage_report.md` | Rule coverage, verdict distribution, anchors and observed citations | Generated when `--coverage-report` is combined with `--out` |

`--export-legalruleml DIR` writes XML under `DIR`; `--coverage-report` without `--out` writes `coverage_report.md` in the current working directory.

### Examples

```powershell
# Fast integration check
cargo run --locked --bin ekp_platform -- --selftest

# Schema validation only
cargo run --locked --bin ekp_platform -- --declared-rules-schema

# Full report + coverage + export + HTML
cargo run --locked --bin ekp_platform -- --params config/engine-params.json `
  --out target/regression-report --coverage-report `
  --export-legalruleml target/regression-lrml --html

# Refusal example: --html without --out
cargo run --locked --bin ekp_platform -- --html
# expected exit code: 2

# Knowledge search with verified citations
cargo run --locked --bin ekp_platform -- --search "變更執行前，須完成風險評估並取得核准紀錄"

# Fixed retrieval regression gate
cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
```

## `ekp_eval`

### Usage

```text
usage: ekp_eval [--selftest] [--cases PATH] [--out DIR] [--systems a,b,c]
```

Run from `eval/`.

| Flag | Argument | Behavior | Default |
| --- | --- | --- | --- |
| `--selftest` | none | Runs the harness self-test and exits | — |
| `--cases` | file path | Case JSONL input | `eval/cases/seed_cases.jsonl` |
| `--out` | directory | Report output directory | `report_out` |
| `--systems` | comma-separated names | Selects adapters | `stub:oracle,stub:allow_all,stub:refuse_all` |

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `2` | Usage error, case-load failure, unknown adapter, or adapter bookkeeping failure |

`ekp_eval` does not return `1` in the current implementation. Treat any non-zero exit as a failed run, not as a partial pass.

### Artifacts

`report.json` and `report.md` under `--out`. The report includes per-system metrics and per-case failure analysis. The default adapters are deterministic stubs used to prove the metric pipeline; they are not production systems.

## Compatibility rules

- Adding an optional flag is backward-compatible; document it and add a check.
- Removing/renaming a flag or changing an exit-code meaning is breaking; require an accepted spec/ADR, a migration note and updated consumers.
- Artifact schema changes must preserve the derivation rule and update [data formats](data-formats.md).
- The CLI is not a stable public API for external automation until a versioning policy and consumer list are approved.

## Verification

- `cargo test --locked` in both crates.
- `--selftest` in both crates.
- `--declared-rules-schema` and `--eval --test-set data/eval/search_eval.json` in `platform/`.
- `gate.ps1 -IncludeSmoke` for report/export/coverage/HTML and structural LegalRuleML checks.
- `scripts/check-docs.ps1` for documentation links (not a CLI behavior test).

## Update triggers

Update this document when a flag, argument, exit code, artifact, default path or compatibility rule changes. Update `CTR-CLI-001`/`CTR-CLI-002` evidence in the [traceability register](../traceability/register.md) in the same change.
