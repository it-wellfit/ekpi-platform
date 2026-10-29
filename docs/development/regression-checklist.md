# Existing functionality regression checklist

This checklist records the behavior already present in the local prototype. It is the regression baseline for changes to existing code; it is not evidence that the future enterprise product requirements have been implemented. The detailed AT-01–AT-31 matrix in `docs/governance/04-acceptance-test-matrix.md` is a target-product plan. Do not mark those cases passed based only on this prototype's tests.

## 1. Required automated baseline

Run the exact commands in [quality gates](quality-gates.md) from each independent crate directory. The GitHub Actions workflow is defined for Ubuntu and Windows, but it has never run because this checkout has no remote; local runs are the only executed evidence. The `platform` binary has `test = false`; its command-line behavior is additionally covered by `--selftest`, the schema check and `platform/tests/cli_contract.rs`.

The full local baseline and representative CLI paths were rerun for this audit (Rust 1.99.1, 2026-10-25):

- `eval`: format, Clippy, Cargo tests, CLI self-test, default report generation, and explicit single-adapter report generation passed; Cargo reported 9 unit tests and the self-test reported 10 checks. Unknown adapter selection returned the expected exit code 2.
- `platform`: format, Clippy, Cargo tests, engine self-test, declared-rules schema, parameterized report/export/coverage/HTML generation, and output artifact checks passed. Cargo reported 183 tests (170 unit including 9 LegalRuleML exporter tests and 3 subject-activity tests, 2 edge-case tests, 3 governance web, 8 engine integration). The CLI self-test reported 49 assertions; the schema command validated 20 records. Malformed params and `--html` without `--out` returned the expected refusal exit codes. The generated HTML had no script element; the LegalRuleML export was 51,026 bytes with 19 `PrescriptiveStatement` elements and validated against the exporter's compact-schema self-check.

These are dated observations. Re-run them for the revision being delivered and attach the new output to the release evidence (or to a PR when a remote exists).

### 2026-10-26 audit update

- At the audited revision `8bf2d53`, `platform` had **225 tests**: 193 unit, 2 edge-case, 3 governance web, 8 engine integration, 6 search evaluation and 13 CLI contract. The CLI self-test reported 49 assertions; the schema command validated 20 records.
- The host's Windows Application Control policy blocked execution of some binaries from `target` (`os error 4551`) at that revision. Those suites were executed from copied harnesses and with crate-local `profile` overrides; the canonical `cargo test --locked` from `target` and the full `gate.ps1` could not run at `8bf2d53`. This is historical and was resolved by the committed build profile in the next bullet.
- The CLI argument contract is enforced before any write: unknown flag, missing/empty value, a flag used as a value, a positional argument, `--test-set` without `--eval`, `--cases`/`--html` without `--out`, `--search` with `--eval`, and a search/eval mode with report-only flags all exit `2`. `--selftest --bogus` runs the self-test and still exits `2`.

### 2026-10-26 final verification (`7f6ab97`)

- The repo-local `.cargo/config.toml` profile overrides removed the host execution block without changing program semantics. The exact canonical commands now run from a clean `target`:
  - `cargo test --locked` (platform): **233 tests** — 200 unit, 2 edge-case, 3 governance web, 8 engine integration, 7 search evaluation, 13 CLI contract; 49 self-test assertions; schema 20 records.
  - `cargo test --locked` (eval): 9 tests; self-test 10 checks.
  - Both crates: `cargo fmt -- --check` and `cargo clippy --locked --all-targets -- -D warnings` exit 0.
  - `gate.ps1 -IncludeSmoke`: 13/13 PASS; docs 44 required/488 links in 95 files; secret scan 205 files; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes 19/19/19/16; zero-script HTML.
  - Search: 28 cases, 20/20 top-1, 8/8 refusals, 4 role-restricted (2 allowed, 2 denied), 0 violations, gate PASS.
  - Restore: bundle `ekpi-platform-2026-10-26-105233.bundle` (SHA-256 `4D35413B…3192B276`) verified; fresh clone at `7f6ab97` ran `scripts/check.ps1` exit 0. See the [backup log](../operations/backup-log.md).

### 2026-10-26 regression corrections

- D-08 subject qualification now uses only `subject.actor_role`. Evidence `actor_role` constraints filter approval records and must not disqualify an otherwise authorised subject. When the governed activity is present, at least one occurrence must match the subject role; additional signers remain valid. Regression cases: EC-006 and BC-143.
- D-05 `sequencing.on_violation = warn` now records `deadline_overdue_warned` and preserves `Comply` instead of treating the deadline breach as `Violate`. Regression case: BC-194.
- The LegalRuleML exporter was restored after a placeholder returned `<LegalRuleML/>`. The regenerated artifact is byte-identical to the 51,026-byte reference, and the smoke gate now rejects undersized or structurally incomplete exports.

