# ADR-0005: Split the governance web workbench into bounded modules

- **Status:** Accepted
- **Date:** 2026-11-13
- **Owner:** Engineering
- **Deciders:** Engineering (refactor-criteria C3 slice; no business decision)
- **Related:** [Refactor criteria](../development/refactor-criteria.md) C3; traceability register `R31`; `platform/src/governance_web.rs`; `platform/tests/governance_web_route_inventory.rs`; [Web-route contract](../contracts/web-routes.md) CTR-WEB-001

## Context

`platform/src/governance_web.rs` is the served HTTP surface of the local
workbench. It has accumulated routing, middleware, authentication, page
rendering, approval views and the API v1 boundary in one file. Measured on
2026-11-13 at `0706145`:

| Measure | Value | Method |
| --- | ---: | --- |
| Physical lines | 4,816 | `[System.IO.File]::ReadAllLines` |
| Non-blank lines | 4,545 | non-blank count over the same file |
| Top-level items | 208 | `fn`/`struct`/`enum`/`impl`/`const`/`static` at column 0 |
| Top-level functions | 159 | same scan, functions only |
| Production route declarations | 47 | `.route(` at the router indentation; 49 including two test-only routers |
| In-file unit tests | 12 | `#[test]` and `#[tokio::test]` |
| Referencing files | 7 | `platform/src`, `platform/src/bin`, `platform/tests` |
| Commits touching the file | 28 | `git log --oneline -- platform/src/governance_web.rs` |

The refactor criteria threshold was `B > 10 || H >= 10` → `DEFER` until a
module design, a complete characterization suite and a single-writer window
exist. The collision surface has grown at every measurement (2026-11-09:
3,246 lines / 19 references / 21 commits; 2026-11-10: 4,025 lines / 24
commits; 2026-11-13: 4,816 lines / 7 direct references / 28 commits). Waiting
longer increases both the conflict risk and the cost of proving equivalence.

The behavior that must not change is broad: 21 registry pages, three detail
pages, 47 production route declarations, the public stylesheet, login/logout,
session-role checks, CSRF, same-origin/host guards, body limits, rate limiting,
request IDs, timeout/concurrency bounds, health/readiness/metrics, the approval
sub-workbench, and the honest 501 API v1 boundary.

## Decision

We will mechanically split `platform/src/governance_web.rs` into a directory
module with the following ownership boundaries:

| Module | Owns |
| --- | --- |
| `governance_web/mod.rs` | Module declarations, shared type aliases and constants, `router`, `router_with_approvals`, and the public re-exports that preserve the existing `ekp_platform::governance_web::*` API |
| `governance_web/observability.rs` | Request IDs, request counters, timeout/concurrency bound, rate-limiter state, `/healthz`, `/ready`, `/metrics`, response security headers |
| `governance_web/security.rs` | Body-size limits, session-token parsing, session/role checks, CSRF validation, same-origin/host guards, request admission |
| `governance_web/auth.rs` | `/login` GET/POST, `/logout` POST, login form, secure-cookie suffix |
| `governance_web/views.rs` | Read-only approval projections shared by pages and API (`approvals_view`, `approvals_view_counts`) |
| `governance_web/shell.rs` | `PageSpec`/`PAGE_REGISTRY`, layout/navigation, page responses, styled 404/fallback, shared markup helpers |
| `governance_web/api.rs` | API v1 handlers and the honest 501 boundary |
| `governance_web/pages/mod.rs` | Page module declarations |
| `governance_web/pages/governance.rs` | Dashboard, projects/evidence/workflows/controls/inspections/lifecycle/audit pages and their mutations |
| `governance_web/pages/approvals.rs` | Approval task pages, approval mutations, SLA calendar and civil-date helpers |
| `governance_web/pages/insights.rs` | Events browser, chain summary, search and data-map pages |
| `governance_web/pages/info.rs` | Settings, API docs, role matrix, glossary, help and about pages |

The public API is a hard constraint:

`router`, `router_with_approvals`, `SharedApprovals`, `PageSpec`,
`PAGE_REGISTRY`, `civil_from_unix`, `days_from_civil`, `days_in_month` and
`event_label` keep their names, signatures and module path.

Non-goals: adding or removing routes, changing status codes, headers, HTML
structure, ledger/approval data formats, authentication semantics, or the
honest 501 API boundary. This is a code-movement slice, not a feature slice.

### Dependency direction

```
mod (shared types/constants, router, public re-exports)
 ├── observability   (no sibling dependency)
 ├── security        (shared constants only)
 ├── auth            → security (cookie parsing)
 ├── views           (shared approval store type only)
 ├── shell           (domain snapshot types, registry)
 ├── api             → views, security
 └── pages/*         → shell, security, views, auth helpers
```

`mod.rs` may reference all modules; sibling edges are one-way and acyclic.
`pages/*` do not depend on `api`; `api` does not depend on a concrete page.
`views` is the only read model shared by both.

### Staged migration

Each stage is a separate commit and must pass the canonical gate before the
next stage starts.

1. **Stage 0 — design and characterization.** Add this ADR and
   `platform/tests/governance_web_route_inventory.rs`; update the refactor
   criteria and register. No production code moves. The new suite must pass on
   the unsplit file.
