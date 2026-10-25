# ekpi-platform instructions

Read [README.md](README.md) for the system map and run commands. Start documentation work at the [documentation map](docs/README.md): it names the canonical source, owner and update trigger for each topic. For product changes, read the relevant governance requirement, data/state model and decision register before changing behavior. Use the issue/spec workflow in [docs/development/README.md](docs/development/README.md), and load [docs/development/quality-gates.md](docs/development/quality-gates.md) for the change's risk level.

## Architecture and invariants

- The eval/ and platform/ directories are independent Rust crates; platform depends on eval. There is no root Cargo workspace. In each crate run cargo fmt -- --check, cargo clippy --locked --all-targets -- -D warnings, and cargo test --locked. Also run `cargo run --locked -- --selftest` in eval/ and `cargo run --locked --bin ekp_platform -- --selftest`, `--declared-rules-schema` plus `--eval --test-set data/eval/search_eval.json` in platform/. See docs/development/quality-gates.md for exact working directories; preserve docs/development/regression-checklist.md.
- Rust code forbids unsafe; Clippy must pass with warnings denied. The platform presentation layer stays semantic HTML and CSS-first with zero JavaScript.
- Treat source evidence and test results as observed facts: do not fabricate integrations, permissions, approvals, scan results, or compliance claims. The governance web prototype is local-only and has no enterprise authentication.
- Only the CLI and `governance_web` are served surfaces. `platform/src/api.rs` is an unwired placeholder that returns a fixed `comply`; never expose it or call it a contract. `local_auth`, `tenant`, `audit`, `worm`, `agents`, `formal_verification`, `runtime_verification` and similar modules are library-only unless the [architecture map](docs/architecture/README.md) says otherwise.
- The local ledger is an append-only JSONL hash chain, not WORM. It records operator-entered actor strings, not authenticated identity.
- Only `EKP_GOVERNANCE_DATA` is read by the current code. Do not document or rely on other `.env.example` names, `DATABASE_URL` or `RUST_LOG`.

## Documentation and traceability

- [docs/README.md](docs/README.md) is the canonical index. Do not create a second copy of a rule; link to its owner and state only the local consequence.
- Product requirements, acceptance cases and business decisions live in [docs/governance/](docs/governance/README.md). Engineering ADRs live in [docs/adr/](docs/adr/); the [decisions index](docs/decisions/README.md) explains when to add one.
- Record every accepted requirement/change in the [traceability register](docs/traceability/register.md) with spec, code, contract, test evidence and verified/validated status. A passing synthetic test verifies conformance; it does not validate real user value.
- Phase status is recorded in [phase gates](docs/quality/phase-gates.md): Phase A is **not approved**; Phase B is **blocked**. Never describe the repository as production-ready, secure, tenant-isolated, compliant or validated without linked evidence.
- Historical summaries (`docs/requirements-spec.md`, `docs/success-metrics.md`, `docs/risk-model.md`, `docs/operations/final-validation-report.md`) are not current evidence. The canonical documents named in [docs/README.md](docs/README.md) win.

## Checks

- Fixed entry point: `scripts/check.ps1` (Windows) or `scripts/check.sh` (POSIX). It includes documentation-link and high-signal secret checks, format, Clippy, tests, self-tests and schema validation.
- Stronger pre-PR/release gate: `gate.ps1`; add `-IncludeSmoke` for report, export, coverage, HTML or data changes.
- A check is `PASS` only if the exact command ran on the named revision and exited 0. A skipped or blocked check is `Not run`/`Blocked`, never `PASS`.
- Record release evidence, backup/restore results and review findings in [docs/reviews/](docs/reviews/README.md) and [docs/operations/backup-log.md](docs/operations/backup-log.md). Do not lower a gate to make the current state look approved.

## Agent skills

### Issue tracker

Work items and specs use GitHub Issues. See [docs/agents/skill-setup.md](docs/agents/skill-setup.md) for plugin setup and [docs/agents/issue-tracker.md](docs/agents/issue-tracker.md) before publishing issues or PRs.

### Triage labels

Use the five canonical Matt Pocock triage labels. Their mapping and current provisioning status are in [docs/agents/triage-labels.md](docs/agents/triage-labels.md).

### Domain docs

Use the single-context root glossary and docs/adr/; read [docs/agents/domain.md](docs/agents/domain.md) for canonical product terminology and source documents. The [community skill assessment](docs/agents/community-skills-assessment.md) records why no additional bundle is installed.
