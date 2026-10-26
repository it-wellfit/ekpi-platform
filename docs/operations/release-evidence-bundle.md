# EKPI MVP Release Evidence Bundle

> Evidence pack for the Phase A (0 → 1 MVP) release. Every claim in this pack
> names the command, revision, date, environment and result. A `PASS` here
> means the named check exited 0 on the named revision; it does **not** prove
> production readiness, tenant isolation, enterprise authentication or
> compliance certification.

## Release identity

- **Revision:** `7f6ab97` (primary), `5328787` (docs-only re-run)
- **Date:** 2026-10-26
- **Branch:** Local only; no remote/CI
- **Toolchain:** Rust stable, rust-toolchain.toml pinned
- **Scope:** Both independent crates (`eval/`, `platform/`), no root workspace

## Verification results

| Check | Command | Working directory | Result | Evidence |
|-------|---------|-------------------|--------|----------|
| Format | `cargo fmt -- --check` | `platform/` | PASS | `7f6ab97`, `5328787` |
| Format | `cargo fmt -- --check` | `eval/` | PASS | `7f6ab97`, `5328787` |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` | `platform/` | PASS, zero warnings | `7f6ab97`, `5328787` |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` | `eval/` | PASS, zero warnings | `7f6ab97`, `5328787` |
| Tests | `cargo test --locked` | `platform/` | PASS, 233 tests | `7f6ab97`, `5328787` |
| Tests | `cargo test --locked` | `eval/` | PASS, 9 tests | `7f6ab97`, `5328787` |
| Selftest | `cargo run --locked --bin ekp_platform -- --selftest` | `platform/` | PASS, 49 assertions | `7f6ab97` |
| Selftest | `cargo run --locked -- --selftest` | `eval/` | PASS, 10 checks | `7f6ab97` |
| Declared-rules schema | `cargo run --locked --bin ekp_platform -- --declared-rules-schema` | `platform/` | PASS, 20 records | `7f6ab97` |
| Search evaluation | `cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json` | `platform/` | PASS, 28 cases, 20/20 top-1, 8/8 refusals, 4 role-restricted | `7f6ab97` |
| Full gate | `gate.ps1 -IncludeSmoke` | Repo root | 13/13 PASS | `7f6ab97`, `5328787` |

## Governance web authentication (A-06)

`local_auth` (previously library-only) is now wired into `governance_web`:

- `AuthStore` state passed to `governance_web::router()`
- `GET /login` / `POST /login` — local password login with session cookie (`HttpOnly`, `Max-Age` from TTL)
- `POST /logout` — session termination and cookie clearance
- All mutating routes (`POST /projects`, `/evidence`, `/evidence/{id}/review`, `/inspections`, `/workflows`, `/workflows/{id}/{version}/publish`, `/controls`, `/controls/{id}/status`, `/lifecycle`) require a valid session cookie; unauthenticated or expired sessions receive `401 Unauthorized`
- GET routes (dashboards, audit, assets) remain publicly readable
- Pilot user `pilot`/`pilot123` (role: `Editor`) seeded at startup for local trial
- No enterprise IdP/MFA; local-only loopback (`127.0.0.1:3000`); not production identity

## Annotation κ (A-05)

`eval/src/annotation.rs` provides Cohen's κ computation for two annotators'
independent verdict labels on the same case set:

- `cohens_kappa(rater1, rater2) -> Option<f64>`
- `kappa_interpretation(kappa) -> &'static str` (Landis & Koch bands)
- 6 unit tests: perfect agreement (κ=1.0), chance agreement (κ=0.0), perfect disagreement (κ<0), mismatched lengths (None), empty input (None), single-label agreement (κ=1.0), moderate agreement (κ=0.5)

## Traceability register (A-02)

Updated `docs/traceability/register.md` with rows for:
- `A-06` Governance web auth (`local_auth` wiring into `governance_web`) — `Implemented`
- `A-05` Annotation κ — `Implemented`
- `S21` End-to-end audit — confirmed `Implemented`

No row is `Validated`: no real pilot session has occurred. A1 remains `Not met`.

## Backup and restore (A-08)

Local restore drills PASS at `283f83a`, `7f6ab97` and `5328787`. Fresh clones from bundles reach the tested revision and `scripts/check.ps1` exits 0. Bundles are local-only; no off-host copy, approved RPO/RTO or scheduled restore test exists.

## Artifacts

| Artifact | Location | Notes |
|----------|----------|-------|
| Canonical checks | `scripts/check.ps1`, `scripts/check.sh` | Both crates, docs, secrets |
| Stronger pre-release gate | `gate.ps1 -IncludeSmoke` | 13/13 including docs, secret scan, smoke |
| Backup scripts | `scripts/backup.ps1`, `scripts/backup.sh` | Git bundle + restore drill |
| Backup log | `docs/operations/backup-log.md` | Three PASS drills at `283f83a`, `7f6ab97`, `5328787` |
| Traceability register | `docs/traceability/register.md` | S1–S21, A-05, A-06 |
| Phase gates | `docs/quality/phase-gates.md` | A4/A8 `Met` (local); Phase A `Not approved` |
| Production readiness | `docs/operations/production-readiness.md` | `Blocked`; PR-01 through PR-17 unassigned/incomplete |
| Threat model | `docs/architecture/threat-model.md` | Local-only assessment |
| CI workflow | `.github/workflows/ci.yml` | Defined; not executed (no remote) |
| AI evaluation | `docs/quality/ai-evaluation.md` | Lexical 28-case gate; AI not in scope |

## Known limitations

1. **No real pilot session** (A1 `Not met`): no named product owner, operator, or user-validated workflow execution. Passing synthetic tests does not validate real user value.
2. **No CI/remote run** (A4): `gate.ps1` passed locally but CI has not executed; branch protection and deployment pipeline do not exist.
3. **No off-host backup** (B7): local bundle restore drills pass; no off-host encrypted copy or approved RPO/RTO.
4. **No enterprise identity** (B1): `local_auth` is wired into `governance_web` but is local-only; no IdP/MFA/SSO.
5. **No tenant isolation** (B4): `tenant` is library-only; not wired into served routes.
6. **No observability** (B2): no structured logs, metrics, traces, SLI/SLO or alerts.
7. **AI not in scope** (B5): no LLM/agent implementation; no provider/data policy approved.

## Sign-off

This evidence pack records what was verified on the named revisions. It does not constitute production approval, compliance certification or enterprise authorization. Phase A remains `Not approved` until A1 (real user value) and the pilot-scope rows of A2/A3/A5/A6/A10 are `Met` with named owners and linked evidence.
