# Monitoring

## Metrics

- P95 latency per case adjudication
- Verdict accuracy on seed cases
- Evidence completeness rate
- Citation accuracy rate
- Manual handoff rate
- Cost per case (LLM + retrieval + storage)
- Error rate (wrong_allow)

## Logging

- Rust tracing with RUST_LOG env var
- Structured JSON logs (R4)
- Audit event log for all high-risk operations
