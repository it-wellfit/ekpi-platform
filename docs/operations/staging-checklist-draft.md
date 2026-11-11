# Staging checklist for the internal single-host direction (draft)

- **Status:** Draft proposal; **not approved**, **not implemented**
- **Owner:** Operations + Security (proposed; acceptance required)
- **Last reviewed:** 2026-11-10
- **Phase gate:** [Phase B](../quality/phase-gates.md#phase-b-mvp--production-grade-gate) (B1/B4/B7)
- **Readiness row:** [PR-02](production-readiness.md) — deployment target and environment
- **External blocker:** [B-10](../assurance/external-blockers.md) — production deployment target and change approval
- **Direction of record:** 2026-11-10 owner decision — single internal Windows host, native binary; recorded in [PR-02](production-readiness.md) and [B-10](../assurance/external-blockers.md) (session `20261110-dev-blockers`)
- **Related:** [install](../deployment/install.md), [security](../deployment/security.md), [monitoring](../deployment/monitoring.md), [local backup](local-backup.md), [release evidence bundle](release-evidence-bundle.md)

## Purpose and limit

This checklist operationalises the first step of the chosen direction: a **staging** deployment on a single internal Windows host, before any production or LAN-wide exposure. It is not an approval, not a deployment record and not evidence that staging exists.

Until the PR-02 fields and the Stage 2 gate below are approved and implemented, the existing operating boundary stands: no shared-host, LAN, container or public exposure; synthetic or authorized non-sensitive data only.

## PR-02 fields to fill (owner blanks)

| Field | Value | Owner |
| --- | --- | --- |
| Host name / OS build | `<blank>` | Operations |
| Service account (least privilege) | `<blank>` | Operations + Security |
| Network boundary (who can reach the port) | Stage 1: local logon only; Stage 2 requires an approved LAN decision | Security |
| TLS (certificate, custody, renewal) | `<blank>` | Security |
| Host hardening baseline | per [deployment/security.md](../deployment/security.md) | Security |
| Capacity and cost owner | `<blank>` | Operations |
| Change approver | `<blank>` | Change approver |

## Stage 1 — single-host staging (this checklist)

Preconditions:

- [ ] Clean Git revision; commit hash recorded
- [ ] Fresh local bundle + restore drill PASS recorded in [backup-log.md](backup-log.md)
- [ ] Synthetic or authorized non-sensitive dataset only; no real customer data
- [ ] Host fields above filled and approved for **staging scope only**

Build and install:

- [ ] `cargo build --release --locked` exit 0 on the host; binary hash recorded
- [ ] Executables staged: `ekp_platform` CLI + `governance_web`
- [ ] Registered as a Windows service (`sc.exe create`); manual start only — no auto-start before Stage 2 approval
- [ ] Ledger path on a dedicated directory; backup target decided

Smoke and health (evidence per check):

- [ ] `/healthz` responds; `/ready` and `/metrics` respond where implemented (see [monitoring.md](../deployment/monitoring.md) current/target split)
- [ ] One `governance_web` login (local session, synthetic credentials) + one CLI selftest
- [ ] One end-to-end local workflow with command + output captured
- [ ] Logs land in an agreed path; no secrets in logs

Rollback:

- [ ] Stop service; restore last bundle; re-run smoke; record outcome
- [ ] Recorded decision: keep staging running / roll back

Stop conditions (do not proceed; roll back and record):

- Any real customer data on disk; missing evidence; failed health/smoke; unexplained log errors.

## Stage 2 gate (out of scope here)

LAN exposure, TLS termination and any non-synthetic data require PR-03 (identity), PR-04 (isolation) and PR-11 (security assurance) progress plus an approved change. Do not enable auto-start or open firewall ports before that approval.

## Evidence

Record every checkbox result (command, revision, output, hash) in the release evidence bundle and link it from [PR-02](production-readiness.md). A checked box without evidence is not done.
