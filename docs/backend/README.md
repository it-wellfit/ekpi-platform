# Backend

- **Status:** Active; describes the current local prototype
- **Owner:** Backend/Engineering
- **Last reviewed:** 2026-10-29
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
| Evidence retrieval | `retrieval` | Deterministic lexical retrieval with citations and conservative abstention; `PermissionContext` filters by role at library level | Evaluation corpus, hard gates, fixed search set (lexical; 4 role-restricted cases over 2 evaluation-only fixtures, not authentication or tenant isolation) |
| Knowledge search CLI | `main` (`--search`, `--eval`), `eval` | Ranks document sections, prints verified verbatim citations, and gates a fixed regression set | `platform/tests/search_eval.rs`, CLI run |
| Decision snapshot | `snapshot` | Records inputs and output for deterministic replay | Self-test, integration tests |
| Report/export | `export`, `coverage_report` | LegalRuleML, coverage, Markdown/HTML reports | Smoke gate and exporter tests |
| Governance ledger | `governance` | Append-only JSONL events for projects, evidence, workflows, quality cases, inspections and gate decisions; replay validates references and state transitions | `governance` and `governance_web` tests |
| Governance web | `governance_web`, `local_auth` | Server-rendered forms and read views; local session login, role checks, same-origin guard and security headers | Route/integration tests |