2. **Stage 1 — observability, security, auth.** Convert the file to
   `governance_web/mod.rs`; extract the three middleware/identity modules and
   their unit tests. Router and handlers stay in `mod.rs`.
3. **Stage 2 — shell, views, api.** Extract the page shell/registry, the shared
   approval view and the API v1 handlers.
4. **Stage 3 — pages.** Extract the four page modules; reduce `mod.rs` to module
   declarations, shared items, router composition and public re-exports.

A stage is complete only when the characterization suite and all 575 existing
workspace tests pass, `cargo fmt --check` and
`cargo clippy --workspace --all-targets -- -D warnings` are clean, and
`scripts/check.ps1` exits 0. `PASS` is recorded only for the exact revision.

### Rollback

Every stage is one commit with no data migration. `git revert <stage-commit>`
restores the previous single-file or partially split state; the ledger and
approval JSONL files are untouched. If a stage cannot be made green without a
behavior change, the stage is abandoned (`git reset --hard <last-green>`) and
the behavior change is proposed separately with a red test and a spec update.

### Single-writer protocol

- The governance web module has one writer for the duration of Stages 1–3.
- Each stage starts only when `git status --porcelain` is empty.
- No other change may touch `platform/src/governance_web*` or its tests while a
  stage is open; a parallel change must wait or be rebased after the stage.
- The writer records the measured inventory and gate result in the same commit
  as the stage (or in the immediately following evidence commit).
- No route, handler or data-format edit rides along with a movement stage.

### Acceptance criteria

1. Public API unchanged; all seven referencing files compile without edits.
2. 47 production route declarations and 21 `PAGE_REGISTRY` entries unchanged;
   route paths, methods, auth gates and response semantics identical.
3. The route-inventory characterization suite passes before Stage 1 and after
   every stage.
4. All pre-existing workspace tests pass; no test is deleted, ignored or
   weakened.
5. `scripts/check.ps1` exits 0 at every stage commit; docs and secret counts are
   reported exactly as observed.
6. `mod.rs` is reduced to routing/state/re-exports; no extracted module exceeds
   1,200 physical lines; no new `unsafe`, `allow`, TODO or FIXME is introduced.
7. Alignment with [refactor-criteria](../development/refactor-criteria.md) §1:
   behavior is preserved, each change is revertible, and equivalence evidence is
   committed with the change.

## Consequences

### Positive

- Middleware, identity, shell, API and page changes get separate review and
  test surfaces; a page change no longer recompiles/relinks one 4,800-line
  translation unit by convention alone.
- The route inventory and registry become explicit contracts rather than
  incidental ordering in one file.
- Future C5-style deduplication can be evaluated per module instead of across a
  monolith.

### Negative / accepted cost

- A mechanical move touches every handler and temporarily increases the diff
  size; review must check movement, not behavior.
- Module-private visibility has to be widened to `pub(super)`/`pub(crate)` for
  cross-module helpers; this is internal only and not a public API change.
- The split does not add enterprise identity, tenancy, observability export or
  compliance evidence. Those remain separate, blocked items.

### Follow-up

- Run the canonical gate on each stage commit and record it in register `R31`.
- After Stage 3, re-measure the file/module sizes and update
  `refactor-criteria.md` §4 with the observed result.
- Keep the route-inventory suite in the canonical gate as the anti-regression
  lock for future route work.

## Alternatives considered

| Alternative | Why rejected |
| --- | --- |
| Do nothing | The file keeps growing (3,246 → 4,816 lines) and every change collides with unrelated work; the deferral conditions are now satisfiable. |
| One-shot rewrite into a new crate | A new crate changes the public path and build graph, needs a new dependency boundary and cannot be reverted as cheaply. |
| Split only pages, keep middleware in one file | Leaves the largest cross-cutting collision surface (auth/CSRF/limits/observability) untouched and still couples every handler to it. |
| Split by line count without a characterization suite | A green compile does not prove route/auth/header equivalence; the inventory suite is the equivalence evidence. |

## Evidence

- 2026-11-13 inventory at `0706145`: 4,816 physical / 4,545 non-blank lines,
  208 top-level items, 159 top-level functions, 47 production route
  declarations, 12 in-file tests, 7 referencing files, 28 commits.
- Existing baseline before this ADR: canonical `scripts/check.ps1` exit 0,
  workspace 575 passed / 0 failed / 0 ignored (register `R30`).
- Stage 0 characterization: `platform/tests/governance_web_route_inventory.rs`
  is added and passes on the unsplit file: **7 passed / 0 failed** on
  2026-11-13. Its exact canonical gate result is recorded in register `R31`.
- No CI run, κ measurement, Phase A approval, Phase B readiness or independent
  review is claimed by this ADR.

## Compliance and security impact

- No trust-boundary, identity, access-control, retention or data-classification
  change is intended. Authentication, CSRF, same-origin/host checks and body
  limits must remain byte-for-byte equivalent in behavior and are locked by the
  characterization suite.
- Any accidental change to those controls is a security regression and blocks
  the stage. A future change to the controls needs its own security review and
  contract update; this ADR does not authorize one.
- No public contract changes: CTR-WEB-001 remains the read-route contract and
  the API v1 501 boundary remains honest and unwired.

## Supersession

- Supersedes: none.
- Superseded by: none.
