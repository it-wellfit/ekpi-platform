# CLI contracts

- **Status:** Active
- **Owner:** Engineering
- **Last reviewed:** 2026-10-29
- **Contract IDs:** `CTR-CLI-001` (`ekp_platform`), `CTR-CLI-002` (`ekp_eval`)

Both CLIs are local, offline tools. Commands must be run from the crate directory because relative data paths resolve from the process working directory or the crate manifest, and the crates are independent (no root Cargo workspace).

## `ekp_platform`

### Usage

```text
usage: ekp_platform [--selftest | --health [--data DIR] | --out DIR [--cases PATH] [--data DIR] [--threshold F]
        [--params PATH] [--declared-rules-schema] [--export-legalruleml DIR]
        [--coverage-report] [--html] | --search QUERY [--audit-log PATH]
        [--llm PROVIDER[:MODEL]] | --eval --test-set PATH | --import-wellfit PATH]
```

Run from `platform/`:

```powershell
cargo run --locked --bin ekp_platform -- --selftest
```

### Flags

| Flag | Argument | Behavior | Default |
| --- | --- | --- | --- |
| `--selftest` | none | Runs the engine integration self-test. It is mutually exclusive with every other flag, including `--health`, `--search`, `--eval`, `--declared-rules-schema` and the report/export flags; after the self-test completes, any other argument exits `2` instead of being silently ignored. Malformed arguments (for example `--bogus`) are also rejected after the self-test, so a typo is never reported as success. | — |
| `--health` | none (optional `--data`) | Local liveness diagnostic. Exits `0` and prints `health: OK` when `<data>/rules.json`, `docs.json` and `processes.json` are present as regular files; otherwise prints `health: DEGRADED (missing: ...)` and exits `1`. It does not parse the files, so `OK` does not prove data validity or readiness. Only `--data` may accompany it; every other flag is a usage error (exit `2`), and unknown flags are rejected before any status is printed. | `platform/data` |
| `--out` | directory | Generates the evaluation report and snapshots in the directory. A missing value is a usage error; the default directory is **not** substituted. | — |
| `--cases` | file path | Overrides the case file used by the report runner. Requires `--out`. | Seed cases |
| `--data` | directory | Overrides the seed data directory. | `platform/data` |
| `--threshold` | float in `[0,1]` | Overrides `retrieval_threshold`; invalid values are refused. | Loaded config or engine default |
| `--params` | file path | Loads the governance parameter file; invalid/tampered invariants fail closed. | `config/engine-params.json` when present |
| `--declared-rules-schema` | none | Validates all declared rules against the rule schema. It may run alone (success exits `0`) or before another mode; a violation exits `1` before search/eval/report work. | — |
| `--export-legalruleml` | directory | Exports active rules as LegalRuleML 1.0 compact XML. | — |
| `--coverage-report` | none | Runs seed + casebook coverage and writes `coverage_report.md`; with `--out`, writes it inside the output directory. | — |
| `--html` | none | With `--out`, renders `report.md` to `report.html`. Without `--out`, refused with exit code 2. | — |
| `--search` | query string | Lexical search over every document's latest version. Prints up to five verified verbatim citations, or `insufficient evidence` with a reason. This mode does not receive a user identity and therefore applies **no permission filter**. Cannot be combined with `--eval` or report-only flags. | — |
| `--eval` | none | Runs a fixed retrieval regression set. Requires `--test-set`. Exits `1` if any case or coverage gate fails. Cannot be combined with `--search` or report-only flags. | — |
| `--test-set` | file path | JSON evaluation set (`{ "version", "threshold", "cases": [...] }` or a bare array). The set may declare its own operating threshold; an explicit `--threshold` overrides it. Requires `--eval`. | — |
| `--audit-log` | file path | With `--search`, appends one line to a local JSONL hash-chained audit log. Each v2 event includes `id`, `event_type`, `actor: search-cli`, `tenant_id: default`, query/citation/result/latency payload, a UTC RFC3339 `timestamp`, and `prev_hash`. The payload is stored as a JSON object produced by the JSON serializer, so quotes, backslashes, control characters and newlines in the query cannot break the event JSON. The hash covers the timestamp and uses a domain separator plus length-prefixed fields. Event IDs must be unique within the file. Before appending, the CLI reads and verifies the existing chain and validates every existing event's `id`, `actor`, `tenant_id` and `payload` structure; a malformed line, blank line, truncated final line, missing/invalid timestamp (pre-v2), duplicate event ID, structurally invalid event, broken chain or the legacy single-object format is refused with exit `1` and the file is left unchanged. Writers serialise on an operating-system advisory lock held on a sibling `<PATH>.lock` file; the kernel releases it when the process exits, so a leftover lock file does not block a new writer. One complete line is written and `sync_all` is called; a failed write is rolled back to the previous file length. A missing or empty file starts a new chain. It is a local diagnostic record, not WORM, an authenticated audit service, or a signed/externally anchored store. Without `--search` it is a usage error (exit `2`). | — |
| `--llm` | `PROVIDER[:MODEL]` | With `--search`, adds a labeled mock RAG demonstration. Only `mock` is implemented; the model label is accepted but unused. Any other provider is a usage error (exit `2`) before search or audit work. The answer is prefixed `mock-answer:` and the output states `provider: mock (no external model)`; the RAG orchestrator refuses an answer that lacks an exact `[doc_id@version]` citation. Without `--search` it is a usage error. This is not an LLM integration or AI-quality evidence. | — |
| `--import-wellfit` | file path | Legacy local import retained for prototype data preparation. Reads a JSON array, adds documents whose `doc_id` is not already in `<data>/docs.json`, defaults missing fields (`version` `1.0`, `effective_from` `2026-01-01`, `section` `1`) and missing permissions to `viewer`/`editor`, then validates document shape before saving. If an existing `docs.json` cannot be loaded, it fails closed without overwriting it. It does not validate the input against a published schema, authenticate the importer or create an audit event. It cannot be combined with `--search`, `--eval` or report-only flags, and it is not part of the supported pilot workflow. | — |

