# Phase A and Phase B gates

- **Status:** Active; Phase A **not approved**, Phase B **blocked**
- **Owner:** Product + QA + Engineering
- **Last reviewed:** 2026-10-29
- **Definition of Done:** [Engineering workflow](../development/README.md#definition-of-done)
- **Production readiness:** [Operations readiness](../operations/production-readiness.md)

These gates separate “the prototype works” from “the product is ready for the intended use.” They are not a waterfall: every new feature re-enters the relevant rows. A gate is approved only by the named accountable role, with linked evidence and a review date.

## Status vocabulary

| Status | Meaning |
| --- | --- |
| `Met` | The required evidence exists, was executed on the named revision, and is linked |
| `Partial` | Some evidence exists, but the row’s full condition or scope is not met |
| `Not met` | The condition is known to fail or the required evidence is absent |
| `Not started` | No work or evidence exists yet |
| `Blocked` | Cannot proceed without an external decision, access, owner or environment |
| `N/A` | The condition does not apply to the approved scope; the reason is recorded |

## Phase A: 0 → 1 MVP gate

Phase A asks: can a real, authorized workflow be completed, reproduced and handed to another operator using this repository, without pretending the prototype is an enterprise service?

| ID | Dimension | Required condition | Evidence | Status (2026-10-30) | Blocker / next action |
| --- | --- | --- | --- | --- | --- |
| A1 | User value | At least one complete real workflow is executed by a representative user and the outcome is recorded | User/owner validation record, not only synthetic tests | `Not met` | Select a pilot workflow and owner (`DEC-01`); run it on authorized non-sensitive data |
| A2 | Functional correctness | Primary use cases and high-risk boundary cases pass for the approved scope | 67-case corpus, unit/integration tests, self-test, hard gates | `Partial` | Prototype scope passes; target acceptance matrix AT-01–AT-31 remains `NOT RUN` |
| A3 | Data correctness | Version binding, rule activity, state transitions, evidence sufficiency and error handling match the spec | Integration tests, schema check, snapshot replay | `Partial` | Add a real-workflow dataset and independent review; no DB transaction semantics exist |
| A4 | Engineering quality | Format, Clippy (warnings denied), unit/integration tests and self-tests pass | **`8ba4e0c` (2026-11-05, current): canonical `scripts/check.ps1` exited 0 with ALL CHECKS PASSED** — docs 53 required/696 links in 118 files, secret scan clean (a committed `DATABASE_URL` credential line found and removed), workspace `cargo test --locked` **445+ passed / 0 failed** (platform lib 273, CLI contract 31, edge 2, governance web 25, integration 8, integration web 11, property 18, search eval 7, web health 4; eval 12), self-test 49, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations, isolation 1.00); fmt/Clippy clean across the workspace including the new `apps/web`, `crates/domain`, `crates/application`, `crates/adapters` members. The prior revision `279c919` passes A4 on its own scope: canonical `scripts/check.ps1` exited 0 on two same-command runs (E-034), platform **345** tests. **Between them the tree did not build:** the intermediate hardening commits left `governance_web.rs` with 5 compile errors until the restoration series `dacdb48`→`746da73`→`8ba4e0c` — a state that must not be cited as a gate PASS | `Partial` — engineering quality itself is `Met` at `8ba4e0c`; the row stays `Partial` because the approved scope includes the target acceptance matrix (AT-01–AT-31) which remains `NOT RUN` |
| A5 | Security | Authentication, authorization, input validation, secret handling and data isolation are appropriate to the approved scope | Threat model, boundary tests, secret scan, manual review | `Partial` | `governance_web` requires a local session for every ledger-backed page, enforces Viewer/Editor/Approver roles, derives the ledger actor from the validated session username and ignores submitted actor/reviewer fields, stores PBKDF2-HMAC-SHA256 hashes with CSPRNG salts, uses `HttpOnly`/`SameSite=Strict` cookies, caps form bodies at 64 KiB and refuses unlocked cross-site mutations. Login failures are generic, lockout state is not disclosed, usernames with whitespace/control characters are rejected, unknown usernames still pay a dummy hash cost, duplicate Cookie header lines, malformed or duplicate session cookies and duplicate/non-loopback Host headers are refused. The ledger has a single-host OS writer lock; replay validates event actors, payload actors, record IDs, references, workflow versions and quality-case transitions, and a repeated or non-UTF-8 `Origin`/`Sec-Fetch-Site` header is refused. There is still no enterprise IdP, SSO, MFA, account recovery, persistent identity database, access review or tenant isolation; the session is process-memory only and does not satisfy `DEC-12`. |
| A6 | Maintainability | Install, configure, diagnose and hand off from the documentation | Install/config/troubleshooting docs; operator walkthrough | `Partial` | No named operator yet; run a handoff dry run |
| A7 | Traceability | Each in-scope requirement links to spec, code, test, release evidence and validation | [Traceability register](../traceability/register.md) and [governance/10](../governance/10-traceability-baseline.md) | `Partial` | Complete rows for the pilot scope; distinguish verified vs validated |
| A8 | Backup and restore | A backup exists from the first important version; a fresh clone passes the restore drill; the result is logged | [Backup log](../operations/backup-log.md): **`653eab4` (2026-11-06, S1–S6 series) PASS** — bundle `ekpi-platform-2026-11-06-204737.bundle` (SHA-256 `2460EA5F…9B0FCE`), `git bundle verify` okay, manual fresh clone reached HEAD `653eab4`, restored `scripts/check.ps1` **`CHECK_EXITCODE=0`** with `ALL CHECKS PASSED` observed (docs 742/130, secrets clean, workspace 491 tests, self-test 49, schema 20, search 28). Two script-driven drill runs were interrupted by shell timeouts and are **not** claimed; the identical-content `204407` bundle is recorded as superseded, not PASS. Prior series PASS: **`8ba4e0c` (2026-11-05)** — bundle `ekpi-platform-2026-11-05-161500.bundle` (SHA-256 `B5D0048B…D0FB4A`), automatic drill `RESTORE DRILL PASSED` (445+ tests). Earlier PASS bundles: `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f`, `3ee2ee2`, `41beaea`, `6ed236d`, `499e0d1` (bundle `ekpi-platform-2026-10-27-170750.bundle`). The hardening revisions `c827351`…`279c919` were never bundled; the broken-tree interval is covered by neither bundle nor gate and must not be cited. The earlier `28b68cc`, `bc7671a`, `c77fec4` and `198d5f3` attempts were **BLOCKED** by host Application Control and are not counted. | `Met` | Local scope: backup + fresh-clone restore + log row pass, including the S1–S6 series revision. Off-host encrypted copy, approved RPO/RTO, scheduled restore tests and a named backup owner remain Phase B (`B7`, `PR-07`, `DEC-14`). The `age` drill under `%TEMP%` is not an off-host copy. |
| A9 | AI quality (if AI is in scope) | Fixed evaluation set covers normal, ambiguous, adversarial, wrong-source, tool-failure and insufficient-context cases; changes rerun the applicable set | [AI evaluation](ai-evaluation.md); 67-case verdict corpus; 28-case retrieval set with 4 role-restricted permission cases | `Partial` | Verdict corpus covers 5 of 6 mandatory dimensions; the retrieval set adds the five deterministic dimensions plus permission coverage, but is lexical-only. Tool-failure is absent and no LLM/agent exists (`EVAL-GAP-01`); `EVAL-GAP-06` is closed as fixed-set permission coverage |
| A10 | Documentation and handoff | Scope, known limits, commands, recovery and evidence are current and readable by a new operator | Docs map, release template, runbooks | `Partial` | Add the pilot handoff record and close stale operational claims |
| A11 | Review and improvement | Major review findings are fixed; a retro is completed; accepted improvements are tracked | PR/review report; [reviews](../reviews/README.md) | `Partial` | No GitHub review/issue execution yet; local review and improvement register exist |

