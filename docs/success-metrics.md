# Quantifiable Success Metrics

- **Status:** Historical summary. **Not the current target register.**
- **Last reviewed:** 2026-10-26
- **Canonical register:** [metrics/targets.md](metrics/targets.md)
- **Phase gates:** [quality/phase-gates.md](quality/phase-gates.md)

> Targets in this file differ from the locked/proposed register and were never all approved or measured. Where they conflict, [metrics/targets.md](metrics/targets.md) wins. Do not cite these numbers as current acceptance criteria or as achieved results.

## Accuracy Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Verdict accuracy | ≥ 0.95 | Correct verdicts / total cases |
| Citation accuracy | 100% | Resolvable citations / total citations |
| Evidence completeness | ≥ 95% | Evidence-covered conclusions / total conclusions |
| Abstention accuracy | 100% | Correct abstentions / total abstention-expected cases |
| Causal misattribution rate | 0% | Causal claims misclassified as normative/process |

## Performance Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| P95 latency per case | < 500ms | 95th percentile of adjudication time |
| P99 latency per case | < 1s | 99th percentile of adjudication time |
| Throughput | ≥ 100 cases/min | Cases processed per minute |

## Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Manual handoff rate | < 20% | Cases requiring human review / total cases |
| Error rate (wrong_allow) | < 1% | Violations judged as comply / total violations |
| False negative rate | < 1% | Missed violations / total violations |
| False positive rate | < 5% | False violations / total compliant cases |
| Version error rate | 0% | Wrong version used / total cases |

## Cost Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cost per case | < $0.01 | LLM + retrieval + storage cost per case |
| Cost per 1000 cases | < $10 | Total cost for 1000 cases |

## Completion Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Completion rate | 100% | Cases with a verdict (including abstain/escalate) / total cases |
| Test pass rate | 100% | Passing tests / total tests |
| Clippy warning count | 0 | Number of clippy warnings |
| Selftest assertion pass rate | 100% | Passing assertions / total assertions |
