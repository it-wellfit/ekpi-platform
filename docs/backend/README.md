# Backend

- **Status:** Active; describes the current local prototype
- **Owner:** Backend/Engineering
- **Last reviewed:** 2026-10-26
- **Domain source of truth:** [governance/06 data and state model](../governance/06-data-and-state-model-v1.md)
- **Contracts:** [CLI](../contracts/cli.md), [web routes](../contracts/web-routes.md), [data formats](../contracts/data-formats.md)

## Runtime surfaces

| Surface | Entry point | Consumer | Status |
| --- | --- | --- | --- |
| `ekp_platform` CLI | `platform/src/main.rs` | Local operator, CI, release checks | Implemented |
| `governance_web` local server | `platform/src/bin/governance_web.rs` | Local operator using a browser on the same machine | Implemented for local pilot data |
| Rust library API | `platform/src/lib.rs` | `ekp_platform`, `governance_web`, tests | Implemented, mixed served/library-only |
| JSON HTTP API | `platform/src/api.rs` | None | **Placeholder, not served.** Do not treat as a contract |

The CLI and the web binary are separate processes. There is no background worker, queue consumer, scheduler or external service call in the served path.

## Domain responsibilities

The authoritative entities, states and invariants are in [governance/06](../governance/06-data-and-state-model-v1.md). The current implementation covers these areas:

| Responsibility | Module | What it actually does | Evidence |
| --- | --- | --- | --- |
| Versioned documents and anchors | `docs`, `engine` | Loads source versions, checks bindings/hashes, resolves clause anchors | Unit and integration tests |
| Active rules and schema | `rules`, `validate`, `params` | Loads active rules, rejects drafts, validates the declared schema, enforces fail-closed parameter invariants | `--declared-rules-schema`, self-test, tests |
| Process replay | `process`, `engine` | Replays events, detects forbidden pairs, deadlines, loops and missing state | Seed/edge cases, self-test |
| Evidence retrieval | `retrieval` | Deterministic lexical retrieval with citations and conservative abstention; `PermissionContext` filters by role at library level | Evaluation corpus, hard gates, fixed search set (lexical; the fixed set has no role-restricted fixture) |
| Knowledge search CLI | `main` (`--search`, `--eval`), `eval` | Ranks document sections, prints verified verbatim citations, and gates a fixed regression set | `platform/tests/search_eval.rs`, CLI run |
| Decision snapshot | `snapshot` | Records inputs and output for deterministic replay | Self-test, integration tests |
| Report/export | `export`, `coverage_report` | LegalRuleML, coverage, Markdown/HTML reports | Smoke gate and exporter tests |
| Governance ledger | `governance` | Append-only JSONL events for projects, evidence, workflows, quality cases, inspections and gate decisions | `governance_web` tests |
| Governance web | `governance_web` | Server-rendered forms and read views; same-origin guard and security headers | Route/integration tests |

