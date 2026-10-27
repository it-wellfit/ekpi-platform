# Local web-route contract

- **Status:** Active; local prototype
- **Owner:** Engineering
- **Last reviewed:** 2026-10-26
- **Contract ID:** `CTR-WEB-001`
- **Source:** `platform/src/governance_web.rs`
- **Verification:** `platform/tests/governance_web.rs`, `platform/tests/web_health.rs`

## Server boundary

- Binary: `governance_web` (auto-discovered from `platform/src/bin/governance_web.rs`).
- Bind address: `127.0.0.1:3000` (loopback only; not configurable in the current code).
- Ledger path: `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl`, or `EKP_GOVERNANCE_DATA` if set.
- Rendering: server-side Maud HTML, CSS only, zero JavaScript.
- Authentication: **local session only**. A single process-memory account is created at startup from `EKPI_LOCAL_AUTH_USER` and `EKPI_LOCAL_AUTH_PASSWORD`; `EKPI_LOCAL_AUTH_ROLE` selects `viewer`, `editor`, `approver` or `admin` (default `editor`). There is no default account, IdP, SSO, MFA, account recovery, persistent identity store or access review. Sessions expire after 3,600 seconds and are lost on process restart. Do not describe this as enterprise authentication and do not expose the server beyond the local machine.
- Passwords: PBKDF2-HMAC-SHA256 with a random 16-byte salt and 600,000 iterations. Session tokens are 32 random bytes from the OS CSPRNG. Failed logins are locked for 300 seconds after 5 failures. The password is never printed, logged or written to the ledger.
- Cookie: `ekpi_session`, `HttpOnly`, `SameSite=Strict`, `Path=/`, `Max-Age` bounded by the session expiry. `Secure` is added only when `EKPI_COOKIE_SECURE=1`; the current listener is plain HTTP on loopback, so a browser will not send a `Secure` cookie from `http://127.0.0.1` unless it is configured to trust that origin.
- Request middleware: session guard on every route except the public paths below; same-origin guard on non-GET/HEAD requests; per-request `x-request-id` validation/echo/generation; 15-second request timeout; at most 64 concurrent requests (excess requests receive `429` with `Retry-After`); one JSON access-log line per request on stdout (request id, method, path, status, duration; no actor, body data or cookie value).
- Form bodies are read with a 64 KiB limit; an oversized or malformed body returns `400` and appends no event.
- Security headers on every response: CSP, `X-Content-Type-Options`, `Referrer-Policy`, `X-Frame-Options`, `Cache-Control: no-store`, `Permissions-Policy`, `Cross-Origin-Opener-Policy` and `Cross-Origin-Resource-Policy`.
- Request body format: `application/x-www-form-urlencoded` for POST routes.

## Authorization matrix

Roles are ordered `viewer < editor < approver < admin`; a higher role satisfies a lower requirement. The role is attached to the session at login and is not re-read from the ledger or an external directory.

| Surface | Required role |
| --- | --- |
| `/healthz`, `/metrics`, `/login`, `/assets/*` | Public |
| All ledger-backed GET pages | `viewer` |
| `POST /projects`, `/evidence`, `/inspections`, `/workflows`, `/controls` | `editor` |
| `POST /evidence/{id}/review`, `/workflows/{id}/{version}/publish`, `/controls/{id}/status`, `/lifecycle` | `approver` |
| `POST /logout` | Any valid session |
| Unauthenticated data-page request | `401` with the login form; no ledger content |
| Authenticated but insufficient role | `403`; no event appended |

This matrix is an application-level local authorization check. It is not tenant isolation, delegation or an access-review control.

## Operational routes

| Method | Path | Purpose | Response | Test evidence |
| --- | --- | --- | --- | --- |
| GET | `/healthz` | Liveness only; does not claim readiness or dependency health | JSON: status, service, version, `scope: local-only`, `readiness: liveness-only` | `web_health.rs` |
| GET | `/metrics` | Prometheus text counters for requests, 4xx/5xx, in-flight, concurrency refusals and duration | `text/plain; version=0.0.4` | `web_health.rs` |
| GET | `/login` | Login form | HTML | `governance_web.rs` |
| POST | `/login` | `username`, `password`; max 64 KiB | `303` to `/` and `Set-Cookie: ekpi_session=…` on success; `401` and no cookie on failure | `governance_web.rs` |
| POST | `/logout` | None | `303` to `/login` and an expired `ekpi_session` cookie | `governance_web.rs` |

`/healthz`, `/metrics`, `/login` and the stylesheet are public. They are aggregate-only or authentication surfaces and must stay on the loopback listener. They do not provide a dashboard, alerting, trace export, SLO, error budget or capacity model.

## Read routes

All routes in this table require a `viewer` session and return `401` without one.

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
| GET | `/assets/governance.css` | Stylesheet (public) | `text/css; charset=utf-8` | Route test |

