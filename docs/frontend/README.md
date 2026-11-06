# Frontend

- **Status:** Active; local prototype only
- **Owner:** Frontend/UX
- **Last reviewed:** 2026-11-06
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
| Project workspace | `GET /projects/{id}` | Aggregate one project's summary, G1–G5 status, evidence, cases, inspections and recent activity | None |
| Evidence | `GET /evidence` | List source references and review state | `POST /evidence`; `POST /evidence/{id}/review` |
| Inspections | `GET /inspections` | Record the 29-item quality inspection | `POST /inspections` |
| Workflows | `GET /workflows` | Draft and publish workflow versions | `POST /workflows`; `POST /workflows/{id}/{version}/publish` |
| Controls | `GET /controls` | Risk, change, defect and CAPA cases | `POST /controls`; `POST /controls/{id}/status` |
| Lifecycle | `GET /lifecycle` | Gate decisions across the 12-stage lifecycle | `POST /lifecycle` |
| Audit | `GET /audit` | Verify and inspect the local hash-chained ledger | None |
| Stylesheet | `GET /assets/governance.css` | CSS-only presentation (public) | None |

Every data page requires a local `viewer` session; ordinary creation requires `editor` and review/publication/quality-status/lifecycle actions require `approver`. The UI is role-aware: read-only roles get a permission note instead of forms, editors keep creation but lose approval/publication/status/gate controls, and approvers/admins see the full action set. This is a usability layer only; the server still enforces the role and returns `403` for a direct request. The audit page accepts read-only filters (`project`, `actor`, `event`, `q`, `from`, `to`) and shows at most the newest 200 matches. There is no tenant selector, user administration page, password recovery, settings page or API documentation page. Those remain Phase B surfaces and are not implemented.

## Page composition

- A shared page shell provides a skip link, navigation, page heading, status/error region and footer.
- The dashboard summarizes projects, evidence, workflows, quality cases, inspections and gate decisions; count cards link to the corresponding page, and recent gate decisions are shown separately.
- The project workspace aggregates one project's summary, G1–G5 recorded decisions, evidence, quality cases, inspections and newest project events. Missing gate decisions render `尚未記錄`, never an inferred pass.
- The lifecycle page shows a per-project G1–G5 matrix containing only recorded decisions.
- Forms use native HTML controls (`form`, `input`, `select`, `textarea`, `button`) with visible labels.
- Long lists render as semantic tables with row-level actions; each scrollable table region has an accessible name and keyboard focus.
- Empty states distinguish “no records yet” from an error; they do not fabricate sample records.
- The audit page shows sequence, actor, event type and chain-verification status, plus a GET filter form.

## Interaction states

| State | Behavior | Evidence |
| --- | --- | --- |
| Unauthenticated data page | `401` and the login form; no ledger content | Route test |
| Insufficient role | The UI hides the control; a direct request still returns `403` with a focused error summary; no event appended | Route tests |
| Invalid credentials | `401`, no session cookie; user-existence is not confirmed; the error summary receives focus | Route test |
| Initial/empty | Renders an empty-state message and the available create action | Route tests; manual review |
| Success | A mutation appends an event, then redirects or re-renders with the new record | `governance_web` tests |
| Validation error | HTTP `200` re-render with preserved values and a focused, assertive error summary; no event is appended | Route tests |
| Oversized or malformed body | `400` before parsing; no event appended | Route test |
| Ledger/read or chain-verification error | HTTP `500` error page with a focused summary; no sensitive content and no stale `200` | Route test |
| Unknown project workspace | HTTP `404` with a safe explanation and a link back to the project list | Route test |
| Invalid audit date | The invalid date is not applied; a visible warning explains which filter was skipped | Route test |
| Foreign-origin mutation | `403` response; no ledger event appended | `governance_web` integration test |
| Publish conflict | Published versions are immutable; the handler refuses an invalid publish | Workflow tests |
| CAPA closure refusal | Closure before verification/effectiveness is refused | Quality-case tests |
| Non-waivable gate block | A blocking gate decision cannot be bypassed by a waiver field | Gate-decision tests |

There is no optimistic UI, client-side cache, toast system or offline mode. The browser's native form behavior is the interaction model.

## Forms and validation