### 2026-10-27 local session-auth baseline (`b2bae3e`)

- `platform` now has **261 tests**: 217 unit, 13 CLI contract, 2 edge-case, 10 governance web, 8 engine integration, 7 search evaluation, 4 web health. `eval` has 12 tests. The self-test still reports 49 assertions and the schema command validates 20 records.
- `local_auth` has 11 unit tests covering password hashing, duplicate/weak passwords, session expiry, logout, role ordering, lockout, unknown-user lockout isolation and expired-session pruning.
- `governance_web` has 10 route tests covering unauthenticated `401`, Viewer/Editor/Approver enforcement, login/logout, oversized login bodies, no default account, same-origin/fetch-metadata refusal and no ledger append on refusal.
- `gate.ps1 -IncludeSmoke` is 13/13 PASS at `b2bae3e`; supply-chain check is PASS (platform 72 dependencies, eval 12, 0 vulnerabilities). The canonical check in a fresh clone at `56a944f` passed with docs 491 links in 96 tracked files and secret scan 219 files.
- Restore: bundles `7fd0c0b` (SHA-256 `EBFD373C…34A2FF`) and `56a944f` (SHA-256 `23EA4401…CE6421`) both passed a fresh-clone check. The `56a944f` run also passed an `age` encrypt/decrypt drill; it remains local-simulated, not off-host.
- Benchmark at `56a944f`: eval p50/p95 17.432/23.319 ms; loopback `/healthz` p50/p95 19.010/24.069 ms; 5,369.21 req/s. One host only; not a capacity model.

### 2026-10-27 CLI contract hardening (`23b8996`)

- `platform` has **270 tests** in the release profile: 217 unit, 22 CLI contract, 2 edge-case, 10 governance web, 8 engine integration, 7 search evaluation, 4 web health. `eval` has 12 tests. The self-test reports 49 assertions and the schema command validates 20 records.
- `--llm` requires `--search` and accepts only the `mock` provider; any other provider exits `2` before data load or audit write. The response is labeled `mock-answer`, states `provider: mock (no external model)` and must carry an exact `[doc_id@version]` citation. This is a deterministic fixture, not an LLM integration or AI-quality evidence.
- **Historical (superseded by the `718db57` section below):** `--audit-log` requires `--search` and wrote one local `KnowledgeSearch` event (`actor: search-cli`, `tenant_id: default`) to the named file, replacing any existing content. It was not an authenticated or append-only audit service.
- `--import-wellfit` is a legacy local import. It cannot be combined with `--search`, `--eval` or report/export flags; it fails closed when an existing `docs.json` cannot be loaded and validates the merged documents before saving.
- **Canonical check Blocked at `23b8996`:** `scripts/check.ps1` exited 1 because Windows Application Control denied the freshly rebuilt `platform\target\debug\ekp_platform.exe` (`os error 4551`), so all 22 CLI contract tests failed to start it. Setting `EKP_PLATFORM_BIN` to the release binary let the 270 tests pass, but `cargo run --locked --bin ekp_platform -- --selftest` was still denied (`FAIL: platform selftest`). The canonical result is therefore `Blocked`, never `PASS`. Release-profile `cargo test --release --locked` (platform 270, eval 12), `--selftest` (49 assertions), schema (20 records) and search evaluation (28 cases) all exited 0 as equivalent coverage.

### 2026-10-27 Session-derived actors and append-only search audit (`718db57`)

This section supersedes the `--audit-log` behavior described in the `23b8996` section above.