## Mutation routes

All mutation routes require a session with at least the role shown. `Editor` covers ordinary creation; `Approver` covers review, publication, quality-status and lifecycle decisions. The server derives the ledger actor from the validated session username and ignores any submitted `actor` or `reviewer` field. This is local process-memory identity, not an enterprise identity.

| Method | Path | Role | Form fields | Success behavior | Refusal behavior |
| --- | --- | --- | --- | --- | --- |
| POST | `/projects` | `editor` | `name`, `department`, `owner`, `risk_tier` | Appends `ProjectCreated`; redirects/re-renders | Validation error; no event |
| POST | `/evidence` | `editor` | `project_id`, `title`, `source_system`, `source_url`, `source_version`, `evidence_type`, `owner` | Appends `EvidenceLinked` | Validation error; no event |
| POST | `/evidence/{id}/review` | `approver` | `result`, `note` | Appends `EvidenceReviewed` | Validation error; no event |
| POST | `/inspections` | `editor` | `project_id`, `QC-01..QC-29` result/note/evidence fields; reviewer is taken from the session | Appends `QualityInspectionRecorded` | Missing/invalid 29-item data refused |
| POST | `/workflows` | `editor` | `base_id`, `title`, `department`, `owner`, `description`, `stages` | Appends `WorkflowSaved` | Validation error; no event |
| POST | `/workflows/{id}/{version}/publish` | `approver` | none; the session user is the publisher | Appends `WorkflowPublished`; version becomes immutable | Invalid/immutable transition refused |
| POST | `/controls` | `editor` | `project_id`, `kind`, `title`, `description`, `owner`, `due_date` | Appends `QualityCaseCreated` | Validation error; no event |
| POST | `/controls/{id}/status` | `approver` | `status`, `note` | Appends `QualityCaseUpdated` | Invalid transition refused; CAPA closure blocked until verification/effectiveness |
| POST | `/lifecycle` | `approver` | `project_id`, `gate_id`, `decision`, `reason`, `evidence_reference`, `risk_acceptor`, `remediation_due`, `non_waivable` | Appends `GateDecisionRecorded` | Non-waivable condition cannot be waived; missing reason/evidence refused |

The field list is the current HTML form contract. Server-side validation is authoritative; HTML attributes are convenience only. Exact error strings are not a stable contract.

## Event and response semantics

- A successful mutation appends exactly one ledger event and then responds with a redirect or a refreshed page.
- A rejected mutation must not append an event. Route tests assert this for foreign-origin requests and for missing/insufficient sessions.
- A mutating request with a foreign `Origin` returns `403` with a plain-text Chinese message and appends no event. When `Origin` is absent, the guard accepts only `Sec-Fetch-Site: same-origin` or `Sec-Fetch-Site: none`; other or missing fetch metadata is refused. This is CSRF hardening for a loopback tool, not a user identity check.
- Unauthenticated data-page requests return `401` and render the login form. Authenticated requests with an insufficient role return `403`. Neither response exposes ledger content.
- The login form accepts at most 64 KiB. Oversized or malformed login bodies return `400` without setting a cookie. Failed credentials return `401`; there is no response difference that confirms whether a username exists.
- Ledger read/validation errors render an error page; they do not expose secrets or stack traces.
- There is no JSON response mode for domain data, pagination, filtering API, ETag or versioned public API. `/healthz` is a liveness JSON document; `/metrics` is aggregate Prometheus text. Neither exposes ledger content or identity.
- The concurrency bound is a local overload guard, not an identity-aware rate limit.
- The access log and metrics are correlation/aggregate signals, not an audit of authenticated identity or an SLO.

## Not part of this contract

- `/health`, `/api/v1/cases/:id/decide` and `/api/v1/cases/:id/snapshot` in `platform/src/api.rs` are **not served** by this binary. The `decide` handler returns a fixed `comply` placeholder and must not be exposed.
- There is no versioned public API, OpenAPI document, tenant header or webhook.
- There is no enterprise authentication, SSO, MFA, account recovery, persistent user database, access review or tenant isolation. The local session is not evidence for any of those controls.
- The HTML may change visually without a contract break, but a route, method, field, role requirement or success/refusal semantic change requires a contract update and tests.

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

The route tests cover authenticated reads, unauthenticated `401`, viewer/editor/approver authorization, login/logout, oversized login bodies, mutation flows, audit chain, same-origin rejection and presentation constraints. The health tests cover liveness semantics, correlation-id validation, security headers and aggregate metrics. Manual browser checks are still required for keyboard, screen-reader, contrast and narrow-screen behavior; see [quality gates](../development/quality-gates.md).

## Update triggers

Update this document when a route, HTTP method, form field, role requirement, authentication environment variable, validation/refusal rule, security header, bind address or ledger path changes. Update `CTR-WEB-001` in the [traceability register](../traceability/register.md) in the same change.