| Form | Fields | Server-side rules |
| --- | --- | --- |
| Login | `username`, `password` | Matches the single startup account; lockout after 5 failures for 300 s; 64 KiB body limit; no session cookie on failure |
| Project | `name`, `department`, `owner`, `risk_tier` | Required fields; text length/cleanup; actor from the session |
| Evidence link | `project_id`, `title`, `source_system`, `source_url`, `source_version`, `evidence_type`, `owner` | Required fields; source URL/version recorded as a reference, not fetched; actor from the session |
| Evidence review | `result`, `note` | Allowed review result; reviewer from the session; note retained |
| Workflow draft | `base_id`, `title`, `department`, `owner`, `description`, `stages` | Stages parsed into an ordered list; version derived from base; actor from the session |
| Workflow publish | No form fields | Published version becomes immutable; publisher from the session |
| Quality case | `project_id`, `kind`, `title`, `description`, `owner`, `due_date` | Type and initial status validated; actor from the session |
| Quality case status | `status`, `note` | Allowed transition; CAPA closure requires verification/effectiveness; actor from the session |
| Gate decision | `project_id`, `gate_id`, `decision`, `reason`, `evidence_reference`, `risk_acceptor`, `remediation_due`, `non_waivable` | Non-waivable conditions cannot be waived; reason/evidence required for exceptions; actor from the session |
| Inspection | `project_id`, plus 29 `QC-01`–`QC-29` result/note/evidence fields | All 29 decisions required; missing evidence rejected; reviewer from the session |

Field-level HTML constraints are a convenience only; the server-side rules are authoritative. No form submits an `actor` or `reviewer` value; the server derives both from the validated session. The exact handler behavior is in `platform/src/governance_web.rs` and is covered by route tests.

## API integration

There is no frontend-to-JSON-API integration. The app does not use `fetch`, `XMLHttpRequest`, WebSockets or a service worker. If a JSON API is introduced later, it needs its own contract, authentication, error model and contract tests; the current `platform/src/api.rs` is a placeholder and is not served.

## Accessibility

Target: WCAG 2.2 AA, with local legal requirements still to be confirmed (`DEC-17`).

Current implementation evidence:

- Semantic landmarks, headings, labels and tables.
- A skip link and visible `:focus-visible` styling.
- Status and error messages rendered in text, not color alone.
- A focused error summary (`role="alert"`, `aria-live="assertive"`, `tabindex="-1"`, one `autofocus`) for validation, authorization and server failures.
- Scrollable table regions with `role="region"`, an `aria-label` and `tabindex="0"`.
- A narrow-screen media rule and CSS-only layout; wide form/list pages split above 1080px and stack at or below 1080px.
- `report.html` is checked for zero `<script>` elements; the governance app is also zero-JS.

Local visual audit evidence (2026-11-06): 52 screenshots for `admin`, `viewer` and `editor` at 1440×1100 and 390×844 in `%TEMP%\opencode\ekpi-ui-shots-v2`. The scripted browser checks confirmed zero `<script>` elements, the expected form/read-only counts per role, project-workspace aggregation, audit filtering, the `404` page, the `403` focus target, validation focus, and zero page-level horizontal overflow at 390px on the dashboard, project workspace and audit page. These are local observations, not an automated accessibility suite.

Not yet evidenced:

- A full manual keyboard-only walkthrough recorded against every route.
- Screen-reader testing with named assistive technologies.
- Contrast measurement against the approved palette.
- Browser/device matrix testing.
- Automated accessibility scanning or browser E2E tests.
- Representative user validation of role labels, copy and workflow fit.

The route tests check semantics, CSP, skip link, focus style, CSS content type, a narrow-screen rule, role-aware rendering, the error-summary focus target and the `500` fail-closed path. They are not a substitute for the manual checks required by [quality gates](../development/quality-gates.md) for UI changes.

## Error, loading and empty states

- **Loading:** not applicable to the server-rendered app; the browser waits for the response.
- **Error:** rendered as an error page with a safe message; no stack trace or secret values.
- **Empty:** explicit empty-state copy; no fabricated records.
- **Partial data:** the page renders the records it has and surfaces the ledger/read error separately.
- **Long-running action:** no progress indicator; the request completes or fails synchronously. A future asynchronous workflow needs a job/status contract.

## Known frontend gaps

| ID | Gap | Impact | Tracking |
| --- | --- | --- | --- |
| FE-GAP-01 | Login exists and the UI is role-aware, but there is no account administration | No user list, recovery, MFA, role change or access review; role changes require a process restart and environment variables | Phase B, `DEC-12` |
| FE-GAP-02 | No tenant selector or tenant-scoped views | Cannot serve multiple organizations | Phase B, `DEC-12` |
| FE-GAP-03 | No browser E2E or automated accessibility suite | Manual checks can be skipped | Phase B quality work |
| FE-GAP-04 | No localization framework | UI copy is Chinese; docs are English | Decide before broader pilot |
| FE-GAP-05 | No loading/progress UI for long actions | User cannot tell whether a slow request is alive | Phase B if async work is added |
| FE-GAP-06 | No CSRF token; same-origin guard only | Missing-Origin requests are not rejected | Security hardening |
| FE-GAP-07 | Audit filtering is read-only, in-memory and capped at 200 displayed events | A large ledger cannot be paged or exported from the UI | Phase B; add a bounded export/pagination contract |

## Update triggers

Update this document when a route, form field, validation rule, page state, rendering constraint or accessibility requirement changes. Update the [web-route contract](../contracts/web-routes.md) and the traceability row in the same change.
