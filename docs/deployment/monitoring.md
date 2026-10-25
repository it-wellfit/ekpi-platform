# Monitoring and observability

- **Status:** Current state documented; production monitoring **not implemented**
- **Owner:** Operations + Engineering (operations owner not yet assigned)
- **Last reviewed:** 2026-10-26
- **Readiness gate:** [Production readiness](../operations/production-readiness.md)
- **Metrics register:** [metrics/targets.md](../metrics/targets.md)

## Current state

The local prototype has no production monitoring:

- No structured logging, log levels, `RUST_LOG` handling or log aggregation. The CLI prints operator-facing messages; the web binary prints its ledger path and bind address.
- No metrics endpoint, tracing exporter, correlation ID or request ID.
- No health/readiness endpoint for operations. (`platform/src/api.rs` defines a `/health` handler but the module is not served.)
- No dashboards, alerting, on-call or SLO/error budget.
- No centralized audit pipeline; the local ledger is the only governed-action record.

Report artifacts contain evaluation metrics such as per-system accuracy, citation recall/precision, mean latency and per-case results. Those are **offline evaluation output**, not production telemetry, and a local mean is not an SLO.

## Target requirements (Phase B)

Phase B requires an approved observability design:

| Area | Minimum requirement |
| --- | --- |
| SLI/SLO | Named service-level indicators, targets, error budget and review cadence |
| Logs | Structured, timestamped, correlated; secrets and personal data redacted; retention defined |
| Metrics | Request/decision counts, latency, error and refusal rates, queue/backlog (if any), resource use |
| Traces | End-to-end correlation for a decision or form submission, where the architecture supports it |
| Health | Liveness/readiness checks with documented semantics |
| Alerts | Actionable alerts with an owner, runbook and escalation path |
| Audit | Tamper-evident governed-action record with retention and access control |
| Incident | Severity model, notification path and drill evidence |

These are blocked by `DEC-16` (operations/service levels) and the absence of an approved deployment target. Do not invent dashboards or targets to fill the gap.

## Local diagnostic procedure

1. Run the CLI command again with the exact arguments and capture stdout, stderr and the exit code.
2. For a report/export issue, inspect `report.json`, `report.md`, `engine_hard_gates.json` and `engine_snapshots.json` in the output directory.
3. For a ledger issue, verify the hash chain on the `/audit` page; preserve a copy before any repair.
4. Record the revision, command, environment and observed result in the issue/review record.
5. If data may be lost or tampered with, follow the [incident log](../operations/incident-log.md) procedure.

## Update triggers

Update this page when logging, metrics, tracing, health checks, alerting, SLOs or the audit pipeline change. Link the implementation and evidence; do not claim production monitoring before it exists.
