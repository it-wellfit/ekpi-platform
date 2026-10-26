# Evaluation policy and fixed corpus

- **Status:** Active for the deterministic core; AI/agent evaluation is **not implemented**
- **Owner:** QA + AI system owner (role not yet assigned)
- **Last reviewed:** 2026-10-26
- **Target register:** [metrics/targets.md](../metrics/targets.md)
- **Hard gates:** H-1–H-6 in the target register

## Scope

This policy covers two different things:

1. **Deterministic evaluation (current):** the engine's verdicts, citations, abstention, process replay and hard gates on a fixed JSONL corpus, plus a fixed lexical-retrieval regression set for `--search`. No model, prompt, external retrieval service or tool is involved.
2. **AI/agent evaluation (target):** any future LLM, prompt, hosted retrieval, connector or tool path. None exists in the served code today; `agents` and `pm4py_sidecar` are library-only and not called by the CLI or web app.

Do not use deterministic-corpus results to claim AI quality, and do not use an AI evaluation result to claim the deterministic engine is correct. They are separate evidence streams.

## Fixed corpus

| File | Cases | Role |
| --- | --- | --- |
| `eval/cases/seed_cases.jsonl` | 9 | Primary self-test and default evaluation set |
| `eval/cases/casebook_v3.jsonl` | 46 | Expanded adjudication corpus |
| `eval/cases/edge_cases.jsonl` | 12 | Boundary, refusal and adversarial cases |
| **Total** | **67** | Fixed corpus (≥50 target M-12) |
| `platform/data/eval/search_eval.json` | 24 | Fixed retrieval regression set for `--search`/`--eval` (18 answer, 6 refusal) |

Observed verdict distribution on 2026-10-26: `comply` 28, `violate` 11, `escalate` 10, `insufficient_evidence` 18. The search set has its own threshold (0.42) and categories; it is not part of the 67-case verdict corpus and must not be counted as one.

The corpus is versioned in Git. Case IDs are stable; expected verdicts may change only with an approved decision and an updated regression test. Never delete a failing case to make a run pass.

## Required evaluation dimensions

The six dimensions below are mandatory for an AI product. For the deterministic core, five are represented; tool failure is a gap because no tool is called.

| Dimension | What it tests | Representative corpus categories | Coverage |
| --- | --- | --- | --- |
| Normal | Correct verdict and citations on clear cases | compliant and violation change cases, approval signatures, close-gate cases; search `general` | Covered |
| Ambiguous | Conflicting sources, partial evidence, unclear ownership or timing | `conflict_no_override`, `conflict_with_override`, `contradiction`, `partial_pass_note`, `version_conflict`, `unknown_activity_noted`; search `ambiguous` | Covered |
| Adversarial | Prompt injection, embedded directives, rule poisoning, attempts to bypass a gate | `prompt_injection`, `injection_resisted`, `injection_and_rule_expiry`, `draft_rule_invisible`; search `adversarial` | Covered |
| Wrong source | Stale version, wrong appendix, missing cross-document evidence, mismatched pin | `stale_pool_pin_mismatch`, `missing_cross_doc_evidence`, `appendix_binding_mismatch`, `future_rule`, `rule_expired`; search `wrong_source` | Covered |
| Tool failure | Timeout, unavailable connector, partial tool result, retry exhaustion, sidecar failure | None; the retrieval CLI's missing-data path is covered by a fail-closed test, which is not a tool-failure case | **Gap** (`EVAL-GAP-01`) |
| Insufficient context | Missing events, unregistered rule, no evidence, deadline anchor missing | `missing_evidence`, `unregistered_rule_id`, `deadline_anchor_missing`, `actor_role_mismatch`, `loop_state_missing`; search `insufficient_context` | Covered |

The category names are the fixture's test categories, not a claim that every real-world variant is covered. The corpus is synthetic.

## Metrics and hard gates

The binding definitions are in [metrics/targets.md](../metrics/targets.md). Key rules:

- Every definitive conclusion must carry at least one resolvable anchor (H-4/M-10).
- An actual violation must never be judged `comply` (H-2/M-2 = 0).
- Evidence-insufficient cases must not be judged `comply` (H-3).
- A prompt injection must not change the verdict (H-5).
- The same snapshot must replay identically (H-6).
- A forbidden process transition must never be released (H-1).

Do not copy thresholds into this document; the register is authoritative. `proposed` metrics need product-owner sign-off and measurement evidence before they block a release.

## Rerun triggers

Rerun the applicable evaluation when any of the following changes:

- model, provider, model version or inference parameters;
- prompt, system instruction, tool description or agent policy;
- retrieval index, chunking, embeddings, reranker or threshold;
- source data, rule corpus, process model or case corpus;
- connector/tool behavior, timeout, retry or failure handling;
- code that changes verdict, citation, abstention or ordering logic;
- a release, a pilot expansion or an incident fix.

