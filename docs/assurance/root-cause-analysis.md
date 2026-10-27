# Root-cause analysis

- **Status:** Active; root causes recorded, several remain externally blocked
- **Owner:** Engineering + QA; host/security and business owners `UNASSIGNED`
- **Last reviewed:** 2026-10-27
- **Evidence baseline:** `6ae6134`; release-profile checks re-run 2026-10-27
- **Matrix:** [phase-a-b-matrix.md](phase-a-b-matrix.md)
- **Test evidence:** [test-evidence.md](test-evidence.md)

This analysis separates a code defect from a test-environment block, a missing capability and a missing external decision. The distinction matters because only the first is fixable by changing this repository.

## Summary

| ID | Symptom | Root cause | Class | Fix / verification | Remaining risk |
| --- | --- | --- | --- | --- | --- |
| RC-01 | Canonical `scripts/check.ps1` exits 1 in the working tree at the platform test | Windows Application Control blocks the existing unsigned debug `ekp_platform.exe` (`os error 4551`); 22 CLI contract tests cannot spawn it | Environment / host policy | Release-profile suite runs the same 22 tests (270 total, exit 0); at `41beaea` a fresh clone ran the canonical check to exit 0; the working-tree run remains `BLOCKED` | Working-tree reproducibility requires an exception or CI runner |
| RC-02 | The `28b68cc` restore failed in a fresh clone | Application Control blocked fresh-clone build scripts (`proc-macro2`, `serde_core`, `serde`, `quote`, `zmij`) at `eval` Clippy | Environment / host policy | `28b68cc` stays `BLOCKED`; the `41beaea` final bundle verifies, clones and passes the automatic restore drill with canonical check exit 0 | Earlier revision remains unverified; off-host/RPO/RTO absent |
| RC-03 | CLI flags were silently ignored or combined | Permissive argument parsing treated missing values and unrelated flags as defaults or no-ops | Code defect | `23b8996` hardened the contract; 22 CLI contract tests pass in release profile and in the `41beaea` clean-clone canonical run | Working-tree canonical execution blocked by RC-01 |
| RC-04 | `--llm` could imply a model capability that did not exist | No provider adapter; a mock was the only implementation, and the output needed explicit labeling | Code/contract and scope | `--llm` accepts only `mock`, requires `--search`, labels output `mock-answer`, requires `[doc_id@version]`; other providers exit 2 before data/audit | No AI quality claim; no real provider |
| RC-05 | `--audit-log` could be mistaken for an append-only audit service | The CLI appends one line to a local JSONL hash chain, verifies the chain before append and fails closed on corrupt/legacy files; there is no authenticated actor or durable audit store | Scope / documentation | Contract states the local-only behavior; `--audit-log` requires `--search`; actor is fixed `search-cli`; tamper/legacy/concurrency tests pass | Not WORM, signed or an external audit service; ledger is not WORM |
| RC-06 | Legacy `--import-wellfit` could overwrite unreadable data or mix modes | The importer did not fail closed on an unreadable `docs.json` and could be combined with unrelated modes | Code defect | `--import-wellfit` is standalone; unreadable input exits 1 without overwrite; imported documents go through `validate_shape()` | Legacy path remains a local migration tool |
| RC-07 | Local login could be described as enterprise authentication | The implementation is a single startup-configured account with process-memory sessions; it has no IdP, MFA or persistent identity | Scope / missing external integration | Documentation and tests state the local boundary; Viewer/Editor/Approver checks and session hardening are tested | A5 stays `PARTIAL`; no tenant isolation |
| RC-08 | No CI, PR or branch protection evidence | The repository has no remote and `gh` is not authenticated | External access | CI workflow and local gate exist; no remote execution | B4 `BLOCKED`; actions are tag-pinned |
| RC-09 | No real pilot or independent review | No named business owner, representative user or independent reviewer | External ownership | Pilot runbook and reviewer checklist prepared | A1 `NOT MET`; A11 `PARTIAL` |
| RC-10 | No AI/agent evaluation | No approved provider/data policy or tool path; only a deterministic lexical retriever and a mock provider exist | Missing capability / external decision | Fixed eval policy and 28-case regression gate exist | A9 `PARTIAL`; B5 `BLOCKED` |
| RC-11 | No off-host backup or approved RPO/RTO | No approved destination, key custodian, retention decision or operations owner | External decision / environment | Local bundle, restore drill and local-simulated `age` drill exist | B7 `PARTIAL`; A8 final restore blocked |
| RC-12 | No SLO, soak or capacity model | No approved workload, user profile, SLO or cost budget | External decision | One-host benchmark exists | B2/B3 `PARTIAL`; exploratory only |

## RC-01: canonical check versus release profile

### Observed evidence

At `6ae6134`, on Windows PowerShell 5.1:

```text
cargo test --locked   (platform)
running 22 tests
... 22 failed ...
run ekp_platform: Os { code: 4551, kind: Uncategorized, message: "應用程式控制原則已封鎖此檔案。" }
FAIL: platform test
CANONICAL_EXIT=1
```

The same revision in the release profile:

```text
cargo test --release --locked   (platform)
217 unit + 22 CLI contract + 2 edge + 10 governance web + 8 integration + 7 search + 4 web health = 270 passed
PLATFORM_EXIT=0
```

`eval` has 12 tests in both profiles; self-test, schema and the 28-case search gate pass in release.

At `41beaea` and `6ed236d`, the automatic restore drill built a fresh debug binary in a clean clone and the canonical `scripts/check.ps1` exited 0 (at `6ed236d`: docs 652/105, secrets 229, platform 270, eval 12, self-test 49, schema 20, search 28). The working tree at those two revisions still exited 1 at the 22 CLI contract tests because its existing `platform\target\debug\ekp_platform.exe` was denied. Later working-tree runs at `4697aa1` and `718db57` passed the full canonical check; those are recorded separately in E-024/E-025, so the earlier clean-clone pass was local and revision-bound, not CI.

### Root cause

The host's Application Control policy can deny a newly created or existing unsigned debug executable at process creation (`ERROR_VIRUS_INFECTED` / code 4551 is used here as the host's denial code). The denial is path/artifact-specific: the working-tree binary was denied while a freshly built clone binary was allowed. The failure occurs before the test body runs, so it is not an assertion failure or a product regression. The release binary and the clone build were executable, which is why the same tests pass there.

### Why this was not "fixed" by changing the gate

- Replacing canonical with release-only would lower the required entry point and hide the host-policy failure.
- Setting `EKP_PLATFORM_BIN` to the release binary makes the test suite pass but does not make `cargo run --locked --bin ekp_platform -- --selftest` runnable; the canonical script still fails at the self-test step.
- Skipping or deleting the 22 tests would remove contract coverage.
- The correct fix is an approved code-integrity/Application Control exception for the build output, or a canonical run on an approved CI/POSIX host.

### Regression guard

The existing `scripts/check.ps1` already fails non-zero and prints the failing command. The distinction between `FAIL` and `BLOCKED` is recorded in this assurance tree; it is not silently converted. A future improvement may add an explicit 4551 preflight message, but it must still exit non-zero.

## RC-02: final bundle restore

### Observed evidence

Earlier blocked attempt:

- Bundle: `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-27-132905.bundle`
- SHA-256: `B9E398CAF38DCC0311DFE371E735846021D573D58312C8F9A8D0613392031C58`
- `git bundle verify`: okay, complete history
- Manual clone `%USERPROFILE%\EKPI-restore-drill-28b68cc`: HEAD `28b68cc`; docs 505 links in 96 tracked files PASS; secrets 220 PASS
- Restored `scripts/check.ps1`: failed at `eval` Clippy with `proc-macro2` build-script denial (`os error 4551`)

Final evidence bundle:

- Bundle: `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-27-141734.bundle`
- SHA-256: `E3E8C3EE2DF8980A46008A14A7AA1A820D192DA47A99B8ABF0079ADEB3EEA13E`
- `git bundle verify`: okay, complete history, HEAD `6ed236d`
- Automatic TEMP drill: clone HEAD `6ed236d`; restored `scripts/check.ps1` exit 0; `RESTORE DRILL PASSED`
- Clean-clone canonical counts: platform 270, eval 12, self-test 49, schema 20, search 28, docs 652/105, secrets 229

### Root cause

A fresh clone has no prebuilt `target/` artifacts. Cargo must compile and execute build scripts (proc-macro crates). Application Control can deny those newly built, unsigned executables. This is the same policy as RC-01, applied at a different stage. The bundle was not corrupt and the revision was present; the environment intermittently refused to execute the build. At `41beaea` the policy allowed the build and the restored check completed.

### Why `3ee2ee2` passed and `28b68cc` did not

The policy is intermittent and path/artifact dependent. `3ee2ee2` was restore-verified by a manual clone before the policy began denying the relevant build scripts; `28b68cc` was blocked later. `41beaea` and `6ed236d` then passed automatically. Historical PASSes and blocks do not transfer across revisions. The `c77fec4`, `bc7671a` and `28b68cc` rows remain `BLOCKED` and are not counted; `6ed236d` is the current verified restore.

### Correct handling

- Keep the `28b68cc` row `BLOCKED`, not PASS; keep the `41beaea` and `6ed236d` rows as local automatic-restore PASSes.
- Record `6ed236d` as the current restore-verified revision; keep `41beaea` and `3ee2ee2` as previous verified revisions.
- Keep the working-tree canonical block visible; the clean-clone PASS does not transfer to the working-tree path.
- Re-run the restore in an approved environment and record each new result in the [backup log](../operations/backup-log.md).
- Do not rewrite the bundle, reset history or bypass the check to manufacture a pass.

## RC-03: CLI contract hardening

### Symptom

Before `23b8996`, flags such as `--out` without a value, `--cases` without `--out`, report flags combined with `--eval`, or `--llm` without `--search` could be accepted with a default, ignored, or combined in a way that made the command's behavior ambiguous. The audit and import paths also needed fail-closed behavior.

### Root cause

Argument parsing was permissive: an absent value fell back to a default, and modes were not mutually exclusive. Tests covered happy paths more than negative contracts.

