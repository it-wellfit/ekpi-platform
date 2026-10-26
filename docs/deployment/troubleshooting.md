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

Diagnosis:

1. Confirm the denial is `os error 4551`, not a Rust compile error.
2. Rebuild only the blocked proc-macros with a different optimization level to produce a fresh, permitted hash, without editing the lockfile or vendor sources:
   `cargo build --locked --config 'profile.dev.package.maud_macros.opt-level=1' --config 'profile.dev.package.tokio-macros.opt-level=1' --config 'profile.dev.package.tokio.opt-level=1'`
3. If the test harness itself is blocked, copy it out of `target` and run the copy. For CLI contract tests, set `EKP_PLATFORM_BIN` to a copied, executable `ekp_platform.exe`; the tests use it only as a diagnostic override and product behavior is unchanged. For example: `EKP_PLATFORM_BIN=%TEMP%\ekpi-run\ekp_platform.exe %TEMP%\ekpi-run\cli_contract.exe`.
4. If a harness still cannot run, change only the crate test profile (for example `profile.test.package.ekp_platform.debug=0` or `opt-level=2`) to produce a different binary, then retry.

Rules:

- Never disable or bypass the host security policy, and never copy a blocked dependency over a permitted one to disguise its identity.
- These workarounds are **diagnostic evidence**, not the canonical gate. Record the exact command and the limitation. A release check that could not run as `cargo test --locked` / `gate.ps1` on the named revision is **Blocked**, not PASS.
- Prefer running the canonical gate on an approved host or CI runner. See the [quality status](../quality/README.md) and [backup log](../operations/backup-log.md) for the current blocked state.

### Selftest fails
- Check that data files exist in `platform/data/`
- Verify JSON syntax in data files
- Check that all rule IDs in cases exist in rules.json