For a pure documentation change, a full evaluation is not required, but the baseline check still runs. For a change that cannot affect behavior, say so explicitly rather than claiming a rerun you did not perform.

## Evaluation procedure

From the repository root:

```powershell
cd eval
cargo test --locked
cargo run --locked -- --selftest
cargo run --locked -- --cases cases/casebook_v3.jsonl --out target/eval-casebook

cd ..\platform
cargo test --locked
cargo run --locked --bin ekp_platform -- --selftest
cargo run --locked --bin ekp_platform -- --declared-rules-schema
cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
cargo run --locked --bin ekp_platform -- --params config/engine-params.json `
  --out target/regression-report --coverage-report --html
```

The retrieval gate is deterministic and offline. It runs the same lexical retriever as `--search` against 24 fixed cases and fails on any miss, unexpected citation, missing required category, or a permission-isolation score below 1.00. A green run is regression evidence for that retriever, not an AI quality metric. The permission metric is currently vacuous: no case declares `roles` and every fixture document has `permissions: []`, so no restricted document exists to leak (`EVAL-GAP-06`). Real filter behavior is covered only by the library unit tests `search_respects_permissions` and `eval_permission_isolation`.

For a tool/AI path, add a comparison run for each model/prompt/tool version, record the exact version identifiers and the corpus hash, and compare against the approved baseline. A missing tool must produce an honest failure/abstention, not a fabricated success.

## Change control

1. Add or update the case, expected verdict and rationale in the same change as the behavior/spec change.
2. Link the requirement/decision ID and the affected metric.
3. Run the focused case, the full corpus and the hard gates.
4. Record the result in the traceability register and the release evidence.
5. If an expected verdict changes, require an approved decision/ADR; do not silently relabel a failure as a pass.
6. Keep the corpus reproducible: no hidden edits, no external service required for the deterministic set.

## Annotation and agreement

`M-11` (Cohen's κ ≥ 0.80 proposed) requires two independent annotators on the expected verdicts. This has **not** been measured. The `eval/src/annotation.rs` module computes κ, but no double-annotation study exists. A single author's labels are not inter-annotator agreement.

## Known gaps

| ID | Gap | Impact | Next action |
| --- | --- | --- | --- |
| EVAL-GAP-01 | No tool-failure cases | A future connector/LLM/tool path has no regression evidence for timeout/partial failure/retry exhaustion. The current retrieval CLI calls no tool; its missing-data test is fail-closed evidence only | Add cases and a failure-injection harness when a tool is introduced |
| EVAL-GAP-02 | No LLM/agent implementation or evaluation | AI quality, hallucination, tool success and cost/latency are unmeasured | `DEC-15`; implement only after provider/data policy approval |
| EVAL-GAP-03 | No double annotation / κ | Expected labels are not independently validated | Run a two-annotator study for the expanded corpus |
| EVAL-GAP-04 | Synthetic corpus only | No evidence that the cases represent a real organization | Pilot with authorized non-sensitive cases; keep them separate from the fixed synthetic set |
| EVAL-GAP-05 | No automated drift/version comparison report | Model/prompt/data changes could regress silently | Add a versioned baseline and comparison report before AI use |
| EVAL-GAP-06 | Fixed search set has no role-restricted case | The `permission isolation 1.00` gate is 1.00 by construction (no case declares `roles`, all fixture documents are unrestricted), so it cannot detect a filter regression at the set level. Library unit tests cover the filter but are not an end-to-end permission test | Add at least one restricted fixture and a positive/negative case pair with a non-empty `roles` allow-list; keep the metric meaningful or remove it from the gate |

## Current status

- Deterministic corpus: 67 cases, fixed and versioned.
- Search retrieval regression: 24 cases (18 answer, 6 refusal), threshold 0.42; `--eval` gate PASS on the recorded revision. Lexical only; paraphrases can abstain.
- Permission coverage in the fixed set: none. All cases omit `roles` and all fixture documents are unrestricted, so `permission isolation 1.00` is vacuous (`EVAL-GAP-06`). The passing result is a no-leak regression check, not permission coverage.
- Hard gates H-1–H-6: exercised by the engine self-test/report pipeline; see the dated regression baseline.
- AI/agent: not implemented; no quality claim.
- Tool failure: not covered for a future connector/LLM/tool path (`EVAL-GAP-01`). The retrieval CLI calls no tool; its missing-data fail-closed test is not a substitute.
- Annotation agreement: not measured.

## Update triggers

Update this document when the corpus, dimensions, metrics, rerun triggers, annotation policy, tool/AI scope or known gaps change. Update the [traceability register](../traceability/register.md) and the release evidence in the same change.