### Fix and evidence

Commit `23b8996` (`feat(cli): enforce the mock-only LLM and local audit contract`) changed `platform/src/main.rs` and expanded `platform/tests/cli_contract.rs` from 13 to 22 tests. The verified behavior:

- `--llm` requires `--search`; only `mock` is accepted; other providers exit `2` before data load or audit write; output is `mock-answer` and requires a versioned citation.
- `--audit-log` requires `--search`; it appends one JSONL `KnowledgeSearch` event (`actor: search-cli`, `tenant_id: default`) to a verified hash chain, refuses corrupt/legacy files with exit `1`, and serialises local writers on a sibling lock directory.
- `--import-wellfit` is standalone; it fails closed on an unreadable `docs.json`, does not overwrite it, and validates the imported documents before saving.
- Missing flag values, unknown flags and mixed modes are usage errors (exit `2`), and do not create report output.

The 22 CLI contract tests pass in the release profile at `6ae6134` (E-002), in the `41beaea`/`6ed236d` clean-clone canonical runs (E-020/E-023), and in the later working-tree canonical runs at `4697aa1`/`718db57` (E-024/E-025; the suite grew to 24 tests at `718db57`). The earlier working-tree block remains historical. The pass is local and revision-bound, not CI.

## RC-04/RC-05/RC-06: mock LLM, local audit and legacy import boundaries

These are related scope decisions, not evidence of AI or audit capability:

- `rag.rs` contains only `MockLlmProvider`. The CLI labels the output `mock-answer` and requires a citation of the form `[doc_id@version]`. This proves the contract and refusal behavior, not model quality.
- `--audit-log` appends one JSONL event to a hash chain that is verified before each append, and refuses corrupt or legacy single-object files with exit `1`; the actor is the fixed label `search-cli`. It is not authenticated, WORM, signed or externally anchored, and it remains a local diagnostic record, not an audit service. The governance ledger itself is an append-only JSONL hash chain, but it is not WORM and a filesystem administrator can rewrite it.
- `--import-wellfit` is a legacy local migration path. Its fail-closed behavior is tested; it is not a supported ingestion service.

## RC-07: local authentication boundary

`local_auth` was implemented to remove the "no login at all" state from the local workbench, not to satisfy enterprise identity:

- one account registered at startup from `EKPI_LOCAL_AUTH_USER` / `EKPI_LOCAL_AUTH_PASSWORD`; role from `EKPI_LOCAL_AUTH_ROLE` (default `editor`); no default account;
- PBKDF2-HMAC-SHA256, 16-byte random salt, 600,000 iterations; 32-byte CSPRNG session token; constant-time comparison;
- five failed logins lock an existing account for 300 seconds; unknown usernames do not grow the lockout map;
- sessions live in process memory, expire after 3,600 seconds and are pruned on login; restart clears them;
- `ekpi_session` is `HttpOnly`, `SameSite=Strict`, `Path=/`; `Secure` requires `EKPI_COOKIE_SECURE=1`;
- every ledger-backed GET requires Viewer; creation requires Editor; review/publish/quality/lifecycle requires Approver; public paths are `/healthz`, `/metrics`, `/login`, `/assets/*`; `/logout` requires a session;
- mutations with a foreign `Origin` are refused; a missing `Origin` is accepted only with `Sec-Fetch-Site: same-origin` or `none`; form bodies are capped at 64 KiB.

Residual root cause: there is no IdP, MFA, recovery, persistent identity, access review or tenant boundary. The ledger actor is the validated local session username for web mutations (fixed `search-cli` for the CLI audit), but the session is process-memory only, so this is not an enterprise-verified identity. This is an architectural scope limit, not a bug to be hidden by wording.

## RC-08 to RC-12: external dependencies

| Root cause | What would be needed | What can be done now |
| --- | --- | --- |
| No remote/CI | A repository remote, authenticated CI, branch protection, required checks | Keep the workflow and local gates current; record `BLOCKED`; do not claim CI |
| No pilot | Named business owner and representative user; approved non-sensitive workflow | Pilot runbook, templates and synthetic workflow automation |
| No independent review | A reviewer who did not implement the change; second annotator for κ | Reviewer checklist, evidence index and reproducible commands |
| No AI evaluation | Approved provider, data policy, tool path and model/prompt/index versioning | Fixed deterministic corpus and mock contract tests; no AI claim |
| No off-host backup | Approved destination, key custody, retention, RPO/RTO and operations owner | Local bundle, restore drill and local-simulated `age` drill |
| No SLO/capacity | Approved workload, SLI/SLO, cost budget and target environment | One-host benchmark; label it exploratory |

## Verification of fixes

The code fixes in RC-03 to RC-06 are covered by the 24 CLI contract tests and the 12 eval tests at `718db57`, where the canonical debug profile passed; the earlier `23b8996` evidence was release-profile only. The local-auth fixes are covered by 11 governance-web route tests and 11 `local_auth` unit tests. These are `LOCAL`; no independent reviewer has reproduced them, and the suite's local pass does not make it CI, staging or production evidence.
