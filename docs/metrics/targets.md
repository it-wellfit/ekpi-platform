# EKPI Metrics Targets & Success Specification

> Repository-resident, measurable target register. Every row defines a metric,
> its measurement method, a proposed or locked target, owner, and review date.
> Numeric targets marked `proposed` require product-owner sign-off before they
> block a release. Locked targets are hard gates derived from Addendum A1/A2/A5
> and the tech-selection acceptance criteria (H-1..H-6).

## Status conventions

| Status | Meaning |
|---|---|
| `locked` | Hard gate; must be met for any release. |
| `proposed` | Working target; needs product-owner sign-off to become locked. |
| `deferred` | Intentionally not measured in this iteration. |

## Target register

| ID | Metric | Definition | Measurement method | Target | Status | Owner | Review date |
|---|---|---|---|---|---|---|---|
| M-1 | Verdict accuracy (deterministic cases) | cases where engine verdict == expected verdict / total deterministic cases | `ekp_eval` summary `accuracy_nonabstain` over `seed_cases.jsonl` + `casebook_v3.jsonl` | 100% | locked | Product | 2026-11 |
| M-2 | Wrongful-allow rate | actual violations judged `comply` | `ekp_eval` summary `violation_as_comply` (hard gate H-2) | 0 | locked | Product | 2026-11 |
| M-3 | Correct refusal rate | insufficient-evidence cases correctly abstained | `ekp_eval` summary `correct_refusal` over abstain-required cases | confirm | proposed | Product | 2026-11 |
| M-4 | Unwarranted refusal rate | definitive cases wrongly abstained (`insufficient_evidence` on non-abstain case) | `ekp_eval` summary `refusal_on_definitive` | confirm | proposed | Product | 2026-11 |
| M-5 | Processing time per case | wall-clock ms for `decide` end-to-end | `Decision.latency_ms`; report `mean_latency_ms`, p50/p99 | confirm | proposed | Eng | 2026-11 |
| M-6 | Manual handoff rate | cases requiring human resolution / total cases | escalate verdicts + unwarranted refusals / total | confirm (<10%) | proposed | Product | 2026-11 |
| M-7 | Error rate | failed case loads / total loads | `ekp_eval` loader warnings + `collect` errors / total cases | confirm (<1‰) | proposed | Eng | 2026-11 |
| M-8 | Cost per case | token/compute cost estimate per adjudication | `Decision.cost_tokens`; report `mean_cost_tokens` | confirm | proposed | Product | 2026-11 |
| M-9 | Completion rate | cases reaching a terminal verdict / escalate / abstain | cases with non-missing decision / total cases | 100% | locked | Product | 2026-11 |
| M-10 | Citation recall (anchor resolvability) | expected citation ∩ cited citation / expected citation | `ekp_eval` summary `citation_recall`; H-4 = every definitive conclusion ≥1 resolvable anchor | confirm | proposed | Product | 2026-11 |
| M-11 | Inter-annotator agreement (κ) | Cohen's κ between two independent annotators on expected verdicts | double-annotation of expanded case set | confirm (≥0.80) | proposed | QA | 2026-12 |
| M-12 | Case count | total scored cases in eval corpus | count of `eval/cases/*.jsonl` records | ≥50 | proposed | QA | 2026-11 |

## Locked hard gates (never relaxed)

| Gate | Source | Requirement |
|---|---|---|
| H-1 | Forbidden state-transition release rate = 0 | ORD-001-type cases observed→violate, never released |
| H-2 | Leakage rate (actual violation judged comply) = 0 | M-2 locked at 0 |
| H-3 | Evidence-insufficient judged comply = 0 | abstain on missing evidence |
| H-4 | Citation completeness | every definitive conclusion ≥1 resolvable anchor |
| H-5 | Injection does not change verdict | SEC-001-type cases ignore embedded directives |
| H-6 | Replay consistency | same snapshot replays identically across runs |

## Derivation rule

Every public number in `report.json` / `report.md` / `engine_hard_gates.json` must be derivable from a named, documented source: `engine_snapshots.json`, `engine_hard_gates.json`, or persisted `validation_findings`/case row counts. In-memory counts must agree with persisted counts. No number is invented.

## Review cadence

Review this register each release. Move a `proposed` row to `locked` only with product-owner sign-off and supporting measurement evidence attached to the release PR.