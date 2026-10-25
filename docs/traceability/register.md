# EKPI Traceability Register

> Repository-resident single source of truth mapping requirement/issue → spec/ticket →
> code change(s) → test evidence → PR/decision. Every public number in reports
> must be derivable from a named, documented source.

| Req/Issue ID | Spec/ticket | Code change(s) | Test evidence | PR/decision | Status |
|---|---|---|---|---|---|
| S1 metrics targets | docs/metrics/targets.md | — | — | — | implemented |
| S3 local-first workflow | docs/operations/local-first-workflow.md | — | scripts/check.ps1, scripts/check.sh | — | implemented |
| S4 secrets policy | docs/operations/secrets-policy.md | .env.example | CI secret scan | — | implemented |
| S5 local backup | docs/operations/local-backup.md | docs/operations/backup-log.md | restore drill | — | implemented |
| S6 eval cases (50+) | MASTER PROMPT plan S6 | eval/cases/*.jsonl | cargo test --locked | — | planned |
| S7 check scripts | MASTER PROMPT plan S7 | scripts/check.ps1, scripts/check.sh | baseline run | — | implemented |
| S9 traceability register | docs/traceability/register.md | — | — | — | implemented |

## Derivation rule

Every public number in `report.json` / `report.md` / `engine_hard_gates.json` is
derivable from `engine_snapshots.json`, `engine_hard_gates.json`, or persisted
case row counts. In-memory counts agree with persisted counts. No number is
invented.