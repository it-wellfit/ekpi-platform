# EKPI Traceability Register

> Repository-resident record mapping work item → spec/ticket → code change → contract →
> test evidence → release/decision. Every public number in reports must be derivable
> from a named, documented source. `Verified` means a test proves conformance;
> `Validated` means a representative user/owner confirmed fitness for the real use.
> They are not interchangeable.

- **Owner:** QA
- **Last reviewed:** 2026-10-26
- **Rules and ID scheme:** [Traceability rules](README.md)
- **Target-product baseline:** [governance/10](../governance/10-traceability-baseline.md)
- **Current release evidence:** [Regression checklist](../development/regression-checklist.md); baseline commit `a2520ed`; verified local revision `283f83a` (2026-10-26)

## Register

| Req/work item | Spec/ticket | Code change(s) | Contract(s) | Test evidence | Verified | Validated | Release/PR | Status |
|---|---|---|---|---|---|---|---|---|
| S1 Metrics targets | [metrics/targets.md](../metrics/targets.md) | `eval/src/metrics.rs`, `eval/src/report.rs` | — | Metric fields asserted by eval tests; report values derived from artifacts | Partial: M-3–M-8 targets still `proposed` | No | `a2520ed` | Implemented |
| S2 Metrics instrumentation | MASTER PROMPT plan S2; [metrics/targets.md](../metrics/targets.md) | `eval/src/metrics.rs`, `eval/src/report.rs` | CTR-DATA-005 | `cargo test --locked` (eval, 9 tests) | Yes | No | `a2520ed` | Implemented |
| S3 Local-first workflow | [operations/local-first-workflow.md](../operations/local-first-workflow.md) | `scripts/check.ps1`, `scripts/check.sh`, `scripts/backup.ps1`, `scripts/backup.sh` | — | Offline checks PASS 2026-10-26; local bundle + restore drill PASS | Yes (local scope) | No | `283f83a` | Verified |
| S4 Secrets policy | [operations/secrets-policy.md](../operations/secrets-policy.md) | `scripts/check-secrets.ps1`, `.sh`, `.env.example`, `.gitignore` | — | Local high-signal scan PASS 2026-10-26 (197 files); CI security job defined but not run on a remote | Yes (local scan) | No | `283f83a` | Verified |
| S5 Local backup | [operations/local-backup.md](../operations/local-backup.md), [backup log](../operations/backup-log.md) | `scripts/backup.ps1`, `scripts/backup.sh` | — | Restore drill PASS 2026-10-26 at `283f83a`; fresh clone passed restored `scripts/check.ps1`; bundle SHA-256 recorded; local-only | Yes (local bundle) | No | `283f83a` | Verified |
| S6 Eval corpus (67 cases) | MASTER PROMPT plan S6; [AI evaluation](../quality/ai-evaluation.md) | `eval/cases/seed_cases.jsonl` (9), `casebook_v3.jsonl` (46), `edge_cases.jsonl` (12); `platform/tests/edge_cases.rs` | CTR-DATA-005 | `cargo test --locked`; corpus-size assertion; self-test at `283f83a` | Yes (synthetic corpus only) | No | `283f83a` | Verified |
| S7 Fixed check scripts | [quality gates](../development/quality-gates.md) | `scripts/check.ps1`, `scripts/check.sh`, `gate.ps1` | — | `scripts/check.ps1` and `scripts/check.sh` PASS 2026-10-26; `gate.ps1 -IncludeSmoke` 12/12 at `283f83a` | Yes (at that revision) | No | `283f83a` | Verified |
| S9 Traceability register | This file; [traceability rules](README.md) | Documentation only | — | Manual row review 2026-10-26; no automated ID/link check yet | Partial | No | `283f83a` | Partial |
| S10 Remote collaboration criteria | [remote-collaboration-criteria.md](../remote-collaboration-criteria.md) | — | — | Criteria documented; remote/CI upgrade **not executed** | Partial: doc only | No | Pending | Blocked |
| S11 Final validation report | [operations/final-validation-report.md](../operations/final-validation-report.md) | — | — | Dated 2026-10-25 snapshot; contains stale counts and an overstated S5 claim | No: historical, not current evidence | No | `a2520ed` | Historical |
| S12 Documentation architecture | [docs/README.md](../README.md); `product/`, `architecture/`, `backend/`, `frontend/`, `contracts/`, `quality/`, `operations/` | Documentation only | — | `scripts/check-docs.ps1` / `.sh` PASS 2026-10-26: 44 required files, 427 links in 92 files | Yes | No | `283f83a` | Verified |
| S13 Phase gates and production readiness | [quality/phase-gates.md](../quality/phase-gates.md), [operations/production-readiness.md](../operations/production-readiness.md) | Documentation only | — | Manual review against code/wiring audit | Partial: gate status is an assessment, not an approval | No | `283f83a` | Implemented |
| S14 AI evaluation policy | [quality/ai-evaluation.md](../quality/ai-evaluation.md) | `eval/cases/*.jsonl` | CTR-DATA-005 | Eval tests; corpus counts; no AI/agent implementation | Partial: tool-failure dimension absent | No | `283f83a` | Implemented |
| S15 Contracts | [contracts/README.md](../contracts/README.md), [CLI](../contracts/cli.md), [web routes](../contracts/web-routes.md), [data formats](../contracts/data-formats.md) | `platform/src/main.rs`, `platform/src/governance_web.rs`, `platform/src/governance.rs`, `eval/src/main.rs` | CTR-CLI-001, CTR-CLI-002, CTR-WEB-001, CTR-DATA-001–006, CTR-EXP-001 | `--selftest` (49 assertions), schema (20 records), governance route tests (3), smoke 2026-10-26 at `283f83a` | Yes for served CLI/web/data; placeholder API explicitly excluded | No | `283f83a` | Verified |
| S16 Threat model | [architecture/threat-model.md](../architecture/threat-model.md) | — | — | Code/wiring audit; no penetration test or independent security review | Partial: self-assessment | No | `283f83a` | Implemented |
| S17 Reviews and improvements | [reviews/README.md](../reviews/README.md), [bootstrap retro](../reviews/bootstrap-retro.md) | — | — | Self-review only; no independent facilitator; IMP-001–004 closed with evidence | Partial | No | `283f83a` | Implemented |