The following modules are compiled and unit-tested but not wired into any served request path: `api`, `local_auth`, `tenant`, `audit`, `worm`, `agents`, `benchmark`, `formal_verification`, `runtime_verification`, `multimodal`, `ocr`, `pm4py_sidecar`, `reranker`, `retrieval_v2`, `vector_search`, `xes_import`, `bpmn`, `fulltext`, `knowledge_graph`. `evidence` is not referenced outside its own module. `causal` is different: `engine.rs` calls `CausalAnalyzer::analyze` and stores `EngineOutcome.causal_results`, but no CLI, report or snapshot consumer reads that field, so it is not a user-visible capability. `tenant` is reached only by the `PermissionContext::from(&TenantContext)` conversion in `retrieval.rs`; no served path constructs a `TenantContext`. See the [architecture module table](../architecture/README.md#module-boundaries). Do not describe these as delivered capabilities.

## Persistence and data stores

| Store | Format | Location | Consistency boundary |
| --- | --- | --- | --- |
| Seed documents | JSON | `platform/data/docs.json` | Read-only fixture; loaded at process start |
| Seed rules | JSON | `platform/data/rules.json` | Validated against `design/rule-schema-v0.1.1.schema.json` |
| Seed processes | JSON | `platform/data/processes.json` | Read-only fixture |
| Engine parameters | JSON | `platform/config/engine-params.json` | Fail-closed validation; CLI override for threshold only |
| Search evaluation set | JSON | `platform/data/eval/search_eval.json` | Versioned fixed set with its own operating threshold; loaded and validated per run |
| Governance ledger | JSONL, one event per line | `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` or `EKP_GOVERNANCE_DATA` | Append + `sync_data` per event; SHA-256 chain |
| Reports and exports | JSON/Markdown/HTML/XML | Operator-selected output directory | Regenerated; not transactional |

There is no relational database, ORM, migration engine or multi-process lock. Two processes writing the same ledger path at the same time are not coordinated. Use a unique ledger path for smoke tests, as the release procedure requires.

## Transactions, idempotency and retry

- **CLI:** a run either produces a complete artifact set or exits non-zero. It does not partially commit a database transaction. Output files can be partially written if the process is killed; the next run overwrites the selected output directory.
- **Ledger:** each POST appends one event. A duplicate form submission creates a second event; there is no request idempotency key. Callers must avoid double submission.
- **Engine:** duplicate evidence references and duplicate rule references are deduplicated by deterministic checks; see the duplicate-related cases in the corpus.
- **Retry:** no automatic retry, backoff, circuit breaker or queue exists. A failed CLI command is an operator-visible failure, not a retryable job.
- **Concurrency:** the web server wraps the ledger in `Arc<Mutex<...>>` for in-process serialization; this is not a cross-process lock.

## Authentication and authorization boundary

The served web app has **no authentication** and **no authorization**. It binds to `127.0.0.1`, uses a same-origin guard for mutations, and sets CSP/frame/referrer/nosniff headers. These are defense-in-depth for a local tool, not an identity control.

`local_auth` contains local password hashing, session expiry and a four-role enum, but it is not wired into `governance_web`. `tenant` contains an in-memory tenant store, but it is not wired. The target identity, role, MFA and tenant requirements are in [governance/07](../governance/07-security-integration-spec-v1.md) and decisions `DEC-02`, `DEC-12` and `DEC-17`.

The `permissions` field on a document and `retrieval::PermissionContext` are a **library-level retrieval filter**, not an authorization decision. `--search` receives no user identity and applies no filter. The evaluation schema can declare a role list, but the current fixed set declares no roles and every fixture document is unrestricted; its `permission isolation` metric is a no-leak regression check, not evidence that restricted documents are protected. See `EVAL-GAP-06` in [AI evaluation](../quality/ai-evaluation.md#known-gaps).

## API surface

There is no production JSON API. The only HTTP surface is the local HTML form app in [web routes](../contracts/web-routes.md).

`platform/src/api.rs` defines `/health`, `/api/v1/cases/:id/decide` and `/api/v1/cases/:id/snapshot`, but:

- it is declared in the library but never mounted by the `ekp_platform` CLI or the `governance_web` router;
- `adjudicate_case` returns a fixed `comply` with no citations or rule IDs;
- `get_snapshot` returns `not_found`.

It is a placeholder and **must not be exposed**. A real API requires an engine-backed implementation, authentication/authorization, error semantics and contract tests before it can be called a capability.

## Errors and exit codes

The CLI convention is documented in [CLI contract](../contracts/cli.md): `0` success, `1` fail-closed validation/IO/schema/export failure, `2` usage or argument error. The web app returns a rendered error page for ledger/validation failures and `403` for a foreign-origin mutation.

Error messages must not include secrets or real personal data. The current app writes operator-visible messages; it does not have a structured error code registry. A stable error-code contract is a Phase B item.

## Observability

Current state: local `println!`/`eprintln!` output and test assertions. There is no structured logging, metrics endpoint, tracing exporter, correlation ID, health/readiness endpoint for operations, or alerting. The report records per-case latency and metrics, but that is evaluation output, not production telemetry.

Phase B requires SLI/SLO definitions, structured logs, metrics, traces, alerting and an incident path; see [production readiness](../operations/production-readiness.md) and `DEC-16`.

## Migration and recovery

- Rule and data formats are versioned by files and schema (`v0.1`, `v0.1.1`). There is no database migration tool.
- Schema changes require a new schema version, validation tests, fixture updates and a compatibility note in [data formats](../contracts/data-formats.md).
- Recovery is a fresh clone from a git bundle plus a restore drill; see [local backup](../operations/local-backup.md) and [backup log](../operations/backup-log.md).
- The local ledger is not included in a git bundle. Back up its path separately if it contains data you need to retain.

## Known backend gaps

| ID | Gap | Impact | Tracking |
| --- | --- | --- | --- |
| BE-GAP-01 | `api.rs` is a false `comply` placeholder | Cannot be exposed; would be a safety defect if wired | [Improvement register](../reviews/README.md) |
| BE-GAP-02 | No authentication/authorization in the served app | Any local user can act; no role separation | Phase B, `DEC-12` |
| BE-GAP-03 | No tenant isolation | Cannot serve multiple organizations | Phase B, `DEC-12` |
| BE-GAP-04 | No cross-process lock or transactional store | Concurrent writers can corrupt or interleave ledger events | Phase B |
| BE-GAP-05 | No structured logs, metrics or traces | No operational detection or SLO evidence | Phase B, `DEC-16` |
| BE-GAP-06 | Library-only modules appear capable but are unwired | Misleading capability claims; maintenance cost | Wire with spec/tests or remove |
| BE-GAP-07 | No idempotency key for form posts | Double submission creates duplicate events | Phase B |
| BE-GAP-08 | `--search` has no identity or permission filter, and search is lexical only | Results are not access-controlled and paraphrased questions can abstain | Phase B identity work; new retrieval design needs an ADR and eval update |

## Update triggers

Update this document when a served surface, module, data store, authorization boundary, error convention, persistence design or observability capability changes. Update the relevant contract and traceability row in the same change.
