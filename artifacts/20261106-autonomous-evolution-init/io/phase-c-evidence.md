# Phase C — ekpi-platform Phase A Evidence & API Contract Design

**Session:** `20261106-autonomous-evolution-init`
**Date:** 2026-11-06
**Revision:** a268319 (working tree; docs/traceability/register.md modified)

---

## 1. Git & Remote Status

| Item | Value |
|------|-------|
| Branch | main |
| Remote | origin = https://github.com/it-wellfit/ekpi-platform.git (fetch/push) |
| Status | Ahead of origin/main by 1 commit (a268319) |
| Working tree | 1 modified file: docs/traceability/register.md (unstaged) |
| Latest commit | a268319 feat(S4/S5): decisions export v2, wired read API, SLO draft, fresh benchmark baseline |

**Conclusion:** Git remote already exists and is connected. No action needed (plan item #10 resolved).

---

## 2. Baseline Four Gates (executed 2026-11-06)

| Gate | Command | Exit code | Result |
|------|---------|-----------|--------|
| Format | `cargo fmt -- --check` | 0 | PASS |
| Clippy | `cargo clippy --all-targets -- -D warnings` | 0 | PASS (0 warnings) |
| Tests | `cargo test --locked` | 0 | PASS (all tests passed; 0 failed / 0 ignored) |
| Audit | `cargo audit` | 0 | PASS (1 pre-accepted warning: RUSTSEC-2026-0374 basic-toml unmaintained) |

**Conclusion:** All four gates EXIT=0 on revision a268319. Baseline confirmed (plan item #1 resolved).

---

## 3. Phase A Evidence Gap Summary

Phase A is **not approved**. The following rows must reach `Met` (or have an explicitly accepted, time-bound exception with accountable owner) before Phase A can be approved:

| ID | Dimension | Current status | Decisive gap | Owner needed |
|----|-----------|---------------|--------------|--------------|
| A1 | User value | NOT MET | No named pilot owner or representative user; no real workflow record | Business pilot owner (UNASSIGNED) |
| A2 | Functional correctness | PARTIAL | Target acceptance cases AT-01–AT-31 are NOT RUN | QA owner (UNASSIGNED) |
| A3 | Data correctness | PARTIAL | No real-workflow dataset; no independent review; no DB transaction semantics | Data owner (UNASSIGNED) |
| A4 | Engineering quality | MET (at 279c919) | Canonical check passes; but working-tree revision a268319 not yet verified | Engineering + QA |
| A5 | Security | PARTIAL | No enterprise IdP/MFA/tenant isolation; local session only | Security owner (UNASSIGNED) |
| A6 | Maintainability | PARTIAL | No named operator; no independent handoff dry run | Operations owner (UNASSIGNED) |
| A7 | Traceability | PARTIAL | No pilot run ID; no validated row | QA + Product (UNASSIGNED) |
| A8 | Backup and restore | MET (local) | No off-host copy; no approved RPO/RTO | Backup owner (UNASSIGNED) |
| A9 | AI quality | PARTIAL | Mock LLM only; no AI/agent evaluation; no tool-failure coverage | AI system owner (UNASSIGNED) |
| A10 | Documentation and handoff | PARTIAL | No pilot handoff record | Engineering + Operations |
| A11 | Review and improvement | PARTIAL | Self-review only; no independent reviewer; no second annotator | Independent reviewer (UNASSIGNED) |

**Key blocker:** A1 (user value) cannot be waived by any test result. It requires a named pilot owner and a real workflow execution record.

**External blockers (all UNASSIGNED):** Pilot owner (B-01), enterprise identity/MFA/tenant model (B-03), security/operations owners (B-07), data owner/data policy (B-04), AI provider/data policy (B-05), off-host backup/key custody/RPO/RTO (B-06), independent reviewer/second annotator (B-08), host Application Control policy exception (B-11), remote/CI/branch protection (B-02).

---

## 4. Production API Contract Design (design only, no implementation)

### 4.1 Current state

`platform/src/api.rs` is an **unwired placeholder** that returns a fixed `"comply"` verdict. Per `AGENTS.md` and `docs/contracts/README.md`:
- It is **not** a contract and must not be exposed or documented as stable.
- No OpenAPI/Swagger file exists (correctly — "Do not create empty OpenAPI or event-schema files to look complete").
- No production HTTP API exists.

### 4.2 Design principles (for future Phase B approval)

If a production API is approved (requires Phase B decisions: IdP, data policy, deployment target, security owner), the contract should follow:

1. **Authentication:** OIDC authorization-code + PKCE with approved IdP; server-side session store; claims-to-role mapping. No local process-memory auth in production.
2. **Authorization:** Deny-by-default; role + tenant + resource scope checks on every endpoint.
3. **Versioning:** URL path versioning (`/api/v1/...`); breaking changes require ADR + migration note + updated consumers.
4. **Error format:** Structured JSON: `{"error": {"code": "E001", "message": "...", "request_id": "..."}}`.
5. **Audit:** Every mutating endpoint produces an audit event (event ID, actor, UTC timestamp, before/after hash).
6. **Rate limiting:** Per-client rate limits with 429 + Retry-After.
7. **Idempotency:** Mutation endpoints accept an idempotency key; duplicate requests return the original response.

### 4.3 Candidate endpoints (design sketch, not implemented)

| Method | Path | Purpose | Auth | Status |
|--------|------|---------|------|--------|
| GET | `/api/v1/health` | Liveness + readiness | None | Design only |
| GET | `/api/v1/cases` | List cases (paginated) | Bearer/OIDC | Design only |
| GET | `/api/v1/cases/:id` | Get case detail | Bearer/OIDC | Design only |
| POST | `/api/v1/cases/:id/decide` | Adjudicate case | Bearer/OIDC + CSRF | Design only |
| GET | `/api/v1/cases/:id/snapshot` | Get case snapshot | Bearer/OIDC | Design only |
| GET | `/api/v1/rules` | List declared rules | Bearer/OIDC | Design only |
| GET | `/api/v1/rules/:id` | Get rule detail | Bearer/OIDC | Design only |
| GET | `/api/v1/search` | Search documents | Bearer/OIDC | Design only |
| GET | `/api/v1/audit` | Query audit log | Bearer/OIDC (admin) | Design only |

### 4.4 Prerequisites for implementation

| Prerequisite | Status | Blocker |
|-------------|--------|---------|
| Enterprise IdP approved | BLOCKED | B-03 (no security owner, no IdP) |
| Data policy approved | BLOCKED | B-04 (no data owner) |
| Deployment target approved | BLOCKED | B-02 (no remote/CI/branch protection) |
| Security owner assigned | BLOCKED | B-07 (no security/operations owner) |
| AI provider/data policy approved | BLOCKED | B-05 (no AI system owner) |

**Conclusion:** Production API contract is **design only**. Implementation requires Phase B decisions that are currently blocked. No source code changes made (plan constraint honored).

---

## 5. CI Workflow Status

The `.github/workflows/ci.yml` defines validate/security/workflow-lint/SBOM jobs. However:
- No Git remote CI has been executed (no remote configured at the time the workflow was created; remote now exists but CI has not run).
- No branch protection or required checks configured.
- Actions are tag-pinned, not SHA-pinned.

**Recommendation:** Run CI on the next push to origin/main to capture pass/fail evidence. This requires pushing the current commit (a268319) which is ahead of origin/main by 1 commit.

---

## 6. Phase C Summary

| Item | Status |
|------|--------|
| Phase A evidence gap list | Documented (§3) |
| Production API contract design | Documented (§4) — design only, no implementation |
| CI workflow status | Documented (§5) — not yet executed |
| Git remote | Already connected (origin exists) |

**Next:** Phase D — Flaps ERP S0 baseline reconciliation (execute per `1793033512795-flaps-erp-workbench-completion.md`).
