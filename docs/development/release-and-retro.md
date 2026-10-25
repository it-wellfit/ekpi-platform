# Release, smoke checks, backup, and retro

- **Status:** Active
- **Owner:** Engineering + Operations
- **Last reviewed:** 2026-10-26
- **Phase gates:** [Phase A/B](../quality/phase-gates.md)
- **Template:** [Release notes and operations handoff](release-template.md)

Use the release template for a release or pilot handoff. Record observed output and link evidence; use `Pending` with an owner for unresolved deployment, support, training, or policy decisions.

## Release readiness

Before a release or tag:

1. Confirm the accepted issue/spec, the affected phase gate, and all required checks have traceable passing evidence at the release revision.
2. Update the [traceability register](../traceability/register.md) with the release revision, test evidence, contracts and any `Validated` session.
3. Review version notes, known limitations, security/dependency findings, and data/compatibility impact.
4. Run the local gate for the revision:

   ```powershell
   powershell -ExecutionPolicy Bypass -File gate.ps1 -IncludeSmoke
   ```

5. On Windows, stop any running executable that would lock release outputs; build the locked binaries separately in each crate:

   ```powershell
   Set-Location eval
   cargo build --release --locked --bins
   cargo run --release --locked -- --selftest
   Set-Location ..\platform
   cargo build --release --locked --bins
   cargo run --release --locked --bin ekp_platform -- --selftest
   cargo run --release --locked --bin ekp_platform -- --declared-rules-schema
   ```

6. Create a backup and execute the restore drill, then record the bundle path and result in the [backup log](../operations/backup-log.md):

   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts/backup.ps1
   ```

   A release without restore evidence is **not verified**; record it as blocked or pending, never as passed.
7. Record the release version, source revision, toolchain, validation output, operator, and rollback/recovery steps.
8. Commit and tag only after the evidence exists. Commit the evidence itself (traceability, backup log, review record).

There is no approved deployment target in this repo. Do not deploy `governance_web` to shared or public networks: the prototype has no enterprise authentication and is local-only. Until a deployment target, identity model, data policy, and operator are approved, delivery ends at verified artifacts and documented local operation.

## Post-release smoke and monitoring

### Governance web prototype

Use a unique, disposable ledger path so the smoke test cannot read or alter an operator's existing ledger. From PowerShell:

```powershell
Set-Location platform
$env:EKP_GOVERNANCE_DATA = Join-Path $env:TEMP ("ekpi-governance-smoke-" + [guid]::NewGuid().ToString("N") + ".jsonl")
cargo run --locked --bin governance_web
```

Open `http://127.0.0.1:3000`; verify the dashboard, project, evidence, workflow, and audit views; create only synthetic, non-sensitive test records; verify the audit page; then stop the server with Ctrl+C. Record the disposable ledger path and smoke evidence. Do not place the ledger under a shared/public service or use production data.

### Engine and reports

Run the eval self-test, platform engine self-test, and declared-rules schema check from the crate directories using [the baseline commands](quality-gates.md). If a report, export, case-loading, or parameter path changed, also run the focused CLI scenario in [the regression checklist](regression-checklist.md) and inspect every expected artifact.

Monitoring/alerts and health checks do not exist for the prototype. Do not report post-release monitoring as present; Phase B requires an approved observability design (`DEC-16`). See [production readiness](../operations/production-readiness.md).

## /retro

Run /retro after each merged work unit, release, pilot session, material CI failure, or incident. Use the [retro template](../reviews/retro-template.md) and update the [improvement register](../reviews/README.md#improvement-register).

- Prefer a test or deterministic CI check for mechanical failures; keep AGENTS.md short and move conditional detail to the referenced document.
- Create a linked GitHub issue for accepted improvements and assign an owner and validation condition. If the remote is unavailable, record the action as `Blocked` with the reason; do not pretend an issue exists.
- Do not use a retro to lower a legal, security, data-protection or phase gate. A gate change requires an approved decision with a risk acceptor.
- A retro action is `Done` only with linked evidence.

## Update triggers

Update this procedure when the gate, release template, backup/restore method, smoke path, phase gate or improvement process changes. Keep the release record and traceability register aligned.
