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
- **Current release evidence:** [Regression checklist](../development/regression-checklist.md); baseline commit `a2520ed`; verified local revision `c131815` (clean-checkout gate 13/13); latest backup bundle revision `198d5f3` (restore blocked, see [backup log](../operations/backup-log.md)) (2026-10-26)

## Register

| Req/work item | Spec/ticket | Code change(s) | Contract(s) | Test evidence | Verified | Validated | Release/PR | Status |
|---|---|---|---|---|---|---|---|---|
| S1 Metrics targets | [metrics/targets.md](../metrics/targets.md) | `eval/src/metrics.rs`, `eval/src/report.rs` | — | Metric fields asserted by eval tests; report values derived from artifacts | Partial: M-3–M-8 targets still `proposed` | No | `a2520ed` | Implemented |
| S2 Metrics instrumentation | MASTER PROMPT plan S2; [metrics/targets.md](../metrics/targets.md) | `eval/src/metrics.rs`, `eval/src/report.rs` | CTR-DATA-005 | `cargo test --locked` (eval, 9 tests) | Yes | No | `a2520ed` | Implemented |
| S3 Local-first workflow | [operations/local-first-workflow.md](../operations/local-first-workflow.md) | `scripts/check.ps1`, `scripts/check.sh`, `scripts/backup.ps1`, `scripts/backup.sh` | — | Offline checks PASS 2026-10-26; local bundle + restore drill PASS at `283f83a`; latest `198d5f3` restore blocked (see S5) | Partial: latest backup restore is not verified | No | `198d5f3` | Partial |
| S4 Secrets policy | [operations/secrets-policy.md](../operations/secrets-policy.md) | `scripts/check-secrets.ps1`, `.sh`, `.env.example`, `.gitignore` | — | Local high-signal scan PASS 2026-10-26 (200 files at `c131815`, clean tree); CI security job defined but not run on a remote | Yes (local scan) | No | `c131815` | Verified |
| S5 Local backup | [operations/local-backup.md](../operations/local-backup.md), [backup log](../operations/backup-log.md) | `scripts/backup.ps1`, `scripts/backup.sh` | — | Restore drill PASS 2026-10-26 at `283f83a`; fresh clone passed restored `scripts/check.ps1`; latest `198d5f3` bundle clone reached HEAD and passed docs/secret, but the full check was **BLOCKED** by Windows Application Control (`os error 4551`); bundle SHA-256 recorded; local-only | Partial: prior drill PASS, latest bundle unverified | No | `198d5f3` | Partial |
| S6 Eval corpus (67 cases) | MASTER PROMPT plan S6; [AI evaluation](../quality/ai-evaluation.md) | `eval/cases/seed_cases.jsonl` (9), `casebook_v3.jsonl` (46), `edge_cases.jsonl` (12); `platform/tests/edge_cases.rs` | CTR-DATA-005 | `cargo test --locked`; corpus-size assertion; self-test at `283f83a` | Yes (synthetic corpus only) | No | `283f83a` | Verified |
| S7 Fixed check scripts | [quality gates](../development/quality-gates.md) | `scripts/check.ps1`, `scripts/check.sh`, `gate.ps1` | — | `scripts/check.ps1` and `scripts/check.sh` PASS 2026-10-26; `gate.ps1 -IncludeSmoke` 13/13 at `c131815` including the fixed search gate | Yes (at that revision) | No | `c131815` | Verified |
| S9 Traceability register | This file; [traceability rules](README.md) | Documentation only | — | Manual row review 2026-10-26; no automated ID/link check yet | Partial | No | `283f83a` | Partial |
| S10 Remote collaboration criteria | [remote-collaboration-criteria.md](../remote-collaboration-criteria.md) | — | — | Criteria documented; remote/CI upgrade **not executed** | Partial: doc only | No | Pending | Blocked |
| S11 Final validation report | [operations/final-validation-report.md](../operations/final-validation-report.md) | — | — | Dated 2026-10-25 snapshot; contains stale counts and an overstated S5 claim | No: historical, not current evidence | No | `a2520ed` | Historical |
| S12 Documentation architecture | [docs/README.md](../README.md); `product/`, `architecture/`, `backend/`, `frontend/`, `contracts/`, `quality/`, `operations/` | Documentation only | — | `scripts/check-docs.ps1` / `.sh` PASS 2026-10-26: 44 required files, 445 links in 92 files at `c131815` | Yes | No | `c131815` | Verified |
| S13 Phase gates and production readiness | [quality/phase-gates.md](../quality/phase-gates.md), [operations/production-readiness.md](../operations/production-readiness.md) | Documentation only | — | Manual review against code/wiring audit | Partial: gate status is an assessment, not an approval | No | `283f83a` | Implemented |
| S14 AI evaluation policy | [quality/ai-evaluation.md](../quality/ai-evaluation.md) | `eval/cases/*.jsonl` | CTR-DATA-005 | Eval tests; corpus counts; no AI/agent implementation | Partial: tool-failure dimension absent | No | `283f83a` | Implemented |
| S15 Contracts | [contracts/README.md](../contracts/README.md), [CLI](../contracts/cli.md), [web routes](../contracts/web-routes.md), [data formats](../contracts/data-formats.md) | `platform/src/main.rs`, `platform/src/bin/governance_web.rs`, `platform/src/governance.rs`, `eval/src/main.rs`, `platform/src/eval.rs` | CTR-CLI-001, CTR-CLI-002, CTR-WEB-001, CTR-DATA-001–007, CTR-EXP-001 | `--selftest` (49 assertions), schema (20 records), governance route tests (3), `--eval --test-set data/eval/search_eval.json` 24/24, smoke 2026-10-26 | Yes for served CLI/web/data; placeholder API explicitly excluded | No | `9d0d793` | Verified |
| S16 Threat model | [architecture/threat-model.md](../architecture/threat-model.md) | — | — | Code/wiring audit; no penetration test or independent security review | Partial: self-assessment | No | `283f83a` | Implemented |
| S17 Reviews and improvements | [reviews/README.md](../reviews/README.md), [bootstrap retro](../reviews/bootstrap-retro.md) | — | — | Self-review only; no independent facilitator; IMP-001–004 closed with evidence | Partial | No | `283f83a` | Implemented |
| S18 Search and citations CLI | [contracts/cli.md](../contracts/cli.md), [architecture data flow](../architecture/README.md#knowledge-search-and-retrieval-evaluation) | `platform/src/retrieval.rs`, `platform/src/main.rs` | CTR-CLI-001 | `--search` real run 2026-10-26; retrieval unit tests; citation text is verbatim | Yes (lexical, local) | No | `9d0d793` | Implemented |
| S19 Permission-aware retrieval (library) | [data formats](../contracts/data-formats.md#documents), [threat model TH-13](../architecture/threat-model.md) | `platform/src/docs.rs`, `platform/src/retrieval.rs`, `platform/src/eval.rs` | CTR-DATA-001 | `search_respects_permissions`, `eval_permission_isolation`; fixed set permission isolation 1.00 | Yes (library filter only) | No | `9d0d793` | Implemented; **not** authorization or tenant isolation |
| S20 Fixed search evaluation gate | [AI evaluation](../quality/ai-evaluation.md), [data formats](../contracts/data-formats.md#search-evaluation-set) | `platform/src/eval.rs`, `platform/src/main.rs`, `platform/data/eval/search_eval.json`, `platform/tests/search_eval.rs` | CTR-DATA-007 | 6 integration tests; CLI `--eval` 24 cases, 18/18 top-1, 6/6 refusals, gate PASS | Yes (deterministic lexical regression) | No | `9d0d793` | Verified |

## Release evidence

A row records the revision that was actually tested. The row is committed after that run, so it is not present in the tested revision itself; the code and scripts are unchanged by adding the row. Verify the named revision from Git history, not from the presence of its own evidence row.

| Revision | Date | Scope | Evidence | Result |
|---|---|---|---|---|
| `a2520ed` | 2026-10-25 | Initial workflow/skills/quality-gate bootstrap | `gate.ps1` 10/10; eval 9 tests + 10 self-test checks; platform 183 tests + 49 self-test assertions; schema 20 records; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes, 19/19/19/16; `report.html` zero script | Passed for the tested revision |
| `283f83a` | 2026-10-26 | Documentation architecture, phase gates, contracts, checks, local restore drill | `scripts/check.ps1` and `scripts/check.sh` PASS; docs 44 required/427 links in 92 files; secret scan 197 files; `gate.ps1 -IncludeSmoke` 12/12; eval 9 tests + 10 self-test checks; platform 183 tests + 49 assertions; schema 20; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes 19/19/19/16; `report.html` zero script; fresh clone of bundle `21D2CD…ED443` passed the restored check | Passed for the tested local revision; no CI/remote run; uncommitted work excluded |
| `4eacf5c` | 2026-10-26 | Knowledge search/citations, permission-aware retrieval library, fixed 24-case search gate, docs, check integration | Fresh clone: `scripts/check.ps1` PASS; docs 44 required/444 links in 92 files; secret scan 202 files; eval 9 tests + 10 self-test checks; platform 212 tests (193 unit, 2 edge, 3 web, 8 integration, 6 search_eval) + 49 assertions; schema 20; `--eval --test-set data/eval/search_eval.json` 24 cases, 18/18 top-1, 6/6 refusals, permission isolation 1.00, gate PASS; `gate.ps1 -IncludeSmoke` 13/13; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes 19/19/19/16; `report.html` zero script; `cargo audit` both lockfiles 0 vulnerabilities | Passed for the tested local revision; search is lexical and `--search` is not authorization; no CI/remote run; no AI/LLM evaluation; uncommitted work excluded |
| `f34c0d6` | 2026-10-26 | Search/retrieval documentation and evidence update | Fresh clone: docs 44 required/444 links in 92 files; secret scan 201 files; `gate.ps1 -IncludeSmoke` 13/13; platform 212 tests + 49 self-test assertions; eval 9 tests + 10 self-test checks; schema 20; search gate 24/24; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes 19/19/19/16; `report.html` zero script; `cargo audit` both lockfiles 0 vulnerabilities | Passed for the tested local revision; docs-only difference from `4eacf5c`; no CI/remote run |
| `198d5f3` | 2026-10-26 | Backup bundle and restore attempt for the docs-evidence revision | Bundle `ekpi-platform-2026-10-26-020948.bundle`, SHA-256 `E6EAB351…D903847`; `git bundle verify` okay; fresh clone reached `198d5f3`; docs 44 required/444 links in 92 files and secret scan 201 files (200 repository files plus the local drill log) PASS; full restored check **BLOCKED** at eval clippy: Windows Application Control denied freshly built build scripts (`os error 4551`); code-identical `f34c0d6` (docs-only diff) passed clean-clone `gate.ps1 -IncludeSmoke` 13/13; no CI/remote run | **Partial**: code revision inherits the `f34c0d6` gate, but the latest bundle restore is not verified; A8 stays `Partial` |
| `3a5ac47` | 2026-10-26 | Blocked-restore documentation update | Separate clean checkout with no local logs: docs 44 required/445 links in 92 files; secret scan 200 files; `scripts/check.ps1` PASS; `gate.ps1 -IncludeSmoke` 13/13; eval 9 tests + 10 self-test checks; platform 212 tests + 49 self-test assertions; schema 20; search gate 24/24; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes 19/19/19/16; `report.html` zero script | Passed for the tested local revision; docs-only difference from `198d5f3`; latest bundle restore remains **BLOCKED**; no CI/remote run |
| `ff86d83` | 2026-10-26 | Final verification and backup-status documentation | Separate clean checkout with no local logs: docs 44 required/445 links in 92 files; secret scan 200 files; `scripts/check.ps1` PASS; `gate.ps1 -IncludeSmoke` 13/13; eval 9 tests + 10 self-test checks; platform 212 tests + 49 self-test assertions; schema 20; search gate 24/24; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes 19/19/19/16; `report.html` zero script | Passed for the tested local revision; docs-only difference from `3a5ac47`; latest `198d5f3` bundle restore remains **BLOCKED**; no CI/remote run |
| `c131815` | 2026-10-26 | Final traceability and phase-gate documentation | Separate clean checkout with no local logs: docs 44 required/445 links in 92 files; secret scan 200 files; `scripts/check.ps1` PASS; `gate.ps1 -IncludeSmoke` 13/13; eval 9 tests + 10 self-test checks; platform 212 tests + 49 self-test assertions; schema 20; search gate 24/24; smoke `G_ekpi_engine acc=1.000`, H-1–H-6 OK; LegalRuleML 51,026 bytes 19/19/19/16; `report.html` zero script | Passed for the tested local revision; docs-only difference from `ff86d83`; latest `198d5f3` bundle restore remains **BLOCKED**; no CI/remote run |

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
- S10 is blocked by the missing remote/authentication. S5 has a PASS for the `283f83a` bundle only; the `198d5f3` bundle restore is blocked by host Application Control and is not verified. Off-host encrypted backup, approved RPO/RTO and scheduled restore tests remain Phase B (`B7`).
- S11 is historical and must not be cited for current counts.
- The register is reviewed manually; an automated ID/link check is future work.
- The restore drill protects committed history only. Uncommitted work, the external governance ledger and ignored secrets are outside the bundle; commit and copy them separately when required.
