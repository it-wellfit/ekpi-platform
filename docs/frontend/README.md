# Frontend

- **Status:** Active; local prototype only
- **Owner:** Frontend/UX
- **Last reviewed:** 2026-10-26
- **Route contract:** [Web routes](../contracts/web-routes.md)
- **UX requirements:** [governance/05 functional spec](../governance/05-functional-spec-v1.md)
- **Accessibility baseline:** WCAG 2.2 AA target; `DEC-17` still needs local legal review and an approved browser/device matrix

## Information architecture

The governance workbench is a server-rendered, CSS-first application. It has no client-side router, JavaScript bundle or API fetch layer. Each page is a full HTML response; mutations are HTML form POSTs followed by a redirect or a re-render.

| Page | Route | Purpose | Mutations |
| --- | --- | --- | --- |
| Login | `GET /login` | Local username/password form | `POST /login` |
| Work overview | `GET /` | Dashboard counts and recent governance state | `POST /logout` |
| Projects | `GET /projects` | List and register projects | `POST /projects` |
| Evidence | `GET /evidence` | List source references and review state | `POST /evidence`; `POST /evidence/{id}/review` |
| Inspections | `GET /inspections` | Record the 29-item quality inspection | `POST /inspections` |
| Workflows | `GET /workflows` | Draft and publish workflow versions | `POST /workflows`; `POST /workflows/{id}/{version}/publish` |
| Controls | `GET /controls` | Risk, change, defect and CAPA cases | `POST /controls`; `POST /controls/{id}/status` |
| Lifecycle | `GET /lifecycle` | Gate decisions across the 12-stage lifecycle | `POST /lifecycle` |
| Audit | `GET /audit` | Verify and inspect the local hash-chained ledger | None |
| Stylesheet | `GET /assets/governance.css` | CSS-only presentation (public) | None |

Every data page requires a local `viewer` session; ordinary creation requires `editor` and review/publication/quality-status/lifecycle actions require `approver`. The UI does not hide actions the current session cannot perform: the server rejects them with `403`. There is no tenant selector, user administration page, password recovery, settings page or API documentation page. Those remain Phase B surfaces and are not implemented.

## Page composition

- A shared page shell provides a skip link, navigation, page heading, status/error region and footer.
- The dashboard summarizes projects, evidence, workflows, quality cases, inspections and gate decisions.
- Forms use native HTML controls (`form`, `input`, `select`, `textarea`, `button`) with visible labels.
- Long lists render as semantic tables with row-level actions.
- Empty states distinguish “no records yet” from an error; they do not fabricate sample records.
- The audit page shows sequence, actor, event type and chain-verification status.

## Interaction states

| State | Behavior | Evidence |
| --- | --- | --- |
| Unauthenticated data page | `401` and the login form; no ledger content | Route test |
| Insufficient role | `403`; no event appended | Route test |
| Invalid credentials | `401`, no session cookie; user-existence is not confirmed | Route test |
| Initial/empty | Renders an empty-state message and the available create action | Route tests; manual review |
| Success | A mutation appends an event, then redirects or re-renders with the new record | `governance_web` tests |
| Validation error | The request is rejected and an error page/message is rendered; no event is appended | Handler tests and manual checks |
| Oversized form body | `400` before parsing; no event appended | Route test |
| Ledger/read error | An error page is rendered without exposing sensitive content | `error_response` path |
| Foreign-origin mutation | `403` response; no ledger event appended | `governance_web` integration test |
| Publish conflict | Published versions are immutable; the handler refuses an invalid publish | Workflow tests |
| CAPA closure refusal | Closure before verification/effectiveness is refused | Quality-case tests |
| Non-waivable gate block | A blocking gate decision cannot be bypassed by a waiver field | Gate-decision tests |

There is no optimistic UI, client-side cache, toast system or offline mode. The browser's native form behavior is the interaction model.

## Forms and validation