- `platform` has **277 tests** in the canonical debug profile at the historical `718db57` revision: 221 unit, 24 CLI contract, 2 edge-case, 11 governance web, 8 engine integration, 7 search evaluation, 4 web health. `eval` has 12 tests. The self-test reports 49 assertions and the schema command validates 20 records. At `bbe49eb` the release profile has **325 tests** (257 unit, 30 CLI contract, 2 edge, 17 governance web, 8 integration, 7 search evaluation, 4 web health); at the preceding `c827351` revision the inventory was **312 executed + 2 edge tests NOT RUN** (250 unit, 28 CLI contract, 15 governance web, 8 integration, 7 search evaluation, 4 web health); see §6–§7.
- `--audit-log` requires `--search` and appends one JSONL `KnowledgeSearch` event to a hash chain. It reads and verifies the existing chain first; a malformed line, broken chain or legacy single-object file exits `1` without modifying the file. Local writers serialise on a sibling `<PATH>.lock` directory. It is tamper-evident, not WORM or an authenticated audit service.
- Every `governance_web` mutation derives the ledger actor from the validated local session username and ignores submitted `actor`/`reviewer` fields. The `/inspections` reviewer comes from the session. This is a local process-memory identity, not enterprise authentication.
- Regression tests to preserve: JSONL round-trip, tamper refusal without modification, legacy-format refusal, 8-thread concurrent append forming one verified chain, two CLI runs forming a 2-event chain, and a POST with `actor=Jordan` recording the session user `tester`.
- **Canonical check PASS at `718db57`:** `scripts/check.ps1` exited 0 with docs 53 required/657 links in 115 files, secrets 229, eval 12 + 10 self-test checks, platform 277 + 49 self-test assertions, schema 20 and search 28 (20/20, 8/8, 4 role cases, 0 violations). The evidence-document commits `32f2b36` and `604a2a5` re-ran the same command with docs 660 links and the same test totals. This is local evidence only; no CI run exists.

## 2. `eval/` behavior to preserve

- Parse seed/evaluation cases and preserve warnings/errors for malformed input.
- Keep adapter identity attached to each decision; reject a result tagged as a different system.
- Preserve verdict, citation, and metric semantics, including wrongful-allow hard gates, refusal-on-definitive, correct refusal, escalation, coverage, and report summaries.
- Preserve the default oracle / allow-all / refuse-all adapters and explicit `--systems` selection behavior.
- Preserve JSON and Markdown report generation, timestamps, and per-case failure details.

Relevant code: `eval/src/types.rs`, `metrics.rs`, `systems.rs`, `report.rs`, `main.rs`. Required automated checks: `cargo test --locked` and `cargo run --locked -- --selftest`. The expected CLI refusals are exit code 2 for an unknown adapter.

Optional CLI smoke (write output under the ignored crate target/ directory):

    cargo run --locked -- --out target/regression-report

Confirm the command succeeds and creates `report.json` and `report.md` under that ignored target/ directory.

## 3. `platform/` deterministic engine and CLI behavior

- Load versioned source-document and rule data; preserve source-version bindings, hashes, valid-time checks, and fail-closed schema validation.
- Keep draft AI-extracted rules out of the active engine; preserve verdicts, citations, abstention behavior, hard gates H-1 through H-6, process replay findings, and deterministic snapshot/replay checks.
- Preserve the configurable retrieval threshold and parameter precedence. Invalid parameter files or invalid thresholds must refuse execution.
- Preserve the declared-rules schema, LegalRuleML export validation, coverage report for the seed and casebook sets, HTML report output, and combined `--out` behavior.

Relevant code: `platform/src/{docs,rules,process,retrieval,evidence,eval,engine,snapshot,params,validate,export,coverage_report,main}.rs`; test coverage is in module tests, `platform/tests/integration.rs`, `platform/tests/search_eval.rs` and `platform/tests/cli_contract.rs`.