### Phase A decision

**Not approved.** The prototype is testable and locally runnable. The current code revision `8ba4e0c` (2026-11-05, quality restoration) passes A4 on the local working-tree scope: the canonical `scripts/check.ps1` exited 0 with ALL CHECKS PASSED (docs 53/696/118, secrets clean, workspace 445+ tests, self-test 49, schema 20, search 28) and its bundle passed the automatic restore drill; A1 remains `Not met` and the row stays `Partial` for the unexecuted target acceptance matrix. The predecessor `279c919` passed A4 on the local working-tree scope: two same-command canonical `scripts/check.ps1` runs exited 0 with docs 53/680/115, secrets 231, eval 12 + 10 self-test checks, platform 345 and all runtime checks green (E-034). The final evidence revision `6ed236d` passes A4 on the clean-clone scope: its bundle restored to a fresh clone, and the restored canonical `scripts/check.ps1` exited 0 (platform 270, eval 12, self-test 49, schema 20, search 28). The current evidence revision `499e0d1` also restores to a fresh clone and its restored canonical check exits 0 (docs 657/105, platform 277). The working tree also passed the canonical check at `4697aa1` (docs 53/655/115, platform 270), `718db57` (docs 53/657/115, platform 277), `32f2b36` and `604a2a5` (docs 53/660/115, platform 277); the earlier `41beaea` working-tree block is historical, and a later change is only current when its own canonical run passes. **The preceding hardening revision `c827351` is `Blocked` at the canonical check:** the `edge_cases` harness is denied by host Application Control (`os error 4551`), while the other six platform targets (250 unit, 28 CLI contract, 15 governance web, 8 integration, 7 search evaluation, 4 web health = 312) pass and the two edge tests are NOT RUN (E-029). The preceding `bbe49eb` revision passed 325 release-profile platform tests plus self-test 49, schema 20 and search 28, but its canonical run at docs HEAD `17df06a` is `Blocked` at the debug `ekp_platform.exe` for all 30 CLI contract spawns (E-031). The current `279c919` revision passed **345** release-profile platform tests (273 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health) and two same-command canonical runs exited 0 (E-034). The preceding `21e23d6` revision passed 308 executed release-profile platform tests (265 unit, 31 CLI contract, 8 integration, 4 web health); 27 tests were blocked at spawn across three harnesses (`governance_web` 18, `search_eval` 7, `edge_cases` 2), and its canonical entry point was `Blocked` on two same-command attempts at the debug lib harness (E-033). The preceding `b2d8afc` revision passed 329 executed release-profile platform tests (262 unit, 30 CLI contract, 18 governance web, 8 integration, 7 search evaluation, 4 web health); its 2 `edge_cases` tests were blocked at spawn by host policy, and its canonical entry point was `Blocked` on two same-command attempts (debug lib harness, then `cli_contract` harness; E-032). A green release profile is not a canonical PASS. There is no CI run. Local bundle restore drills passed at `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f`, `3ee2ee2`, `41beaea`, **`6ed236d`** and **`499e0d1`** (A8 `Met` for `499e0d1`, local-only; `c827351`, `bbe49eb`, `b2d8afc` and `21e23d6` are not yet bundled). A1 (real user value) is still `Not met`, and A7 (complete traceability for the pilot scope) plus the pilot-scope parts of A2/A3/A5/A6/A10 remain `Partial`. Passing A4/A8 alone is not an MVP acceptance, and neither row closes the Phase B off-host/RPO/RTO gaps.

