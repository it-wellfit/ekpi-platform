# Release, smoke checks, and retro

Use [the release-notes and operations-handoff template](release-template.md) for a release or pilot handoff. Record observed output and link evidence; use `Pending` with an owner for unresolved deployment, support, training, or policy decisions.

## Release readiness

Before a release:

- Confirm the accepted issue/spec and all required checks have traceable passing evidence.
- Review version notes, known limitations, security/dependency findings, and data/compatibility impact.
- On Windows, stop any running executable that would lock release outputs; build the locked binaries separately in each crate:

      Set-Location eval
      cargo build --release --locked --bins
      cargo run --release --locked -- --selftest
      Set-Location ..\platform
      cargo build --release --locked --bins
      cargo run --release --locked --bin ekp_platform -- --selftest
      cargo run --release --locked --bin ekp_platform -- --declared-rules-schema

- Record the release version, source revision, toolchain, validation output, operator, and rollback/recovery steps.

There is no approved deployment target in this repo. Do not deploy governance_web to shared or public networks: the prototype has no enterprise authentication and is local-only. Until a deployment target, identity model, data policy, and operator are approved, delivery ends at verified artifacts and documented local operation.

## Post-release smoke and monitoring

### Governance web prototype

Use a unique, disposable ledger path so the smoke test cannot read or alter an operator's existing ledger. From PowerShell:

    Set-Location platform
    $env:EKP_GOVERNANCE_DATA = Join-Path $env:TEMP ("ekpi-governance-smoke-" + [guid]::NewGuid().ToString("N") + ".jsonl")
    cargo run --locked --bin governance_web

Open `http://127.0.0.1:3000`; verify the dashboard, project, evidence, workflow, and audit views; create only synthetic, non-sensitive test records; verify the audit page; then stop the server with Ctrl+C. Record the disposable ledger path and smoke evidence. Do not place the ledger under a shared/public service or use production data.

### Engine and reports

Run the eval self-test, platform engine self-test, and declared-rules schema check from the crate directories using [the baseline commands](quality-gates.md). If a report, export, case-loading, or parameter path changed, also run the focused CLI scenario in [the regression checklist](regression-checklist.md) and inspect every expected artifact.

When an approved deployment target exists, add its owner-approved health endpoint, rollback mechanism, alert thresholds, log retention, and smoke cases before automating deployment. Record observed failures and recovery; never report a check as passed when it was not run.

## /retro

Run /retro after each merged work unit or release, and after a material CI or production incident. Review navigation, automated guardrails, agent instructions, tool cost, and information access. Prefer a test or deterministic CI check for mechanical failures; keep AGENTS.md short and move conditional detail to the referenced document. Create a linked GitHub issue for accepted improvements and assign an owner and validation condition.
