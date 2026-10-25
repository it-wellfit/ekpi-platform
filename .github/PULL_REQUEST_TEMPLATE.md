## Summary

<!-- Describe the problem and the user-visible or operational outcome. -->

## Traceability

- Issue:
- Spec / requirements:
- Contract IDs (if any):
- Risk tier (Low / Medium / High):
- Related ADRs / decisions:
- Traceability register row updated (yes / no, with reason):
- Phase gate affected (Phase A / Phase B / none):

## Verification

- [ ] `scripts/check` passed (documentation links, high-signal secret scan, format, Clippy, tests, self-tests, schema)
- [ ] `gate.ps1 -IncludeSmoke` passed, or not required with a reason
- [ ] Focused regression or integration tests added/updated
- [ ] Contract/schema checks updated where a public format changed
- [ ] CI `validate` matrix (Ubuntu and Windows) passed
- [ ] CI `security` job passed
- [ ] Review findings fixed; affected checks rerun
- [ ] Release notes updated, or this change is marked not release-facing
- [ ] Documentation, contracts and traceability updated in the same change

## Evidence

<!-- Commands, revision, environment, date and observed result. Do not paste secrets or real customer data. A skipped or blocked check must be recorded as such, never as PASS. -->

- Revision / date:
- Commands and results:
- Artifacts or screenshots:
- Backup/restore evidence (release or high-risk data change):

## Risk-specific evidence

<!-- For high-risk work, include applicable permission-denied, fail-closed, version, retry/idempotency, migration/rollback, tool-failure and security evidence. Mark not applicable with a reason. -->

## UI and accessibility (when applicable)

- [ ] Keyboard navigation and visible focus checked
- [ ] Semantic labels/status/error announcements checked
- [ ] Contrast and narrow-screen layout checked
- [ ] Existing behavior preserved or explicitly changed by the spec
- [ ] Automated route/render tests updated; manual checks recorded

## Data, compatibility, and recovery

- Data/schema or external contract impact:
- Migration / rollback / recovery:
- Release smoke checks:
- Known limitations:
- Security/privacy impact and owner review (if applicable):

## Phase gate claim (only if applicable)

<!-- Passing tests do not approve an MVP or production release. Link the gate row and evidence; if the gate is not approved, say so. -->

- Gate and row:
- Evidence:
- Status (Met / Partial / Not met / Blocked):
- Remaining blockers and owner:
