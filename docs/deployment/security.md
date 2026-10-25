# Security Hardening

## Controls

- `#![forbid(unsafe_code)]` in all Rust code
- Clippy with `-D warnings` (zero warnings policy)
- WORM evidence storage with SHA-256 hash chain
- Tenant isolation via row-level filtering
- Prompt injection defense (SEC-001 test case)
- Agent operation whitelist
- Human approval for high-risk operations
- Audit event log (append-only, SHA-256 chain)

## Secret Handling

- No secrets in source code
- Environment variables for credentials
- .gitignore includes *.env, *.key, *.pem