Phase A may be approved only when every row in the approved scope is `Met` or has an explicitly accepted, time-bound exception with the accountable owner. A1 cannot be waived by a test result.

## Phase B: MVP → production-grade gate

Phase B asks: can this be operated safely for multiple users/organizations, at the approved scale, with evidence for failure, recovery, security and change?

| ID | Dimension | Required condition | Evidence required | Status (2026-10-26) | Blocking decision |
| --- | --- | --- | --- | --- | --- |
| B1 | Security and governance | Threat model owned; identity/MFA; least privilege; tenant isolation; secret management; audit/retention; vulnerability and supply-chain process | Threat model review, authZ/denial tests, tenant isolation tests, scan/SBOM results, access review | `Blocked` | No deployment target, IdP, data policy or security owner (`DEC-12`, `DEC-13`, `DEC-14`, `DEC-20`) |
| B2 | Reliability and observability | SLI/SLO/error budget; structured logs, metrics, traces, correlation IDs; timeouts, bounded retries, backoff, circuit breaker, rate limits; health checks; failure isolation, degradation and recovery drills | Dashboards, alert drills, fault injection, RTO/RPO and data-loss test | `Partial` | Local request timeout, concurrency bound, JSON access log, correlation id, aggregate `/metrics` and liveness `/healthz` exist and are tested. No SLO, alerting, traces, retry/backoff, circuit breaker, fault-injection drill or target environment (`DEC-16`) |
| B3 | Performance and scale | Reproducible benchmark/load/stress/soak tests; bottleneck analysis; capacity model; caching/batching/backpressure; scale decision based on cost/latency/throughput | Load scripts, capacity report, trend and cost model | `Partial` | One local release benchmark exists (eval p50/p95 and loopback `/healthz` throughput); no approved workload, sustained load, soak, bottleneck analysis or capacity model (`DEC-19`) |
| B4 | Delivery and change governance | CI required checks; dependency/security scans; migration and compatibility policy; versioned API/events; feature flags, progressive rollout, rollback, post-release verification; PR review and branch protection | CI runs, migration/rollback drill, deployment record, release checklist | `Blocked` | No remote, `gh` auth, branch protection or deployment target; CI has not run |
| B5 | AI and agent quality | Model/prompt/retrieval/tool/data versioning; fixed eval set; hallucination/citation/tool-success metrics; cost/latency budget; retry and human takeover; online/offline regression | Version comparison, error taxonomy, regression report, cost/latency evidence | `Blocked` | AI is not implemented and no provider/data policy is approved (`DEC-15`) |
| B6 | Operations and support | Named service/environment owner; support and escalation; start/stop/health; data locations/classification/retention; backup owner; incident procedure; maintenance and dependency ownership | Runbook, on-call/escalation record, incident drill, handoff sign-off | `Blocked` | No operating owner or service level (`DEC-16`) |
| B7 | Backup and disaster recovery | Off-host encrypted backup; defined RPO/RTO; scheduled restore tests; integrity check; data-loss boundary; incident path | Backup/restore log, RPO/RTO approval, off-host evidence | `Partial` | Local git bundle/restore drill only; no off-host copy or approved RPO/RTO (`DEC-14`) |
| B8 | Data governance | Classification, lawful basis, retention/deletion, legal hold, residency, access and AI data boundaries | Data inventory, retention schedule, deletion test, access review | `Blocked` | `DEC-13`; no data owner |

