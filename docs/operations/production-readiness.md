# Production readiness

- **Status:** Blocked
- **Owner:** Operations + Security (roles not yet assigned)
- **Last reviewed:** 2026-10-26
- **Phase gate:** [Phase B](../quality/phase-gates.md#phase-b-mvp--production-grade-gate)
- **Threat model:** [Local prototype threat model](../architecture/threat-model.md)
- **Decision register:** [governance/08](../governance/08-decision-register.md)

## Current answer

**The system is not production ready and must not be described as such.**

The repository contains a tested local prototype: a deterministic adjudication core and a loopback-only governance workbench with a local session login. It has no approved deployment target, enterprise identity, tenant isolation, off-host backup, operational telemetry, incident owner or service level. Those are not missing documentation paragraphs; they are unimplemented controls and unassigned accountabilities.

This page is the readiness checklist and evidence index. It does not replace the [Phase B gate](../quality/phase-gates.md), which is the approval record.

## Readiness dimensions

| ID | Dimension | Required before any production claim | Current state | Blocker |
| --- | --- | --- | --- | --- |
| PR-01 | Approved purpose and scope | Named product owner, approved use, excluded uses, data classification | Not approved | `DEC-01`, `DEC-13`; no product owner named |
| PR-02 | Deployment target and environment | Named environment, network boundary, TLS, host hardening, capacity, cost owner | None | No target approved; local process only |
| PR-03 | Identity and authorization | Enterprise IdP/MFA, role model, least privilege, joiner/mover/leaver, access review | Local-only session login with PBKDF2-HMAC-SHA256, CSPRNG tokens and Viewer/Editor/Approver checks is implemented in `governance_web`; it has no IdP, SSO, MFA, recovery, persistent identity store, access review or per-user administration. Retrieval's `PermissionContext` remains a library filter with no identity source | `DEC-12`; no IdP or security owner |
| PR-04 | Tenant and data isolation | Tenant-scoped storage and authorization tests, or an explicit single-tenant decision | None; `tenant` is library-only | `DEC-12`; no tenant model approved |
| PR-05 | Data governance | Inventory, lawful basis, retention/deletion, legal hold, residency, AI data boundary | Draft policy only | `DEC-13`; no data owner |
| PR-06 | Audit and retention | Tamper-evident audit store with retention, access control and external anchoring where required | Local JSONL hash chain; not WORM | `DEC-14`; no retention decision |
| PR-07 | Backup and disaster recovery | Off-host encrypted backup, approved RPO/RTO, scheduled restore tests, data-loss boundary | Local bundle restore drills PASS at `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f`, `3ee2ee2`, `41beaea`, `6ed236d` and `499e0d1` (`3ee2ee2` by manual fresh clone after the automatic TEMP drill was blocked; the later three by the automatic drill with restored `scripts/check.ps1` exit 0). The earlier `198d5f3`, `c77fec4`, `bc7671a` and `28b68cc` attempts remain **BLOCKED**. The `age` drill at `56a944f` encrypted and decrypted the bundle byte-identically, but its destination was local-simulated under `%TEMP%`. There is no off-host copy, approved RPO/RTO, scheduled restore test or key-custody owner. A draft proposal is recorded in [backup-policy-draft.md](backup-policy-draft.md), but it is not approved or implemented | `DEC-14`; [backup log](backup-log.md) has local evidence only; off-host/RPO/RTO remain Phase B |
| PR-08 | Observability and SLO | SLI/SLO/error budget, structured logs, metrics, traces, correlation IDs, health/readiness, alerting | Local JSON access log, request correlation id, aggregate Prometheus counters and a liveness `/healthz` exist and are tested. No SLO, error budget, traces, dashboards, alerting, readiness probe or on-call | `DEC-16`; no monitoring target |
| PR-09 | Reliability controls | Timeouts, bounded retries with backoff, idempotency, rate limits, circuit breaking, degradation and recovery drills | Per-request 15 s timeout, 64-way concurrency bound with `429`, 64 KiB form limit and graceful shutdown exist. No retry/backoff model, idempotency key, circuit breaker, cross-process lock or failure drill | `DEC-16`; no target environment |
| PR-10 | Performance and capacity | Approved workload, reproducible load/stress/soak tests, capacity and cost model | One local release benchmark (eval p50/p95 and loopback `/healthz` throughput) is recorded; no approved workload, soak test, bottleneck analysis or capacity model | `DEC-19`; no workload owner |
| PR-11 | Security assurance | Threat model review, SAST/dependency/secret scanning, penetration test, vulnerability disposition, SBOM where required | Local threat model and boundary tests; local secret scan, Clippy, `cargo audit` and CycloneDX SBOM artifacts; no independent review, SAST service or penetration test | `DEC-20`; no security owner |
| PR-12 | Supply chain | Pinned dependencies, audited lockfiles, pinned actions, provenance review for skills/tools | Lockfiles present; local `cargo audit` PASS on both lockfiles (0 vulnerabilities); CycloneDX SBOM is generated and hash-listed; actions are tag-pinned, not SHA-pinned; CI security job not run | Security owner and policy required; CI security job not run |
| PR-13 | Change and release governance | Required CI checks, branch protection, review rules, migration/compatibility policy, progressive rollout, rollback and post-release verification | Local checks and templates exist; no remote, branch protection or deployment pipeline | Remote/auth unavailable; `DEC-20` |
| PR-14 | Incident response | Severity model, on-call/escalation, notification path, evidence preservation, drill | Local checklist only | `DEC-16`; no incident owner |
| PR-15 | Support and handoff | Named service owner, support hours, runbooks, training and handoff sign-off | None | No operations owner |
| PR-16 | AI/agent controls (if enabled) | Provider and data approval, fixed evaluation, human takeover, cost/latency budget, versioning | AI not implemented; no provider approved. A deterministic 28-case lexical retrieval gate exists with 4 role-restricted permission cases, but it is not an LLM/agent evaluation; tool failure remains open (`EVAL-GAP-01`) | `DEC-15` |
| PR-17 | Legal and compliance | Applicable requirements identified, control mapping, non-waivable controls, certification scope if claimed | Not assessed | `DEC-09`, `DEC-13`; no compliance owner |

## Evidence standard

A dimension becomes `Ready` only when:

1. The accountable owner is named and accepts the residual risk.
2. The control is implemented, not merely documented.
3. The evidence names the environment, revision, date, command or drill, and result.
4. A failure or skipped check is recorded as `Not met` or `Blocked`, never converted to `PASS`.
5. Exceptions have a risk acceptor, compensating control, expiry and review date; non-waivable legal/security controls cannot be excepted by a normal approval.

## Minimum production gate

Before the first production release, all of the following must be true:

- PR-01 through PR-07 have named owners and linked evidence; PR-03 and PR-04 are implemented and tested, not planned.
- PR-08 and PR-09 have an approved SLO, dashboards, alerts and at least one executed failure/recovery drill.
- PR-10 has an approved workload and a capacity result within the approved budget.
- PR-11 has a completed security review with findings dispositioned; no unresolved critical or high finding remains without an accepted exception.
- PR-13 has required CI checks, branch protection, a rollback drill and a post-release verification record.
- PR-14 and PR-15 have named owners and an exercised escalation path.
- PR-16 is either explicitly out of scope or fully controlled; enabling AI without evaluation is prohibited.

## What would change this page

A production claim requires an approved deployment target, an assigned security owner and operations owner, the decisions listed above, and executed evidence. Until then, this page stays **Blocked**, and the Phase B gate stays **Blocked**.

Do not replace a blocker with a placeholder owner, a fabricated scan, a simulated restore or an aspirational paragraph. If a control cannot be evidenced, the correct status is `Blocked`.

## Related documents

- [Phase gates](../quality/phase-gates.md)
- [Threat model](../architecture/threat-model.md)
- [Security integration spec](../governance/07-security-integration-spec-v1.md)
- [Decision register](../governance/08-decision-register.md)
- [Local backup](local-backup.md), [backup log](backup-log.md), [off-host backup proposal](backup-policy-draft.md)
- [Pilot runbook and ownership proposal](pilot-runbook.md)
- [Secrets policy](secrets-policy.md)
- [Incident log](incident-log.md)
- [Release and retro](../development/release-and-retro.md)

## Update triggers

Update this page when an owner is assigned, a control is implemented, a decision is made, a drill or scan is executed, or the deployment boundary changes. Link the evidence; do not change the status without it.

---

## Production-Readiness Gap Analysis (2026-10-30)

> This section synchronizes all identified production-readiness gaps into the production-readiness document.
> Each gap includes a description, priority (P0/P1/P2/P3), impact, risk, recommended solution, and implementation direction.
> Gaps are classified into Phase A (locally executable) and Phase B (requires external decisions/resources).
> The system is **not** production-ready. No gap is marked `Ready` without linked evidence.

### Gap Summary by Dimension

| Dimension | P0 | P1 | P2 | P3 | Total | Phase A | Phase B |
|-----------|----|----|----|----|-------|--------|--------|
| Security | 3 | 3 | 1 | 0 | 7 | 2 | 5 |
| Performance | 0 | 1 | 3 | 0 | 4 | 3 | 1 |
| Scalability | 0 | 1 | 0 | 2 | 3 | 0 | 3 |
| Observability | 0 | 1 | 4 | 0 | 5 | 1 | 4 |
| Test Coverage | 0 | 0 | 3 | 2 | 5 | 2 | 3 |
| Deployment | 0 | 2 | 2 | 0 | 4 | 1 | 3 |
| Error Handling | 0 | 2 | 4 | 0 | 6 | 4 | 2 |
| Data Persistence | 0 | 2 | 1 | 0 | 3 | 2 | 1 |
| **Total** | **3** | **12** | **18** | **4** | **37** | **15** | **22** |

### Detailed Gap List

#### Security (7 gaps)

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| S-01 | No enterprise identity (IdP/SSO/MFA) | P0 | B | Cannot authenticate real users | Identity spoofing, unauthorized access | Integrate OIDC/OAuth2 (Keycloak/Azure AD), MFA via TOTP/WebAuthn, persistent identity store | `identity_provider.rs` module; `EKPI_IDP_URL` env var; OIDC redirect/callback handlers |
| S-02 | No tenant isolation | P0 | B | Multi-tenant data leakage | Cross-tenant data exposure, compliance violation | `tenant_id` on every ledger event; tenant-scoped queries; cross-tenant denial tests | Modify `governance.rs` Ledger struct; add `tenant_id` field; tenant filter in all handlers |
| S-03 | Ledger is not true WORM | P0 | B | Audit trail tampering possible | Regulatory non-compliance, evidence inadmissibility | OS-level append-only flag (Linux: chattr +a); external Merkle anchoring | Integrate `worm` module; `WormBackend` trait; dual-write (local + external) |
| S-04 | Weak local auth (predictable salt/token) | P1 | A | Offline dictionary attack possible | Password cracking, session hijacking | CSPRNG per-user random salt; CSPRNG 128-bit session tokens; configurable lockout | `User::new()` uses `rand::thread_rng()`; `generate_token()` uses `uuid::Uuid::new_v4()` |
| S-05 | Form size limit & DoS | P2 | A | Large forms rejected; 64 concurrent slots fillable | Service denial, poor UX | Per-route form size limits; IP-based rate limiting (30 req/min); request queue timeout | `MAX_FORM_BYTES` per-route; `RateLimiter` middleware with `governor` crate |
| S-06 | No CSRF protection | P1 | A | Cross-site request forgery | Unauthorized data modification | Double-submit cookie pattern; CSRF token in every form; middleware validation | `csrf_token()` generator; hidden input in forms; `csrf_guard` middleware |

#### Performance (4 gaps)

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| P-01 | Lexical search has no index | P1 | A | O(n*m) per query; linear degradation | Query latency growth; cannot handle large corpora | Inverted index (term → document positions); pre-computed document vectors; BM25 index | `InvertedIndex` struct in `retrieval.rs`; build at `Platform::load()` |
| P-02 | No JSON parsing cache | P2 | A | Repeated I/O and JSON parsing overhead | Slow startup on large datasets | Stream parsing (`serde_json::from_reader`); `mmap` + `simd-json`; `lazy_static` cache | `Platform::load()` with `Arc<DocDatabase>` shared instance |
| P-03 | Full ledger load into memory | P2 | A | Linear memory growth with ledger size | OOM; slow startup | Seek + incremental read; load only recent N events; archive old events | `Ledger::open(max_events)`; `Ledger::load_recent(n)` |
| P-04 | Single-threaded execution | P2 | B | Requests serialized; no multi-core utilization | High latency under load | `RwLock` for concurrent reads; `spawn_blocking` for engine; `tokio::sync::Mutex` | `SharedLedger = Arc<tokio::sync::RwLock<Ledger>>` |

#### Scalability (3 gaps)

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| SC-01 | No database (JSON/JSONL only) | P1 | B | No transactions, no concurrency control, no query optimization | Data corruption, concurrent write conflicts | SQLite (local) or PostgreSQL (production); `sqlx`/`diesel`; JSONL as export format | `storage` module with `StorageBackend` trait; `SqliteBackend` + `JsonlBackend` |
| SC-02 | No horizontal scaling | P3 | B+ | Single point of failure; cannot handle high traffic | Service unavailability | Reverse proxy + multi-instance + shared database; Redis session store | Phase B+ (after PR-02 deployment target approved) |
| SC-03 | No message queue | P3 | B+ | All operations synchronous; no retry mechanism | Request timeout; incomplete data processing | `tokio::sync::mpsc` local queue; RabbitMQ/NATS for production | Phase B+ |

#### Observability (5 gaps)

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| O-01 | No SLO/SLI/error budget | P1 | B | Cannot quantify service quality; no alerting | Undetected service degradation | Define SLI (availability, p99 latency, error rate); SLO (99.9% availability, p99 < 500ms); error budget calculation; Prometheus alerting rules | `docs/operations/slo.md`; Prometheus alert rules YAML |
| O-02 | No distributed tracing | P2 | B | Cannot trace requests across services; debugging difficult | Low debugging efficiency in production | OpenTelemetry integration; propagate `traceparent` header; Jaeger/Zipkin export | `request_context` generates `trace_id`; OTLP exporter |
| O-03 | No readiness probe | P2 | A | Kubernetes readiness probe cannot correctly determine service readiness | Traffic routed to unready instances | `/ready` endpoint checking ledger and auth store state | `governance_web.rs` `ready()` handler; returns 200/503 |
| O-04 | No dashboard or alerting | P2 | B | No visual monitoring; no automated alerts | Service anomalies go unnoticed | Grafana dashboard JSON; Prometheus alerting rules (error rate > 1%, p99 > 1s) | `docs/operations/grafana-dashboard.json`; `docs/operations/prometheus-alerts.yml` |
| O-05 | No fault injection or recovery drill | P2 | B | Cannot verify system behavior under failure | Unknown behavior during production failures | Chaos engineering tests (kill process, corrupt ledger, disk full); record RTO/RPO | `tests/fault_injection.rs`; `docs/operations/fault-injection-report.md` |

#### Test Coverage (5 gaps)

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| T-01 | No E2E tests | P2 | A | UI interaction flows unverified; CSS/HTML rendering untested | UI regressions undetected | Playwright E2E tests covering all pages and forms; HTML rendering validation | `tests/e2e/` directory; GitHub Actions CI Playwright step |
| T-02 | No load/stress tests | P2 | B | Cannot quantify system capacity; no performance regression detection | Unknown production performance | k6 load test scripts; wrk benchmarks; CI performance regression checks | `tests/load/k6-script.js`; `docs/operations/benchmark-report.md` |
| T-03 | No property-based tests | P3 | A | Boundary case coverage incomplete | Undiscovered boundary bugs | `proptest` property tests for `engine.decide()`, snapshot replay, ledger hash chain | `tests/property_tests.rs` with proptest strategies |
| T-04 | No fuzz testing | P3 | B | Potential panic/crash not discovered | DoS via malformed input | cargo-fuzz targets for JSON parsing, form parsing, search query | `fuzz/` directory with fuzz targets |
| T-05 | No web integration tests (real HTTP) | P2 | A | HTTP layer behavior unverified (headers, cookies, redirects) | Middleware configuration errors undetected | `axum::serve` + `reqwest` real HTTP tests | `tests/integration_web.rs` with real HTTP client |

#### Deployment (4 gaps)

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| D-01 | No deployment target | P1 | B | Cannot deploy to any environment | Deployment inconsistency, manual errors | Dockerfile (multi-stage build); systemd service file; deploy.sh script | `Dockerfile`, `deploy/systemd/ekpi-platform.service`, `scripts/deploy.sh` |
| D-02 | No CI/CD pipeline | P1 | B | No automated quality checks; no automated deployment | Manual errors; quality regressions | GitHub Actions CI (fmt, clippy, test, selftest, schema, secret scan); CD pipeline (build Docker image, push, deploy) | `.github/workflows/ci.yml` + `.github/workflows/cd.yml` |
| D-03 | No rollback mechanism | P2 | B | Cannot quickly rollback on deployment failure | Extended service unavailability | Git tag-based versions; `scripts/rollback.sh`; blue/green deployment | `scripts/rollback.sh`; `docs/operations/rollback-runbook.md` |
| D-04 | No configuration management | P2 | A | Deployment configuration inconsistent; misconfiguration hard to debug | Service failure due to misconfiguration | `config.toml` with schema validation; `EKPI_CONFIG_PATH` env var; startup validation | `config.rs` module with `Config` struct; `config.toml` template |

#### Error Handling (6 gaps)

| ID | Problem | Priority | Phase | Impact | Risk | Recommended Solution | Implementation Direction |
|----|---------|----------|-------|--------|------|----------------------|--------------------------|
| E-01 | No unified error type | P1 | A | Error tracking difficult; cannot programmatically handle errors | Low debugging efficiency; inconsistent error handling | `EkpiError` enum with `thiserror`; `Result<T, EkpiError>` across all modules; error codes + context + source chain | `errors.rs` with `EkpiError` enum; replace `String` errors |
| E-02 | No panic recovery | P1 | A | Process terminates on panic; no graceful degradation | DoS via panic-inducing input | `std::panic::set_hook` logging; `catch_unwind` in handler layer; return 500 instead of crash | `main.rs` panic hook; `governance_web.rs` middleware `catch_unwind` wrapper |
| E-03 | No retry/backoff | P2 | B | Transient failures cause request failures | Low service availability | `tokio-retry` or custom retry with exponential backoff; circuit breaker for external calls | `retry.rs` module with `RetryPolicy`; wrap ledger operations |
| E-04 | No graceful shutdown | P2 | A | In-flight requests interrupted on shutdown | Data inconsistency | `axum::serve` with graceful shutdown; drain in-flight requests; flush ledger | `tokio::select!` with shutdown signal; `server.with_graceful_shutdown()` |
| E-05 | No request timeout | P2 | A | Slow requests can occupy connections | DoS | `tower::timeout::TimeoutLayer` with 15s per-request timeout; per-route overrides | `.layer(TimeoutLayer::new(REQUEST_TIMEOUT))` in router |
| E-06 | No structured error response | P2 | A | Frontend/CLI cannot programmatically handle errors | Poor UX; difficult debugging | JSON error response with `{"error": {"code": "E001", "message": "...", "request_id": "..."}}` | `ErrorResponse` struct with `Serialize`; handlers return `Json<ErrorResponse>` |

#### Data Persistence (3 gaps)

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

**Phase A scope (P1/P2 locally executable):** 17 items
**Phase B scope (requires external decisions/resources):** 20 items

---

## Constraints (non-negotiable)

- `#![forbid(unsafe_code)]` on every Rust file; clippy `-D warnings` zero warnings; `cargo fmt -- --check` clean. Edition 2021.
- PowerShell boundary: multiple commands stitched with `;`, never `&&`/`|`/`&` in a single shell line.
- UTF-8 only for file writes; never PowerShell `Set-Content` without `-Encoding UTF8` (use Write tool / Python).
- No secrets, credentials, tokens, or real customer/org facts in the repo. `.env.example` only.
- No fabricated integrations, permissions, approvals, scan results, or compliance claims. Governance web prototype is local-only; no enterprise authentication.
- Library-only modules (`tenant`, `audit`, `worm`, `agents`, `formal_verification`, `runtime_verification`) must not be described as product capabilities until wired with spec + integration tests.
- A check result is evidence only when it names command, revision, date, and result.