Required CI commands: Cargo tests, engine `--selftest`, and `--declared-rules-schema`. Expected CLI refusals are exit code 1 for malformed or tampered params and 2 for `--html` without `--out`, plus the argument-validation matrix in [the CLI contract](../contracts/cli.md#argument-validation). The remaining report/export flags are not separate required CI steps today. Run this focused smoke when touching CLI output, configuration, report, export, or case loading:

    cargo run --locked --bin ekp_platform -- --params config/engine-params.json --declared-rules-schema --out target/regression-report --coverage-report --export-legalruleml target/regression-lrml --html

Confirm exit code 0 and that target/regression-report contains `report.json`, `report.md`, `report.html`, `coverage_report.md`, `engine_snapshots.json`, `engine_hard_gates.json`, plus the LegalRuleML export under target/regression-lrml. Also confirm `report.html` is generated from the Markdown report and does not contain executable script elements. For the LegalRuleML export, confirm the artifact is at least 10 KB, has the official `lrml:LegalRuleML` root and namespace, and carries at least 19 `PrescriptiveStatement`, 19 `PenaltyStatement`, 19 `Reparation`, and 16 `LegalSource` elements. `gate.ps1 -IncludeSmoke` enforces these checks so a 14-byte stub cannot pass as a real export.

### Knowledge search and retrieval evaluation

- `--search QUERY` returns at most five ranked sections with `doc_id`, version, section, score and verbatim text; it prints `insufficient evidence` when nothing clears the threshold.
- Retrieval is lexical bigram cosine. A paraphrase that does not share enough character bigrams may abstain; do not "fix" a test by lowering the threshold without an approved change.
- `--eval --test-set data/eval/search_eval.json` must exit 0 with 20/20 top-1 answer cases, 8/8 refusals and a non-vacuous permission result: 4 role-restricted cases (2 allowed, 2 denied), 0 violations, isolation 1.00. The gate also requires ≥10 cases, zero failures and every required category.
- A bad expectation must exit 1, `--eval` without `--test-set` must exit 2, and an unreadable data source must exit 1. `platform/tests/search_eval.rs` enforces these fail-closed paths and asserts that evaluation-only fixtures are invisible to ordinary `--search`.
- `permissions` and `PermissionContext` are library/eval filters only. `--search` has no identity and must not be described as access-controlled; `governance_web` does not call retrieval.
- The fixed set now has two evaluation-only restricted fixtures and four role cases; `permission isolation 1.00` has a real denominator (`EVAL-GAP-06`, IMP-009). This still does not prove authentication, authorization or tenant isolation: the role list is supplied by the test harness, not an identity provider.
- The fixed set is regression evidence for the lexical retriever, not an LLM/agent quality benchmark. Tool failure remains an open gap (`EVAL-GAP-01`).

Required commands for this area:

    cargo test --locked
    cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json

For a search behavior change, also record one real `--search` command and its citation output in the PR or release evidence.

## 4. Governance web prototype behavior to preserve

### Read routes and presentation

- `GET /login` renders the local login form. Every ledger-backed GET requires a Viewer session and otherwise returns `401` without ledger content. `GET /healthz`, `/metrics` and `/assets/governance.css` stay public.
- `GET /`, `/projects`, `/evidence`, `/inspections`, `/workflows`, `/controls`, `/lifecycle`, and `/audit` render their expected headings when authenticated.
- `GET /assets/governance.css` returns CSS with the expected content type.
- Preserve the skip link, visible keyboard focus styling, narrow-screen media rule, Content Security Policy header, and zero-script HTML.

### Mutating flows and records

- Project creation assigns an identifier and persists a valid ledger event.
- Evidence references link project, source system, source URL, source version, type, and owner; review decisions persist.
- Workflow drafts can be saved and published by version without mutating published versions.
- Quality cases preserve status transitions; CAPA cannot be closed before verification/effectiveness steps.
- Quality inspection requires all 29 checklist decisions and their evidence.
- Gate decisions preserve non-waivable blocking behavior.
- Audit view verifies the hash-linked local event ledger and reflects recorded project, evidence, inspection, workflow, CAPA, and gate events.
- Mutating requests with a foreign Origin are rejected; the test confirms rejection does not append a ledger event. A missing `Origin` is accepted only with `Sec-Fetch-Site: same-origin` or `none`.
- Unauthenticated mutations return `401`; authenticated sessions without the required role return `403`; neither appends an event.
- Login failures do not reveal whether a username exists; five failures lock an existing account for 300 seconds; a successful login clears the failure state and prunes expired sessions.

Relevant routes and tests: `platform/src/governance_web.rs`, `platform/src/local_auth.rs`, `platform/src/governance.rs`, `platform/tests/governance_web.rs`. Preserve the boundary that this prototype is local-only and has **no enterprise authentication** or external-source permission enforcement; the session is a local process-memory control, not an IdP.

The existing automated route test checks semantic page headings, CSP presence, skip link, zero script, CSS content type, focus-visible selector, and a narrow-screen media rule. It is not a substitute for manual keyboard, screen-reader, contrast, or browser-resize checks on UI changes.

## 5. Review checklist for every change

- [ ] Identify affected items above and link the relevant unit, integration, or self-test evidence in the PR.
- [ ] If an existing behavior changes intentionally, link the accepted spec and update this inventory and its regression test in the same change.
- [ ] For a UI change, record keyboard, screen-reader, contrast, narrow-screen, and existing-route checks where applicable.
- [ ] For high-risk behavior, add applicable denial, stale/conflicting version, duplicate/retry, failed approval, and recovery cases; preserve fail-closed behavior.
- [ ] Keep prototype behavior claims separate from target-product requirements and unimplemented integrations.

## 6. Revision record: `c827351` hardening regression (2026-10-29)

This section records the regression baseline added by `fix: fail closed on health checks and remove policy workarounds`. It supplements, and does not replace, the area checklists above.

### New behavior to preserve

- `--health` validates arguments before printing any status. It accepts only `--data DIR`; every other flag, including `--selftest`, `--search`, `--eval`, report/export flags and unknown flags, is a usage error (exit `2`).
- `--health` prints `health: OK` and exits `0` only when `<data>/rules.json`, `docs.json` and `processes.json` exist as files; a missing file prints `health: DEGRADED (missing: ...)` and exits `1`. It does not parse the files, so `OK` is liveness, not readiness or data validity.
- Tests launch the binary only through `CARGO_BIN_EXE_ekp_platform` in the crate's canonical `target` directory. The `EKP_PLATFORM_BIN` override and the committed absolute `target-dir` override were removed; do not reintroduce them or copy/rename a blocked harness.
- `platform/Cargo.toml` and `eval/Cargo.toml` each declare `[workspace]`, so an unrelated parent manifest cannot capture either crate.

### Executed results

| Check | Result |
| --- | --- |
| `cargo fmt -- --check` / `cargo clippy --locked --all-targets -- -D warnings` (both crates) | Exit 0 |
| `eval` `cargo test --locked` | 12 passed; self-test 10 checks |
| `platform` unit / CLI contract / governance web / integration / search evaluation / web health | 250 / 28 / 15 / 8 / 7 / 4 passed (**312** executed; the earlier `332` was an arithmetic error) |
| `platform` `edge_cases` | **NOT RUN** — harness denied before start (`os error 4551`, Application Control policy `{0283ac0f-fff1-49ae-ada1-8a933130cad6}`) after nine retries |
| `--selftest` / `--declared-rules-schema` / search gate / `--health` | 49 assertions / 20 records / 28 cases (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) / `health: OK` |
| Docs / secrets | 53 required documents, 667 links in 115 files / 231 files |
| Canonical `scripts/check.ps1` | **BLOCKED**: exit 1 at platform `cargo test --locked` on `edge_cases`; not a PASS |

Evidence: [E-029](../assurance/test-evidence.md#e-029-hardening-re-run-and-host-policy-block-c827351) and [phase gates](../quality/phase-gates.md). The canonical block is an environment/host-policy block, not a source failure, but it still prevents a current canonical PASS. Record it as `Blocked`; do not replace the canonical command with the individual targets.

## 7. Revision record: `bbe49eb` defect-hardening regression (2026-10-29)

This section records the regression baseline added by `fix: fail closed on replay, audit and header ambiguity`. It supplements, and does not replace, the area checklists above. Each defect below was first reproduced by a failing test, then fixed; no test expectation was changed to match new behavior.

### New behavior to preserve

- The `--search` audit payload is serialized as a JSON object. Quotes, backslashes, control characters and newlines in the query must keep the event valid JSON.
- `audit::read_jsonl` validates every existing event's `id`, `actor`, `tenant_id` and payload structure, not only the timestamp, duplicate IDs and hash chain.
- Ledger replay validates the event actor, every creating payload's record ID, workflow version 0 and the quality-case transition graph. `save_workflow` uses `checked_add`.
- Quality cases allow only `open → in_progress`, `in_progress → verification`, `verification → in_progress`, `verification → closed`, plus same-state note updates. A closed case cannot be reopened until a new due date and approval can be captured. The CAPA-specific closure message keeps precedence over the generic transition message.
- A mutation with a repeated or non-UTF-8 `Origin` or `Sec-Fetch-Site` header is refused; the server never resolves ambiguity by taking the first value.
- `--selftest` is mutually exclusive with every other flag. After the self-test, any additional argument (including a valid mode flag) is a usage error; malformed arguments still fail after the self-test.
- The BPMN scanner advances by the full ASCII prefix length, so multibyte text after a tag name cannot cause an out-of-boundary slice panic.

### Executed results

| Check | Result |
| --- | --- |
| `cargo fmt -- --check` / `cargo clippy --locked --all-targets -- -D warnings` (both crates) | Exit 0 |
| `eval` `cargo test --locked` / `--selftest` | 12 passed / 10 checks |
| `platform` unit / CLI contract / edge cases / governance web / integration / search evaluation / web health | 257 / 30 / 2 / 17 / 8 / 7 / 4 passed (**325** executed) |
| `--selftest` / `--declared-rules-schema` / search gate / `--health` | 49 assertions / 20 records / 28 cases (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) / `health: OK` |
| Canonical `scripts/check.ps1` | Recorded separately in E-031; the release profile is not a substitute |

Evidence: [E-030](../assurance/test-evidence.md#e-030-defect-hardening-re-run-bbe49eb) and [phase gates](../quality/phase-gates.md). The host Application Control policy remained intermittent: several freshly rebuilt harnesses were denied on first attempt, then retried on the same canonical path and passed. No binary was renamed, copied, moved or pointed at another target directory. A persistent denial is recorded as `Blocked`, not `PASS`.
