# Code Review Checklist

## Before Submitting

- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy --locked --all-targets -- -D warnings` passes (zero warnings)
- [ ] `cargo test --locked` passes (all tests)
- [ ] `--selftest` passes (all assertions)
- [ ] No `unsafe` code (forbidden by `#![forbid(unsafe_code)]`)
- [ ] No new dependencies without `standards.csv` entry
- [ ] All public APIs have doc comments
- [ ] Error handling uses `Result<T, E>`, not panics
- [ ] No secrets or credentials in source code
- [ ] No commented-out code
- [ ] Test coverage for new functionality
- [ ] Evidence chain preserved (citations, versions, hashes)
- [ ] No conflation of normative dependency / process order / true causation

## Review Questions

1. Does this change preserve determinism?
2. Does this change maintain evidence traceability?
3. Does this change handle insufficient evidence correctly?
4. Does this change respect the fail-closed principle?
5. Does this change introduce any security risks?