`--selftest` is the primary integration check and is mutually exclusive with every other flag. `--declared-rules-schema` may be run alone and returns success after validating all records; when combined with another mode it validates first and the mode continues. `--html` is not a standalone mode. `--search` and `--eval` are standalone modes: combining either with `--out`, `--cases`, `--html`, `--coverage-report` or `--export-legalruleml` is rejected with exit code `2` instead of silently ignoring the flag.

### Argument validation

Before any data load, report write or filesystem side effect, the CLI scans the whole argument list:

- an unknown flag (including `--bogus` after a valid flag) is a usage error;
- a value flag without a value, with an empty value, or followed by another flag (`--out --html`) is a usage error — the value is never taken from a default;
- a bare positional argument is a usage error;
- `--test-set` without `--eval`, `--cases` without `--out`, and `--html` without `--out` are usage errors;
- `--search` and `--eval` are mutually exclusive, and neither accepts report/export flags;
- `--audit-log` and `--llm` require `--search`; `--llm` accepts only the `mock` provider, and any other provider is rejected before data load or audit write;
- `--import-wellfit` cannot be combined with `--search`, `--eval` or report/export flags;
- `--health` accepts only `--data`; combining it with `--selftest`, `--search`, `--eval`, `--import-wellfit`, report/export flags or any other flag is a usage error, and validation happens before the health status is printed;
- `--selftest` is mutually exclusive with every other flag: after the self-test, any additional argument (including a syntactically valid mode or report flag) is a usage error instead of being silently ignored.

A usage error prints the message and usage text to stderr and exits `2`. Validation runs before the self-test's result is returned, so `--selftest --bogus` runs the self-test and then exits `2`, preventing a typo from being reported as success.

### Search evaluation set

The fixed set is [platform/data/eval/search_eval.json](../../platform/data/eval/search_eval.json) (`CTR-DATA-007`). It contains 28 cases across `general`, `ambiguous`, `adversarial`, `wrong_source`, `insufficient_context` and `permission`; eight cases expect an honest refusal. Four permission cases use two evaluation-only restricted fixtures (`quality_manager` and `auditor`, disjoint roles): two expect an allowed citation and two expect an honest refusal. `tool_failure` is not a case category because retrieval calls no external tool; the CLI's missing-data path is covered by a fail-closed integration test instead.

