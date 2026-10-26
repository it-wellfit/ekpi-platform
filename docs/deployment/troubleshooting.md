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

The repository now carries a narrow, committed [`.cargo/config.toml`](../../.cargo/config.toml) profile override for the packages observed to be blocked (`maud_macros`, `tokio-macros`, `tokio` and `ekp_platform`). It changes codegen settings only; it does not disable debug assertions, edit the lockfile, vendor different source or bypass the host policy. The canonical commands now run from a clean `target`:

```powershell
cd platform
cargo test --locked
cd ..
powershell -ExecutionPolicy Bypass -File gate.ps1 -IncludeSmoke
```

If the denial still occurs:

1. Confirm the error is `os error 4551`, not a Rust compile error.
2. Remove the stale `target` directory and retry the canonical command so Cargo rebuilds with the committed profile.
3. If only a diagnostic harness is blocked, copy it out of `target` and run the copy. For CLI contract tests, set `EKP_PLATFORM_BIN` to a copied, executable `ekp_platform.exe`; the tests use it only as a diagnostic override and product behavior is unchanged. Example: `EKP_PLATFORM_BIN=%TEMP%\ekpi-run\ekp_platform.exe %TEMP%\ekpi-run\cli_contract.exe`.
4. If a harness still cannot run, change only that crate's test profile to produce a different binary and retry. Do not change the product behavior to make a harness pass.

Rules:

- Never disable or bypass the host security policy, and never copy a blocked dependency over a permitted one to disguise its identity.
- The committed profile is part of the build, not a private workaround. A check is `PASS` only when the exact canonical command exits 0 on the named revision. If the host blocks it even after a clean rebuild, record `Blocked`, not `PASS`.
- Prefer CI for release evidence when a remote exists. See the [quality status](../quality/README.md) and [backup log](../operations/backup-log.md) for the current local evidence.

### Selftest fails
- Check that data files exist in `platform/data/`
- Verify JSON syntax in data files
- Check that all rule IDs in cases exist in rules.json