### Phase B decision

**Blocked.** B1/B4/B5/B6/B8 cannot start without decisions and owners that are not in this repository. B2/B3 require an approved target and workload. Do not describe the system as production-ready, secure, reliable, scalable or compliant until these rows have linked evidence.

## Gate evidence standard

For each row that claims `Met`:

1. Name the requirement/decision ID and the accountable owner.
2. Link the command, test, artifact, review or drill record, with revision/date/environment.
3. State the result and any residual limitation.
4. For a failed or skipped check, record `Not met`/`Blocked`, the defect/issue and the next action. Never mark it `PASS`.
5. For an exception, record the reason, impact, risk acceptor, compensating control, expiry and review date. Non-waivable legal/security controls cannot be waived by a normal exception.

## Sign-off

| Gate | Accountable roles | Evidence location |
| --- | --- | --- |
| Phase A | Product owner + QA + Engineering | This page, traceability register, review report |
| Phase B | Product owner + Security + Operations + Engineering | This page, production readiness, release record |

Named people are not recorded because no pilot owner, security owner or operations owner has been assigned. That absence is itself a blocker, not a documentation gap to fill with a placeholder name.

## Review cadence

- Re-evaluate the affected rows at every release and after any material incident.
- Re-run the full gate before a pilot expansion, a new tenant, a new connector, a model/provider change or a deployment change.
- Review this page at least quarterly, or when a decision in `governance/08` changes.
- A retro may add a control or a check, but it cannot lower a gate without an explicit, approved decision.

## Related documents