## Release evidence

| Revision | Date | Scope | Evidence | Result |
|---|---|---|---|---|
| `a2520ed` | 2026-10-25 | Initial workflow/skills/quality-gate bootstrap | `gate.ps1` 10/10; eval 9 tests + 10 self-test checks; platform 183 tests + 49 self-test assertions; schema 20 records; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes, 19/19/19/16; `report.html` zero script | Passed for the tested revision |
| `283f83a` | 2026-10-26 | Documentation architecture, phase gates, contracts, checks, local restore drill | `scripts/check.ps1` and `scripts/check.sh` PASS; docs 44 required/427 links in 92 files; secret scan 197 files; `gate.ps1 -IncludeSmoke` 12/12; eval 9 tests + 10 self-test checks; platform 183 tests + 49 assertions; schema 20; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes 19/19/19/16; `report.html` zero script; fresh clone of bundle `21D2CD…ED443` passed the restored check | Passed for the tested local revision; no CI/remote run; uncommitted work excluded |

## Status vocabulary

| Status | Meaning |
|---|---|
| `Planned` | Accepted work with no implementation evidence yet |
| `Implemented` | Code/docs exist, but verification or validation is incomplete |
| `Verified` | A named test proves the implementation matches the spec at the named revision |
| `Validated` | A representative user/owner confirmed it fits the real use; named session required |
| `Partial` | Some evidence exists; the row's full scope is not met |
| `Blocked` | Cannot proceed without an external decision, access, owner or environment |
| `Historical` | Kept for the record; not current evidence |

## Derivation rule

Every public number in `report.json` / `report.md` / `engine_hard_gates.json` is derivable from `engine_snapshots.json`, `engine_hard_gates.json`, or persisted case row counts. In-memory counts agree with persisted counts. No number is invented.

A target acceptance case (`AT-*`) remains `NOT RUN` until executed on the target product. A passing prototype unit test does not satisfy it.

## Open register gaps

- No row is `Validated`; no real pilot session has occurred.
- S10 is blocked by the missing remote/authentication. S5 is verified only for a local bundle; off-host encrypted backup, approved RPO/RTO and scheduled restore tests remain Phase B (`B7`).
- S11 is historical and must not be cited for current counts.
- The register is reviewed manually; an automated ID/link check is future work.
- The restore drill protects committed history only. Uncommitted work, the external governance ledger and ignored secrets are outside the bundle; commit and copy them separately when required.