| Form | Fields | Server-side rules |
| --- | --- | --- |
| Login | `username`, `password` | Matches the single startup account; lockout after 5 failures for 300 s; 64 KiB body limit; no session cookie on failure |
| Project | `name`, `department`, `owner`, `risk_tier`, `actor` | Required fields; text length/cleanup; actor required |
| Evidence link | `project_id`, `title`, `source_system`, `source_url`, `source_version`, `evidence_type`, `owner`, `actor` | Required fields; source URL/version recorded as a reference, not fetched |
| Evidence review | `result`, `actor`, `note` | Allowed review result; reviewer/actor required; note retained |
| Workflow draft | `base_id`, `title`, `department`, `owner`, `description`, `stages`, `actor` | Stages parsed into an ordered list; version derived from base |
| Workflow publish | `actor` | Published version becomes immutable; publisher recorded |
| Quality case | `project_id`, `kind`, `title`, `description`, `owner`, `due_date`, `actor` | Type and initial status validated |
| Quality case status | `status`, `actor`, `note` | Allowed transition; CAPA closure requires verification/effectiveness |
| Gate decision | `project_id`, `gate_id`, `decision`, `actor`, `reason`, `evidence_reference`, `risk_acceptor`, `remediation_due`, `non_waivable` | Non-waivable conditions cannot be waived; reason/evidence required for exceptions |
| Inspection | `project_id`, `reviewer`, plus 29 `QC-01`–`QC-29` result/note/evidence fields | All 29 decisions required; missing evidence rejected |

Field-level HTML constraints are a convenience only; the server-side rules are authoritative. The exact handler behavior is in `platform/src/governance_web.rs` and is covered by route tests.

## API integration

There is no frontend-to-JSON-API integration. The app does not use `fetch`, `XMLHttpRequest`, WebSockets or a service worker. If a JSON API is introduced later, it needs its own contract, authentication, error model and contract tests; the current `platform/src/api.rs` is a placeholder and is not served.

## Accessibility

Target: WCAG 2.2 AA, with local legal requirements still to be confirmed (`DEC-17`).

Current implementation evidence:

- Semantic landmarks, headings, labels and tables.
- A skip link and visible `:focus-visible` styling.
- Status and error messages rendered in text, not color alone.
- A narrow-screen media rule and CSS-only layout.
- `report.html` is checked for zero `<script>` elements; the governance app is also zero-JS.

Not yet evidenced:

- Manual keyboard-only walkthrough on every route.
- Screen-reader testing with named assistive technologies.
- Contrast measurement against the approved palette.
- Browser/device matrix testing.
- Automated accessibility scanning or browser E2E tests.

The route tests check semantics, CSP, skip link, focus style, CSS content type and a narrow-screen rule. They are not a substitute for the manual checks required by [quality gates](../development/quality-gates.md) for UI changes.

## Error, loading and empty states

- **Loading:** not applicable to the server-rendered app; the browser waits for the response.
- **Error:** rendered as an error page with a safe message; no stack trace or secret values.
- **Empty:** explicit empty-state copy; no fabricated records.
- **Partial data:** the page renders the records it has and surfaces the ledger/read error separately.
- **Long-running action:** no progress indicator; the request completes or fails synchronously. A future asynchronous workflow needs a job/status contract.

## Known frontend gaps

| ID | Gap | Impact | Tracking |
| --- | --- | --- | --- |
| FE-GAP-01 | Login exists, but the UI is not role-aware and has no account administration | Every session sees every action; the server rejects insufficient roles; no user list, recovery, MFA or access review | Phase B, `DEC-12` |
| FE-GAP-02 | No tenant selector or tenant-scoped views | Cannot serve multiple organizations | Phase B, `DEC-12` |
| FE-GAP-03 | No browser E2E or automated accessibility suite | Manual checks can be skipped | Phase B quality work |
| FE-GAP-04 | No localization framework | UI copy is Chinese; docs are English | Decide before broader pilot |
| FE-GAP-05 | No loading/progress UI for long actions | User cannot tell whether a slow request is alive | Phase B if async work is added |
| FE-GAP-06 | No CSRF token; same-origin guard only | Missing-Origin requests are not rejected | Security hardening |

## Update triggers

Update this document when a route, form field, validation rule, page state, rendering constraint or accessibility requirement changes. Update the [web-route contract](../contracts/web-routes.md) and the traceability row in the same change.
