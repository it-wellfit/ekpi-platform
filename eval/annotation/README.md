# Inter-annotator agreement study (R2-7 / M-11)

- **Status:** NOT STARTED. 0 of 2 annotators assigned; Cohen's κ is **not measured**.
- **Corpus:** the fixed 67 synthetic verdict cases declared by `../cases/corpus.json`. A study must cover every corpus case id exactly once; a subset or a foreign case id is rejected.
- **Proposed acceptance threshold:** κ ≥ 0.80 (`KAPPA_ACCEPTANCE` in `../src/annotation.rs`). Product-owner sign-off is still required before it becomes a release gate.
- **Non-claims:** this directory contains no label evidence. The blank sheet is not an annotation. A single author's labels, an AI pass, or two passes by the same person do not satisfy M-11.

## Files

| File | Role |
| --- | --- |
| `blinded-cases.jsonl` | Generated from the fixed corpus. Contains only case facts: `case_id`, `task`, `doc_versions`, `events`, `rules`, `loop`. It intentionally excludes `expected_verdict`, `notes`, `citations`, `category` and `abstain_required`. 67 rows. |
| `annotation.schema.json` | Record schema for a completed annotator file. |
| `README.md` | This protocol and rubric. |
| `annotator-a.jsonl`, `annotator-b.jsonl` | To be created by the two annotators. They must not exist until the study actually runs. |

## Independence rules

1. The two annotators must be different people. Neither may be the case author or the person who wrote the expected labels.
2. Neither annotator may see `../cases/seed_cases.jsonl`, `../cases/casebook_v3.jsonl`, `../cases/edge_cases.jsonl`, any `expected_verdict`, any case `notes`, or the other annotator's file before both files are frozen.
3. No discussion of individual cases until both files are complete.
4. Each annotator labels all 67 cases in one independent pass.
5. Pseudonymous ids are acceptable. Record the mapping separately if the organization requires attribution.

## Annotation inputs

Give each annotator only:

- this README and `annotation.schema.json`;
- `blinded-cases.jsonl`;
- the rule library `../../platform/data/rules.json`;
- the document text `../../platform/data/docs.json`;
- the engine semantics `../../design/engine-spec-v0.1.2.md`.

If a case refers to a rule or document that cannot be resolved from those files, label it `insufficient_evidence` and write the missing reference in `notes`. Do not infer a rule from the case id or category.

## Rubric

| Label | Use when | Do not use when |
| --- | --- | --- |
| `comply` | The observed events satisfy every applicable rule, including order, actor, evidence, deadline and loop constraints. | Any required evidence is absent, unreadable or bound to the wrong version. |
| `violate` | The evidence establishes a definite rule or process violation (wrong order, forbidden transition, duplicate non-repeatable activity, deadline breach that blocks, missing mandatory signature that is proven absent). | The evidence is merely ambiguous or a record is missing; absence of a record is not proof that the event did not happen. |
| `escalate` | The evidence establishes a conflict, authority/role mismatch, or loop-cap condition that requires a human decision; the engine must not pick a side. | The case is simply under-specified; use `insufficient_evidence` instead. |
| `insufficient_evidence` | The evidence needed to decide is missing, unresolvable, not yet effective, expired, draft-only, or bound to a different document version. | A definite violation is already established by the available evidence. |

`escalate` may substitute for `insufficient_evidence` in the evaluation metrics, but annotators must choose the label that best describes the case.

## Procedure

1. The study owner assigns two independent annotators and records the assignment date.
2. Regenerate the blinded sheet from the current corpus:

   ```powershell
   cd eval
   cargo run --locked -- --make-annotation-sheet annotation/blinded-cases.jsonl
   ```

3. Each annotator writes one JSON object per line to their own file:

   ```json
   {"case_id":"ORD-001","verdict":"violate","annotator":"annotator-a","notes":"execute before approve"}
   ```

4. Freeze both files and record their SHA-256 hashes.
5. Compute agreement:

   ```powershell
   cargo run --locked -- --kappa annotation/annotator-a.jsonl annotation/annotator-b.jsonl --corpus cases/corpus.json
   ```

   `--kappa` loads the fixed corpus even when `--corpus` is omitted. Pass it explicitly when freezing the study so the corpus is named in the command. The output records `corpus_id`, `corpus_cases` and `n`; all three must agree with the frozen corpus.

   Exit codes: `0` = threshold met, `1` = below threshold, `2` = invalid input, not run, or a case set that does not exactly cover the fixed corpus. A subset, an extra case or an invented case id is an error, not a partial study.
6. Record the command output verbatim in the evidence register. Do not edit the output.
7. If κ < 0.80, adjudicate every disagreement with a third party, record each adjudication, revise the rubric if needed, and run a new blinded round. Do not relabel cases merely to reach the threshold.
8. If κ ≥ 0.80, record the result. The corpus labels remain unchanged unless an adjudication is approved through change control.

## Current state

| Item | Value |
| --- | --- |
| Annotators assigned | 0 of 2 |
| Completed annotation files | 0 |
| κ | **NOT MEASURED** |
| Tooling state | Exact-corpus κ validator, strict annotation schema and 67-row blinded sheet are implemented and tested |
| CI execution of this study | Not applicable; the study is manual |

Do not describe a synthetic smoke test, a self-review, or an AI-generated label set as an inter-annotator study.