The following modules are compiled and unit-tested but not wired into any served HTTP request path: `api`, `tenant`, `audit`, `worm`, `agents`, `benchmark`, `formal_verification`, `runtime_verification`, `multimodal`, `ocr`, `pm4py_sidecar`, `reranker`, `retrieval_v2`, `vector_search`, `xes_import`, `bpmn`, `fulltext`, `knowledge_graph`. `audit` is used only by the CLI `--audit-log` path; it is not an HTTP audit service. `evidence` is not referenced outside its own module. `causal` is different: `engine.rs` calls `CausalAnalyzer::analyze` and stores `EngineOutcome.causal_results`, but no CLI, report or snapshot consumer reads that field, so it is not a user-visible capability. `tenant` is reached only by the `PermissionContext::from(&TenantContext)` conversion in `retrieval.rs`; no served path constructs a `TenantContext`. See the [architecture module table](../architecture/README.md#module-boundaries). Do not describe these as delivered capabilities.

## Persistence and data stores

| Store | Format | Location | Consistency boundary |
| --- | --- | --- | --- |
| Seed documents | JSON | `platform/data/docs.json` | Read-only fixture; loaded at process start |
| Seed rules | JSON | `platform/data/rules.json` | Validated against `design/rule-schema-v0.1.1.schema.json` |
| Seed processes | JSON | `platform/data/processes.json` | Read-only fixture |
| Engine parameters | JSON | `platform/config/engine-params.json` | Fail-closed validation; CLI override for threshold only |
| Search evaluation set | JSON | `platform/data/eval/search_eval.json` | Versioned fixed set with its own operating threshold; loaded and validated per run |
| Governance ledger | JSONL, one event per line | `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` or `EKP_GOVERNANCE_DATA` | Append + `sync_all` per event; SHA-256 chain; single-writer OS advisory lock held for the `Ledger` lifetime |
| Reports and exports | JSON/Markdown/HTML/XML | Operator-selected output directory | Regenerated; not transactional |

There is no relational database, ORM or migration engine. The ledger serialises writers with an OS advisory lock on a sibling `<PATH>.lock` file and holds that lock for the lifetime of the `Ledger`; a second process cannot open the same ledger path concurrently. The kernel releases the lock when the process exits, so a leftover lock file does not indicate a live writer. The lock is local to one host; it is not distributed coordination. Use a unique ledger path for smoke tests, as the release procedure requires.

## Transactions, idempotency and retry

- **CLI:** a run either produces a complete artifact set or exits non-zero. It does not partially commit a database transaction. Output files can be partially written if the process is killed; the next run overwrites the selected output directory.
- **Ledger:** each POST appends one event. The `Ledger` holds a single-writer OS advisory lock while open, reads and verifies the existing hash chain, and appends one complete line with `sync_all`; a truncated final line, blank line, duplicate ID, dangling reference or invalid workflow/CAPA/gate transition fails closed without appending. A duplicate form submission still creates a second event; there is no request idempotency key. Callers must avoid double submission.
- **Engine:** duplicate evidence references and duplicate rule references are deduplicated by deterministic checks; see the duplicate-related cases in the corpus.
- **Retry:** no automatic retry, backoff, circuit breaker or queue exists. A failed CLI command is an operator-visible failure, not a retryable job.
- **Concurrency:** the web server wraps the ledger in `Arc<Mutex<...>>` for in-process serialization, and `Ledger::open` holds an OS advisory lock for cross-process exclusion on the same host. This is not multi-host coordination or a transactional multi-event commit.

## Authentication and authorization boundary

The served web app has a **local-only session login**. At startup it can register one account from `EKPI_LOCAL_AUTH_USER`/`EKPI_LOCAL_AUTH_PASSWORD` with role `EKPI_LOCAL_AUTH_ROLE` (default `editor`). Passwords use PBKDF2-HMAC-SHA256 with 600,000 iterations and a random salt; session tokens come from the OS CSPRNG; sessions live in process memory for 3,600 seconds. Usernames with whitespace or control characters are rejected. Every authentication failure returns the same `invalid username or password` message; the lockout (5 failures in 300 seconds) is checked after the password-hash work and is not disclosed, and unknown usernames still pay a dummy hash cost. Every ledger-backed page requires a `viewer` session, ordinary creation requires `editor`, and review/publication/quality-status/lifecycle actions require `approver`. `/healthz`, `/metrics`, `/login` and the stylesheet are public. The cookie is `HttpOnly` and `SameSite=Strict`; `EKPI_COOKIE_SECURE=1` adds `Secure`. Duplicate or malformed session-cookie headers and duplicate or non-loopback `Host` headers are rejected.

This is **not** enterprise authentication or authorization. There is no IdP, SSO, MFA, account recovery, persistent user database, access review or joiner/mover/leaver process. The ledger records the validated local session username as the mutation actor, but that session is process-memory only and is not an enterprise-verified identity. `tenant` contains an in-memory tenant store, but it is not wired. The target identity, role, MFA and tenant requirements are in [governance/07](../governance/07-security-integration-spec-v1.md) and decisions `DEC-02`, `DEC-12` and `DEC-17`.

The `permissions` field on a document and `retrieval::PermissionContext` are a **library-level retrieval filter**, not an authorization decision. `--search` receives no user identity and applies no filter. The fixed `--eval` set now has four role-restricted cases and two evaluation-only restricted fixtures, with 2 allowed cases, 2 denied cases and 0 violations; this proves the filter's allow/deny behavior. The fixtures are merged only for the evaluation run and are not in `docs.json`, so `--search` cannot see them. Authentication, authorization policy and tenant isolation remain unimplemented (`PR-03`, `PR-04`, `S19`). See `EVAL-GAP-06` in [AI evaluation](../quality/ai-evaluation.md#known-gaps).

## API surface

There is no production JSON API. The only HTTP surface is the local HTML form app in [web routes](../contracts/web-routes.md).

`platform/src/api.rs` defines `/health`, `/api/v1/cases/:id/decide` and `/api/v1/cases/:id/snapshot`, but:

- it is declared in the library but never mounted by the `ekp_platform` CLI or the `governance_web` router;
- `adjudicate_case` returns a fixed `comply` with no citations or rule IDs;
- `get_snapshot` returns `not_found`.

It is a placeholder and **must not be exposed**. A real API requires an engine-backed implementation, authentication/authorization, error semantics and contract tests before it can be called a capability.

## Errors and exit codes

The CLI convention is documented in [CLI contract](../contracts/cli.md): `0` success, `1` fail-closed validation/IO/schema/export failure, `2` usage or argument error. The web app returns a rendered error page for ledger/validation failures, `400` for an oversized or malformed form body, `401` for an unauthenticated data page, `403` for an insufficient role or foreign-origin mutation, `429` for concurrency refusal and `504` for a request timeout.

Error messages must not include secrets or real personal data. The current app writes operator-visible messages; it does not have a structured error code registry. A stable error-code contract is a Phase B item.

## Observability

Current state: one JSON access-log line per request on stdout (request id, method, path, status, duration), an in-process aggregate `/metrics` endpoint for request/4xx/5xx/in-flight/concurrency-refusal/duration counters, a liveness-only `/healthz`, and a validated correlation id. There are no traces, dashboards, alerting, SLO/error budget, readiness probe or central log collection. The report records per-case latency and metrics, but that is evaluation output, not production telemetry.

Phase B requires SLI/SLO definitions, alerting and an incident path on top of these local signals; see [production readiness](../operations/production-readiness.md) and `DEC-16`.

## Migration and recovery

- Rule and data formats are versioned by files and schema (`v0.1`, `v0.1.1`). There is no database migration tool.
- Schema changes require a new schema version, validation tests, fixture updates and a compatibility note in [data formats](../contracts/data-formats.md).
- Recovery is a fresh clone from a git bundle plus a restore drill; see [local backup](../operations/local-backup.md) and [backup log](../operations/backup-log.md).
- The local ledger is not included in a git bundle. Back up its path separately if it contains data you need to retain.

## Known backend gaps

| ID | Gap | Impact | Tracking |
| --- | --- | --- | --- |
| BE-GAP-01 | `api.rs` is a false `comply` placeholder | Cannot be exposed; would be a safety defect if wired | [Improvement register](../reviews/README.md) |
| BE-GAP-02 | Local session login only; no enterprise identity or access review | Role checks are in-process and restart-scoped; no IdP/MFA/recovery/joiner-mover-leaver | Phase B, `DEC-12` |
| BE-GAP-03 | No tenant isolation | Cannot serve multiple organizations | Phase B, `DEC-12` |
| BE-GAP-04 | Ledger has a single-host OS advisory lock, but no transactional multi-event commit, idempotency key or multi-host coordination | A crash between events can leave partial workflow state; duplicate POSTs still create duplicate events | Phase B |
| BE-GAP-05 | Local logs/metrics only; no traces, SLO or alerting | No operational detection, budget or readiness evidence | Phase B, `DEC-16` |
| BE-GAP-06 | Library-only modules appear capable but are unwired | Misleading capability claims; maintenance cost | Wire with spec/tests or remove |
| BE-GAP-07 | No idempotency key for form posts | Double submission creates duplicate events | Phase B |
| BE-GAP-08 | `--search` has no identity or permission filter, and search is lexical only | Results are not access-controlled and paraphrased questions can abstain | Phase B identity work; new retrieval design needs an ADR and eval update |

## Update triggers

Update this document when a served surface, module, data store, authorization boundary, error convention, persistence design or observability capability changes. Update the relevant contract and traceability row in the same change.