The gate is `metrics.gate_failures()` in `platform/src/eval.rs`: at least 10 cases, zero failed cases, at least one case in every required category, at least two role-restricted cases, at least one allowed and one denied permission case, and zero permission violations. The current set reports 4 role-restricted cases (2 allowed, 2 denied) and 0 violations, so permission isolation is 1.00 with a non-empty denominator. The fixtures are merged into a clone only for `--eval`; ordinary `--search` never loads them and an integration test asserts the fixture text remains invisible there. This proves the library retrieval filter, not authenticated authorization or tenant isolation (`EVAL-GAP-06` closed as fixed-set coverage; S19 remains `Partial`). A green run at threshold `0.42` is retrieval regression evidence, not an LLM or production-quality claim.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success; requested checks/artifacts completed |
| `1` | Fail-closed failure: invalid/tampered parameters, invariant mismatch, schema rejection, export/report IO failure, self-test failure, missing/unreadable data source, failed evaluation gate, or a `--health` check with missing core data files |
| `2` | Usage or argument error: unknown flag, unexpected positional argument, missing/empty value, a flag used as another flag's value, invalid threshold, `--html`/`--cases` without `--out`, `--eval` without `--test-set`, `--test-set` without `--eval`, `--search` with `--eval`, a search/eval mode combined with a report-only flag, `--audit-log`/`--llm` without `--search`, an unsupported `--llm` provider, `--health` combined with another flag other than `--data`, `--selftest` combined with any other flag, or `--import-wellfit` combined with another mode |

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

# Labeled mock RAG demonstration; no external model is called
cargo run --locked --bin ekp_platform -- --search "變更執行前，須完成風險評估並取得核准紀錄" --llm mock:test

# Local search audit record: appends to a JSONL hash chain (not an authenticated audit service)
cargo run --locked --bin ekp_platform -- --search "變更執行前，須完成風險評估並取得核准紀錄" --audit-log target/search-audit.jsonl

# A second run appends to the same chain; a corrupt, tampered, truncated, duplicate-ID or legacy file exits 1 without modification
cargo run --locked --bin ekp_platform -- --search "變更執行前，須完成風險評估並取得核准紀錄" --audit-log target/search-audit.jsonl

# Fixed retrieval regression gate
cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
```

## `ekp_eval`

### Usage

```text
usage: ekp_eval [--cases PATH | --corpus PATH] [--out DIR] [--systems a,b,c]
       ekp_eval --selftest
       ekp_eval --corpus-selftest
       ekp_eval --make-annotation-sheet PATH [--corpus PATH]
       ekp_eval --kappa ANNOTATOR_A.jsonl ANNOTATOR_B.jsonl [--corpus PATH]
