# Troubleshooting

## Common Issues

### Build fails with "could not find Cargo.toml"
- Ensure you're in the `platform/` or `eval/` directory, not the root
- There is no root workspace; each crate is independent

### Clippy fails with warnings
- Run `cargo fmt` first, then `cargo clippy --locked --all-targets -- -D warnings`
- Check for unused imports, variables, and dead code

### Tests fail with "os error 4551"
- Windows application control policy blocks binary execution
- Run tests with `cargo test --locked` which uses the test harness
- Or disable Windows Defender Application Control for the target directory

### Selftest fails
- Check that data files exist in `platform/data/`
- Verify JSON syntax in data files
- Check that all rule IDs in cases exist in rules.json
