# Troubleshooting

## Common Issues

### Build fails with "could not find Cargo.toml"
- Ensure you're in the `platform/` or `eval/` directory, not the root
- There is no root workspace; each crate is independent

### Clippy fails with warnings
- Run `cargo fmt` first, then `cargo clippy --locked --all-targets -- -D warnings`
- Check for unused imports, variables, and dead code

### Windows Application Control blocks target binaries

Observed on a managed Windows host (2026-10-26): Windows Application Control blocks specific **file hashes and paths** under `target`, not all execution. A fresh `hello.exe` and a fresh proc-macro DLL ran, while `maud_macros` and `tokio_macros` DLLs from the Cargo cache and several test harnesses were denied. The error is `os error 4551` ("Application Control policy has blocked this file"). `rustc` can then report a misleading secondary error such as `can't find crate for axum` because the proc-macro failed to load.

The repository carries a narrow, committed [`.cargo/config.toml`](../../.cargo/config.toml) profile override for the packages observed to be blocked (`maud_macros`, `tokio-macros`, `tokio` and `ekp_platform`). It changes codegen settings only; it does not disable debug assertions, edit the lockfile, vendor different source or bypass the host policy. It allowed the canonical commands to pass from `7f6ab97` through `b2bae3e`, but the block returned at `23b8996`: a freshly rebuilt `platform\target\debug\ekp_platform.exe` was denied (`os error 4551`), so `scripts/check.ps1` failed at the platform test even though the source compiled. Do not assume the canonical command will run on this host; verify the exact revision and record the actual result.

At `23b8996` the release profile was executable and passed the same behavior: `cargo test --release --locked` (platform 270, eval 12), `--selftest` (49 assertions), schema (20 records) and the 28-case search evaluation all exited 0. This is equivalent coverage, **not** a canonical `scripts/check.ps1` PASS.

If the denial occurs:

1. Confirm the error is `os error 4551`, not a Rust compile error.
2. Remove the stale `target` directory and retry the canonical command so Cargo rebuilds with the committed profile. If the denial persists, record the check as `Blocked`; do not lower the gate.
3. Do not copy a blocked test binary out of `target`, rename it, point tests at another build, or relocate the build directory to evade the policy. The test harness uses only the Cargo-provided binary path (`CARGO_BIN_EXE_ekp_platform`); the earlier `EKP_PLATFORM_BIN` diagnostic override was removed on 2026-10-29 so it cannot be used as a workaround.
4. If a harness still cannot run, record the exact path, policy ID and result as `Blocked`. Ask the endpoint-security owner for a scoped exception or use an approved CI/clean runner; do not change product behavior or test identity to make a harness pass.

Rules:

- Never disable or bypass the host security policy, and never copy a blocked dependency or test binary over a permitted one, rename it, or move the build output to disguise its identity or evade the policy.
- The committed profile is part of the build, not a private workaround. A check is `PASS` only when the exact canonical command exits 0 on the named revision. If the host blocks it even after a clean rebuild, record `Blocked`, not `PASS`.
- Prefer CI for release evidence when a remote exists. See the [quality status](../quality/README.md) and [backup log](../operations/backup-log.md) for the current local evidence.

### Selftest fails
- Check that data files exist in `platform/data/`
- Verify JSON syntax in data files
- Check that all rule IDs in cases exist in rules.json
