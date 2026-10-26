# Local web-route contract

- **Status:** Active; local prototype
- **Owner:** Engineering
- **Last reviewed:** 2026-10-26
- **Contract ID:** `CTR-WEB-001`
- **Source:** `platform/src/governance_web.rs`
- **Verification:** `platform/tests/governance_web.rs`

## Server boundary

- Binary: `governance_web` (auto-discovered from `platform/src/bin/governance_web.rs`).
- Bind address: `127.0.0.1:3000` (loopback only; not configurable in the current code).
- Ledger path: `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl`, or `EKP_GOVERNANCE_DATA` if set.
- Rendering: server-side Maud HTML, CSS only, zero JavaScript.
- Authentication: **none**. Authorization: **none**. Do not expose this server beyond the local machine.
- Request middleware: same-origin guard on non-GET/HEAD requests; per-request `x-request-id` validation/echo/generation; 15-second request timeout; at most 64 concurrent requests (excess requests receive `429` with `Retry-After`); one JSON access-log line per request on stdout (request id, method, path, status, duration; no actor or body data).
- Security headers on every response: CSP, `X-Content-Type-Options`, `Referrer-Policy`, `X-Frame-Options`, `Cache-Control: no-store`, `Permissions-Policy`, `Cross-Origin-Opener-Policy` and `Cross-Origin-Resource-Policy`.
- Request body format: `application/x-www-form-urlencoded` for POST routes.

## Operational routes

| Method | Path | Purpose | Response | Test evidence |
| --- | --- | --- | --- | --- |
| GET | `/healthz` | Liveness only; does not claim readiness or dependency health | JSON: status, service, version, `scope: local-only`, `readiness: liveness-only` | `web_health.rs` |
| GET | `/metrics` | Prometheus text counters for requests, 4xx/5xx, in-flight, concurrency refusals and duration | `text/plain; version=0.0.4` | `web_health.rs` |

These endpoints are unauthenticated and aggregate-only. They must stay on the loopback listener. They do not provide a dashboard, alerting, trace export, SLO, error budget or capacity model.

## Read routes

| Method | Path | Purpose | Response | Test evidence |
| --- | --- | --- | --- | --- |
| GET | `/` | Work overview/dashboard | HTML | Route test |
| GET | `/projects` | Project list and create form | HTML | Route test |
| GET | `/evidence` | Evidence references and review state | HTML | Route test |
| GET | `/inspections` | 29-item quality inspection form | HTML | Route test |
| GET | `/workflows` | Workflow drafts and publish actions | HTML | Route test |
| GET | `/controls` | Risk/change/defect/CAPA cases | HTML | Route test |
| GET | `/lifecycle` | Lifecycle gate decisions | HTML | Route test |
| GET | `/audit` | Ledger events and chain verification | HTML | Route test |
| GET | `/assets/governance.css` | Stylesheet | `text/css; charset=utf-8` | Route test |

## Mutation routes

| Method | Path | Form fields | Success behavior | Refusal behavior |
| --- | --- | --- | --- | --- |
| POST | `/projects` | `name`, `department`, `owner`, `risk_tier`, `actor` | Appends `ProjectCreated`; redirects/re-renders | Validation error; no event |
| POST | `/evidence` | `project_id`, `title`, `source_system`, `source_url`, `source_version`, `evidence_type`, `owner`, `actor` | Appends `EvidenceLinked` | Validation error; no event |
| POST | `/evidence/{id}/review` | `result`, `actor`, `note` | Appends `EvidenceReviewed` | Validation error; no event |
| POST | `/inspections` | `project_id`, `reviewer`, `QC-01..QC-29` result/note/evidence fields | Appends `QualityInspectionRecorded` | Missing/invalid 29-item data refused |
| POST | `/workflows` | `base_id`, `title`, `department`, `owner`, `description`, `stages`, `actor` | Appends `WorkflowSaved` | Validation error; no event |
| POST | `/workflows/{id}/{version}/publish` | `actor` | Appends `WorkflowPublished`; version becomes immutable | Invalid/immutable transition refused |
| POST | `/controls` | `project_id`, `kind`, `title`, `description`, `owner`, `due_date`, `actor` | Appends `QualityCaseCreated` | Validation error; no event |
| POST | `/controls/{id}/status` | `status`, `actor`, `note` | Appends `QualityCaseUpdated` | Invalid transition refused; CAPA closure blocked until verification/effectiveness |
| POST | `/lifecycle` | `project_id`, `gate_id`, `decision`, `actor`, `reason`, `evidence_reference`, `risk_acceptor`, `remediation_due`, `non_waivable` | Appends `GateDecisionRecorded` | Non-waivable condition cannot be waived; missing reason/evidence refused |

The field list is the current HTML form contract. Server-side validation is authoritative; HTML attributes are convenience only. Exact error strings are not a stable contract.

## Event and response semantics

- A successful mutation appends exactly one ledger event and then responds with a redirect or a refreshed page.
- A rejected mutation must not append an event. Route tests assert this for foreign-origin requests.
- A foreign `Origin` on a POST returns `403` with a plain-text Chinese message. A request without an `Origin` header is currently allowed by the guard.
- Ledger read/validation errors render an error page; they do not expose secrets or stack traces.
- There is no JSON response mode for domain data, pagination, filtering API, ETag or versioned public API. `/healthz` is a liveness JSON document; `/metrics` is aggregate Prometheus text. Neither exposes ledger content or identity.
- The concurrency bound is a local overload guard, not an identity-aware rate limit.

## Not part of this contract

- `/health`, `/api/v1/cases/:id/decide` and `/api/v1/cases/:id/snapshot` in `platform/src/api.rs` are **not served** by this binary. The `decide` handler returns a fixed `comply` placeholder and must not be exposed.
- There is no versioned public API, OpenAPI document, authentication handshake, tenant header or webhook.
- The HTML may change visually without a contract break, but a route, method, field or success/refusal semantic change requires a contract update and tests.

## Compatibility and change process

- Additive form fields are allowed if the server tolerates their absence or provides a default; document the change.
- Removing/renaming a route or field is breaking: update the form, handler, tests, docs and traceability row.
- Route changes must preserve the same-origin guard, CSP and zero-JS constraints unless an approved security decision changes them.
- Never add a route that exposes the placeholder API or writes a decision without the engine and evidence path.

## Verification

```powershell
cd platform
cargo test --locked --test governance_web --test web_health
```

The route tests cover the read pages, mutation flows, audit chain, foreign-origin rejection and presentation constraints. The health tests cover liveness semantics, correlation-id validation, security headers and aggregate metrics. Manual browser checks are still required for keyboard, screen-reader, contrast and narrow-screen behavior; see [quality gates](../development/quality-gates.md).

## Update triggers

Update this document when a route, HTTP method, form field, validation/refusal rule, security header, bind address or ledger path changes. Update `CTR-WEB-001` in the [traceability register](../traceability/register.md) in the same change.
