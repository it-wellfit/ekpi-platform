# ekpi-platform instructions

Read [README.md](README.md) for the system map and run commands. For product changes, read the relevant governance requirement, data/state model, and decision register before changing behavior. Use the issue/spec workflow in [docs/development/README.md](docs/development/README.md), and load [docs/development/quality-gates.md](docs/development/quality-gates.md) for the change's risk level.

## Architecture and invariants

- The eval/ and platform/ directories are independent Rust crates; platform depends on eval. There is no root Cargo workspace. In each crate run cargo fmt -- --check, cargo clippy --locked --all-targets -- -D warnings, and cargo test --locked. Also run `cargo run --locked -- --selftest` in eval/ and `cargo run --locked --bin ekp_platform -- --selftest` plus `--declared-rules-schema` in platform/. See docs/development/quality-gates.md for exact working directories; preserve docs/development/regression-checklist.md.
- Rust code forbids unsafe; Clippy must pass with warnings denied. The platform presentation layer stays semantic HTML and CSS-first with zero JavaScript.
- Treat source evidence and test results as observed facts: do not fabricate integrations, permissions, approvals, scan results, or compliance claims. The governance web prototype is local-only and has no enterprise authentication.

## Agent skills

### Issue tracker

Work items and specs use GitHub Issues. See docs/agents/skill-setup.md for plugin setup and docs/agents/issue-tracker.md before publishing issues or PRs.

### Triage labels

Use the five canonical Matt Pocock triage labels. Their mapping and current provisioning status are in docs/agents/triage-labels.md.

### Domain docs

Use the single-context root glossary and docs/adr/; read docs/agents/domain.md for canonical product terminology and source documents.