- [Engineering workflow and Definition of Done](../development/README.md#definition-of-done)
- [Quality gates and risk tiers](../development/quality-gates.md)
- [AI evaluation](ai-evaluation.md)
- [Production readiness](../operations/production-readiness.md)
- [Release and retro](../development/release-and-retro.md)
- [Decision register](../governance/08-decision-register.md)

---

## Production-Readiness Gap Analysis (2026-10-30)

> This section synchronizes all identified production-readiness gaps into the phase-gate document.
> Each gap includes a description, priority (P0/P1/P2/P3), impact, risk, recommended solution, and implementation direction.
> Gaps are classified into Phase A (locally executable) and Phase B (requires external decisions/resources).
> The system is **not** production-ready. No gap is marked `Met` without linked evidence.

### 1. Security

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| S-01 | No enterprise identity (IdP/SSO/MFA) | P0 | B | Cannot authenticate real users | Identity spoofing, unauthorized access | Integrate OIDC/OAuth2 (Keycloak/Azure AD), MFA via TOTP/WebAuthn, persistent identity store | `identity_provider.rs` module; `EKPI_IDP_URL` env var; OIDC redirect/callback handlers |
| S-02 | No tenant isolation | P0 | B | Multi-tenant data leakage | Cross-tenant data exposure, compliance violation | `tenant_id` on every ledger event; tenant-scoped queries; cross-tenant denial tests | Modify `governance.rs` Ledger struct; add `tenant_id` field; tenant filter in all handlers |
| S-03 | Ledger is not true WORM | P0 | B | Audit trail tampering possible | Regulatory non-compliance, evidence inadmissibility | OS-level append-only flag (Linux: chattr +a); external Merkle anchoring | Integrate `worm` module; `WormBackend` trait; dual-write (local + external) |
| S-04 | Weak local auth (predictable salt/token) | P1 | A | Offline dictionary attack possible | Password cracking, session hijacking | CSPRNG per-user random salt; CSPRNG 128-bit session tokens; configurable lockout | `User::new()` uses `rand::thread_rng()`; `generate_token()` uses `uuid::Uuid::new_v4()` |
| S-06 | No CSRF protection | P1 | A | Cross-site request forgery | Unauthorized data modification | Double-submit cookie pattern; CSRF token in every form; middleware validation | `csrf_token()` generator; hidden input in forms; `csrf_guard` middleware |

### 2. Performance

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| P-01 | Lexical search has no index | P1 | A | O(n*m) per query; linear degradation | Query latency growth; cannot handle large corpora | Inverted index (term → document positions); pre-computed document vectors; BM25 index | `InvertedIndex` struct in `retrieval.rs`; build at `Platform::load()` |
| P-02 | No JSON parsing cache | P2 | A | Repeated I/O and JSON parsing overhead | Slow startup on large datasets | Stream parsing (`serde_json::from_reader`); `mmap` + `simd-json`; `lazy_static` cache | `Platform::load()` with `Arc<DocDatabase>` shared instance |
| P-03 | Full ledger load into memory | P2 | A | Linear memory growth with ledger size | OOM; slow startup | Seek + incremental read; load only recent N events; archive old events | `Ledger::open(max_events)`; `Ledger::load_recent(n)` |
| P-04 | Single-threaded execution | P2 | B | Requests serialized; no multi-core utilization | High latency under load | `RwLock` for concurrent reads; `spawn_blocking` for engine; `tokio::sync::Mutex` | `SharedLedger = Arc<tokio::sync::RwLock<Ledger>>` |

### 3. Scalability

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| SC-01 | No database (JSON/JSONL only) | P1 | B | No transactions, no concurrency control, no query optimization | Data corruption, concurrent write conflicts | SQLite (local) or PostgreSQL (production); `sqlx`/`diesel`; JSONL as export format | `storage` module with `StorageBackend` trait; `SqliteBackend` + `JsonlBackend` |
| SC-02 | No horizontal scaling | P3 | B+ | Single point of failure; cannot handle high traffic | Service unavailability | Reverse proxy + multi-instance + shared database; Redis session store | Phase B+ (after PR-02 deployment target approved) |
| SC-03 | No message queue | P3 | B+ | All operations synchronous; no retry mechanism | Request timeout; incomplete data processing | `tokio::sync::mpsc` local queue; RabbitMQ/NATS for production | Phase B+ |

### 4. Observability

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| O-01 | No SLO/SLI/error budget | P1 | B | Cannot quantify service quality; no alerting | Undetected service degradation | Define SLI (availability, p99 latency, error rate); SLO (99.9% availability, p99 < 500ms); error budget calculation; Prometheus alerting rules | `docs/operations/slo.md`; Prometheus alert rules YAML |
| O-02 | No distributed tracing | P2 | B | Cannot trace requests across services; debugging difficult | Low debugging efficiency in production | OpenTelemetry integration; propagate `traceparent` header; Jaeger/Zipkin export | `request_context` generates `trace_id`; OTLP exporter |
| O-03 | No readiness probe | P2 | A | Kubernetes readiness probe cannot correctly determine service readiness | Traffic routed to unready instances | `/ready` endpoint checking ledger and auth store state | `governance_web.rs` `ready()` handler; returns 200/503 |
| O-04 | No dashboard or alerting | P2 | B | No visual monitoring; no automated alerts | Service anomalies go unnoticed | Grafana dashboard JSON; Prometheus alerting rules (error rate > 1%, p99 > 1s) | `docs/operations/grafana-dashboard.json`; `docs/operations/prometheus-alerts.yml` |
| O-05 | No fault injection or recovery drill | P2 | B | Cannot verify system behavior under failure | Unknown behavior during production failures | Chaos engineering tests (kill process, corrupt ledger, disk full); record RTO/RPO | `tests/fault_injection.rs`; `docs/operations/fault-injection-report.md` |

### 5. Test Coverage

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| T-01 | No E2E tests | P2 | A | UI interaction flows unverified; CSS/HTML rendering untested | UI regressions undetected | Playwright E2E tests covering all pages and forms; HTML rendering validation | `tests/e2e/` directory; GitHub Actions CI Playwright step |
| T-02 | No load/stress tests | P2 | B | Cannot quantify system capacity; no performance regression detection | Unknown production performance | k6 load test scripts; wrk benchmarks; CI performance regression checks | `tests/load/k6-script.js`; `docs/operations/benchmark-report.md` |
| T-03 | No property-based tests | P3 | A | Boundary case coverage incomplete | Undiscovered boundary bugs | `proptest` property tests for `engine.decide()`, snapshot replay, ledger hash chain | `tests/property_tests.rs` with proptest strategies |
| T-04 | No fuzz testing | P3 | B | Potential panic/crash not discovered | DoS via malformed input | cargo-fuzz targets for JSON parsing, form parsing, search query | `fuzz/` directory with fuzz targets |
| T-05 | No web integration tests (real HTTP) | P2 | A | HTTP layer behavior unverified (headers, cookies, redirects) | Middleware configuration errors undetected | `axum::serve` + `reqwest` real HTTP tests | `tests/integration_web.rs` with real HTTP client |

### 6. Deployment Process

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| D-01 | No deployment target | P1 | B | Cannot deploy to any environment | Deployment inconsistency, manual errors | Dockerfile (multi-stage build); systemd service file; deploy.sh script | `Dockerfile`, `deploy/systemd/ekpi-platform.service`, `scripts/deploy.sh` |
| D-02 | No CI/CD pipeline | P1 | B | No automated quality checks; no automated deployment | Manual errors; quality regressions | GitHub Actions CI (fmt, clippy, test, selftest, schema, secret scan); CD pipeline (build Docker image, push, deploy) | `.github/workflows/ci.yml` + `.github/workflows/cd.yml` |
| D-03 | No rollback mechanism | P2 | B | Cannot quickly rollback on deployment failure | Extended service unavailability | Git tag-based versions; `scripts/rollback.sh`; blue/green deployment | `scripts/rollback.sh`; `docs/operations/rollback-runbook.md` |
| D-04 | No configuration management | P2 | A | Deployment configuration inconsistent; misconfiguration hard to debug | Service failure due to misconfiguration | `config.toml` with schema validation; `EKPI_CONFIG_PATH` env var; startup validation | `config.rs` module with `Config` struct; `config.toml` template |

### 7. Error Handling

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| E-01 | No unified error type | P1 | A | Error tracking difficult; cannot programmatically handle errors | Low debugging efficiency; inconsistent error handling | `EkpiError` enum with `thiserror`; `Result<T, EkpiError>` across all modules; error codes + context + source chain | `errors.rs` with `EkpiError` enum; replace `String` errors |
| E-02 | No panic recovery | P1 | A | Process terminates on panic; no graceful degradation | DoS via panic-inducing input | `std::panic::set_hook` logging; `catch_unwind` in handler layer; return 500 instead of crash | `main.rs` panic hook; `governance_web.rs` middleware `catch_unwind` wrapper |
| E-03 | No retry/backoff | P2 | B | Transient failures cause request failures | Low service availability | `tokio-retry` or custom retry with exponential backoff; circuit breaker for external calls | `retry.rs` module with `RetryPolicy`; wrap ledger operations |
| E-04 | No graceful shutdown | P2 | A | In-flight requests interrupted on shutdown | Data inconsistency | `axum::serve` with graceful shutdown; drain in-flight requests; flush ledger | `tokio::select!` with shutdown signal; `server.with_graceful_shutdown()` |
| E-05 | No request timeout | P2 | A | Slow requests can occupy connections | DoS | `tower::timeout::TimeoutLayer` with 15s per-request timeout; per-route overrides | `.layer(TimeoutLayer::new(REQUEST_TIMEOUT))` in router |
| E-06 | No structured error response | P2 | A | Frontend/CLI cannot programmatically handle errors | Poor UX; difficult debugging | JSON error response with `{"error": {"code": "E001", "message": "...", "request_id": "..."}}` | `ErrorResponse` struct with `Serialize`; handlers return `Json<ErrorResponse>` |

### 8. Data Persistence

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| DP-01 | JSONL Ledger has no transactions | P1 | A | Partial writes cause data corruption | Data loss or inconsistency | Write-to-temp + rename (atomic); fsync after write; checksum verification on read | `Ledger::append_event()` uses `write_to_temp()` + `rename()`; `fsync()` |
| DP-02 | No data migration | P2 | B | Schema changes break existing data | Upgrade failure | Schema version in JSON header; migration functions; `--migrate` CLI flag | `migrations/` directory with versioned migration functions |
| DP-03 | No automated backup | P1 | A | Backup depends on manual execution; may be forgotten | Data loss | Windows Task Scheduler / cron scheduling; encrypted backup; off-site copy (rclone to S3/NAS) | `scripts/schedule-backup.ps1`; `docs/operations/backup-schedule.md` |

---

## Priority Ranking Matrix

### P0 (Critical) — Immediate attention
| ID | Problem | Impact | Difficulty | Recommended Timeline |
|----|---------|--------|------------|----------------------|
| S-01 | No enterprise identity | Security | High | Phase B |
| S-02 | No tenant isolation | Security | High | Phase B |
| S-03 | Ledger not true WORM | Security/Compliance | Medium | Phase B |

### P1 (High) — Short-term
| ID | Problem | Impact | Difficulty | Recommended Timeline |
|----|---------|--------|------------|----------------------|
| S-04 | Weak local auth | Security | Medium | Phase A |
| S-06 | No CSRF protection | Security | Medium | Phase A |
| P-01 | No search index | Performance | Medium | Phase A |
| SC-01 | No database | Scalability | High | Phase B |
| E-01 | No unified error type | Error handling | Medium | Phase A |
| E-02 | No panic recovery | Reliability | Medium | Phase A |
| DP-01 | Ledger no transactions | Data integrity | Medium | Phase A |
| DP-03 | No automated backup | Data persistence | Low | Phase A |
| O-01 | No SLO/SLI/error budget | Observability | Medium | Phase B |
| D-01 | No deployment target | Deployment | High | Phase B |
| D-02 | No CI/CD pipeline | Deployment | Medium | Phase B |

### P2 (Medium) — Medium-term
| ID | Problem | Impact | Difficulty | Recommended Timeline |
|----|---------|--------|------------|----------------------|
| S-05 | Form size limit & DoS | Security | Low | Phase A |
| P-02 | No JSON parsing cache | Performance | Low | Phase A |
| P-03 | Full ledger load | Performance | Medium | Phase A |
| P-04 | Single-threaded | Performance | Medium | Phase B |
| O-02 | No distributed tracing | Observability | Medium | Phase B |
| O-03 | No readiness probe | Observability | Low | Phase A |
| O-04 | No dashboard or alerting | Observability | Medium | Phase B |
| O-05 | No fault injection | Reliability | Medium | Phase B |
| T-01 | No E2E tests | Testing | Medium | Phase A |
| T-02 | No load/stress tests | Testing | Medium | Phase B |
| T-05 | No web integration tests | Testing | Medium | Phase A |
| D-03 | No rollback mechanism | Deployment | Medium | Phase B |
| D-04 | No configuration management | Deployment | Low | Phase A |
| E-03 | No retry/backoff | Reliability | Medium | Phase B |
| E-04 | No graceful shutdown | Reliability | Medium | Phase A |
| E-05 | No request timeout | Reliability | Low | Phase A |
| E-06 | No structured error response | Error handling | Medium | Phase A |
| DP-02 | No data migration | Data persistence | Medium | Phase B |

### P3 (Low) — Long-term
| ID | Problem | Impact | Difficulty | Recommended Timeline |
|----|---------|--------|------------|----------------------|
| SC-02 | No horizontal scaling | Scalability | High | Phase B+ |
| SC-03 | No message queue | Scalability | High | Phase B+ |
| T-03 | No property-based tests | Testing | Low | Phase A |
| T-04 | No fuzz testing | Testing | Medium | Phase B |

---

## Implementation Priority Recommendation

### Phase A (0 → 1 MVP) — Start immediately
1. S-04 Weak local auth (CSPRNG salt/token)
2. S-06 CSRF protection (double-submit cookie)
3. E-01 Unified error type (thiserror + EkpiError)
4. E-02 Panic recovery (panic hook + catch_unwind)
5. DP-01 Ledger atomic writes (write-to-temp + rename)
6. O-03 Readiness probe (/ready endpoint)
7. D-04 Configuration management (config.toml + validation)
8. E-05 Request timeout (TimeoutLayer)
9. E-06 Structured error response (JSON error body)
10. P-01 Lexical search index (inverted index)
11. P-02 JSON parsing cache
12. P-03 Ledger incremental load
13. T-01 E2E tests (Playwright)
14. T-05 Web integration tests (real HTTP)
15. T-03 Property-based tests (proptest)
16. DP-03 Backup automation (Task Scheduler/cron)

### Phase B (MVP → Production) — Awaiting decisions
1. S-01/S-02/S-03 Enterprise identity/tenant/WORM (requires IdP + data policy)
2. SC-01 Database (requires deployment target)
3. D-01/D-02 Deployment target + CI/CD (requires remote + branch protection)
4. O-01/O-04 SLO + dashboard (requires monitoring infrastructure)
5. P-04 Multi-threading (requires load model)
6. O-02/O-05 Tracing + fault injection (requires target environment)
7. T-02 Load testing (requires approved workload)
8. D-03 Rollback mechanism (requires deployment pipeline)
9. E-03 Retry/backoff (requires external calls)
10. E-04 Graceful shutdown (requires deployment environment)
11. DP-02 Data migration (requires schema versioning)
12. SC-02/SC-03 Horizontal scaling/message queue (Phase B+)

---

## Statistical Summary

| Dimension | P0 | P1 | P2 | P3 | Total |
|-----------|----|----|----|----|-------|
| Security | 3 | 3 | 1 | 0 | 7 |
| Performance | 0 | 1 | 3 | 0 | 4 |
| Scalability | 0 | 1 | 0 | 2 | 3 |
| Observability | 0 | 1 | 4 | 0 | 5 |
| Test Coverage | 0 | 0 | 3 | 2 | 5 |
| Deployment | 0 | 2 | 2 | 0 | 4 |
| Error Handling | 0 | 2 | 4 | 0 | 6 |
| Data Persistence | 0 | 2 | 1 | 0 | 3 |
| **Total** | **3** | **12** | **18** | **4** | **37** |

**Phase A scope (P1/P2 locally executable):** 21 items
**Phase B scope (requires external decisions/resources):** 16 items

---

## Phase A vs Phase B Classification

### Phase A (locally executable, no external dependencies)
S-04, S-06, P-01, P-02, P-03, E-01, E-02, E-04, E-05, E-06, DP-01, DP-03, O-03, D-04, T-01, T-03, T-05
(17 items, all P1/P2/P3 except P0 items which require external resources)

### Phase B (requires external decisions/resources)
S-01, S-02, S-03, SC-01, SC-02, SC-03, O-01, O-02, O-04, O-05, T-02, T-04, D-01, D-02, D-03, E-03, DP-02, P-04
(18 items, all require external owners, IdP, deployment target, monitoring infrastructure, or approved workload)

---

## Constraints (non-negotiable)

- `#![forbid(unsafe_code)]` on every Rust file; clippy `-D warnings` zero warnings; `cargo fmt -- --check` clean. Edition 2021.
- PowerShell boundary: multiple commands stitched with `;`, never `&&`/`|`/`&` in a single shell line.
- UTF-8 only for file writes; never PowerShell `Set-Content` without `-Encoding UTF8` (use Write tool / Python).
- No secrets, credentials, tokens, or real customer/org facts in the repo. `.env.example` only.
- No fabricated integrations, permissions, approvals, scan results, or compliance claims. Governance web prototype is local-only; no enterprise authentication.
- Library-only modules (`tenant`, `audit`, `worm`, `agents`, `formal_verification`, `runtime_verification`) must not be described as product capabilities until wired with spec + integration tests.
- A check result is evidence only when it names command, revision, date, and result.

## Open decisions (require external input)

- Product owner name (`DEC-01`); security owner (`DEC-12`); operations owner (`DEC-16`); data owner (`DEC-13`); deployment target (`PR-02`).
- Single-tenant vs multi-tenant decision (`DEC-12`); off-host backup location/RPO/RTO (`DEC-14`); AI in-scope or out-of-scope (`DEC-15`); compliance scope (`DEC-09`).
- Remote collaboration upgrade only when criteria in `docs/remote-collaboration-criteria.md` are all met.

## Out of scope

- Enterprise SSO/MFA (IdP not approved).
- Live Microsoft 365/Google connectors; Kubernetes; message queue; relational DB; public network exposure; 24×7 support; regulatory certification.
- AI approval or autonomous gate release.
- Converting `local_auth` into an enterprise identity system.