```

Run from `eval/`.

| Flag | Argument | Behavior | Default |
| --- | --- | --- | --- |
| `--selftest` | none | Runs the 9-case adapter self-test and exits. It must be the only argument. | — |
| `--corpus-selftest` | none | Runs the fixed 67-case corpus self-test and exits. It must be the only argument. | — |
| `--cases` | file path | Case JSONL input | `eval/cases/seed_cases.jsonl` |
| `--corpus` | manifest path | Loads the corpus declared by a manifest. Mutually exclusive with `--cases`. | `eval/cases/corpus.json` for corpus, κ and sheet modes |
| `--out` | directory | Report output directory | `report_out` |
| `--systems` | comma-separated names | Selects adapters; the list must be non-empty and contain no duplicates | `stub:oracle,stub:allow_all,stub:refuse_all` |
| `--make-annotation-sheet` | output file path | Writes the blinded annotation sheet and exits. Accepts one optional `--corpus`. | — |
| `--kappa` | two annotation file paths | Computes Cohen's κ for two independent annotators. Accepts one optional `--corpus`. | — |

`--kappa` requires the two annotation files to cover the fixed corpus case ids exactly once. A missing case, an extra case or an invented case id is rejected; the study cannot silently shrink to the cases the annotators happened to label.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success; for `--kappa`, the proposed threshold was met |
| `1` | `--kappa` only: the study is valid but below the proposed 0.80 threshold |
| `2` | Usage error, case/corpus-load failure, unknown or duplicate adapter, corpus-coverage failure, invalid annotation input, or artifact-write failure |

Treat any non-zero exit as a failed run, not as a partial pass. `--selftest` and `--corpus-selftest` also exit `2` when they receive any extra argument.

### Artifacts

`report.json` and `report.md` under `--out`. The report includes per-system metrics and per-case failure analysis. The default adapters are deterministic stubs used to prove the metric pipeline; they are not production systems.

`--make-annotation-sheet` writes the requested JSONL file. Each row contains only the case facts (`case_id`, `task`, `doc_versions`, `events`, `rules`, `loop`) plus `verdict: null` and `annotator: null`; it must not contain `expected_verdict`, `notes`, `citations`, `category` or `abstain_required`.

`--kappa` prints a JSON object to standard output with `corpus_id`, `corpus_cases`, `n`, observed and expected agreement, `cohens_kappa`, the interpretation band, the acceptance threshold and the pass flag. It does not write a report directory.

## Compatibility rules

- Adding an optional flag is backward-compatible; document it and add a check.
- Removing/renaming a flag or changing an exit-code meaning is breaking; require an accepted spec/ADR, a migration note and updated consumers.
- Artifact schema changes must preserve the derivation rule and update [data formats](data-formats.md).
- The CLI is not a stable public API for external automation until a versioning policy and consumer list are approved.

## Verification

- `cargo test --locked` in both crates.
- `platform/tests/cli_contract.rs` runs the real binary for the rejection paths: unknown flag (alone and after a valid flag), missing/empty value, flag-as-value, unexpected positional argument, `--test-set`/`--cases`/`--html` prerequisites, `--search` + `--eval`, report flags with eval, and self-test precedence. It asserts that `--selftest` combined with a valid mode flag exits `2` without running that mode. It also covers `--audit-log` refusal without `--search`, two successive runs forming a verified 2-event chain, refusal of a corrupt chain and of a legacy single-object file without modifying either file, a hostile query whose quotes/backslashes/control characters must still produce a valid JSON payload, `--llm mock` labeling with a versioned citation, unsupported-provider rejection before audit write, and legacy `--import-wellfit` fail-closed/validation behavior. `--health` tests cover the OK and DEGRADED exits, unknown-flag rejection and refusal to combine it with another mode. The suite asserts exit codes (including exit `1` for an audit write failure) and that no report, audit or imported data is written on a usage error. Audit unit tests additionally cover timestamp hashing, pre-v2/blank/truncated refusal, structural validation of existing events, duplicate event IDs, leftover lock files, rollback and concurrent appenders.
- `eval/tests/cli_contract.rs` spawns the real `ekp_eval` binary. It covers `--selftest` and `--corpus-selftest` exclusivity, unknown flags with no report write, `--cases`/`--corpus` mutual exclusion, empty and duplicate `--systems` selections, κ argument errors, exact-corpus κ success, a missing case, a foreign case id, a reused annotator id, a leaked `expected_verdict` field, and blinded-sheet generation and refusal. The κ success case asserts that the reported `corpus_id`, `corpus_cases`, `n` and `cohens_kappa` agree with the fixed corpus.
- `--selftest` and `--corpus-selftest` in `eval/`; `--selftest` in `platform/`.
- `--declared-rules-schema` and `--eval --test-set data/eval/search_eval.json` in `platform/`.
- `gate.ps1 -IncludeSmoke` for report/export/coverage/HTML and structural LegalRuleML checks.
- `scripts/check-docs.ps1` for documentation links (not a CLI behavior test).

## Update triggers

Update this document when a flag, argument, exit code, artifact, default path or compatibility rule changes. Update `CTR-CLI-001`/`CTR-CLI-002` evidence in the [traceability register](../traceability/register.md) in the same change.
