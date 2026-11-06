# Data and export formats

- **Status:** Active
- **Owner:** Engineering
- **Last reviewed:** 2026-10-26
- **Contract IDs:** `CTR-DATA-001`–`CTR-DATA-007`, `CTR-EXP-001`

These are the current local formats. They are file contracts, not a production API. All files are UTF-8. Counts and byte sizes below are observations on 2026-10-26 and must be rechecked after data changes.

## Documents

`platform/data/docs.json` (`CTR-DATA-001`)

```json
{
  "documents": [
    {
      "doc_id": "SOP-CHANGE",
      "title": "...",
      "permissions": [],
      "versions": [
        {
          "version": "3.2",
          "effective_from": "2026-03-01",
          "effective_to": null,
          "injection_directive": null,
          "sections": [
            { "section": "4.3", "page": 12, "verbatim_text": "..." }
          ]
        }
      ]
    }
  ]
}
```

Rules:

- `doc_id` + `version` + `section` identify a clause anchor.
- `effective_from`/`effective_to` bound the valid-time window. A version outside its window is not applicable.
- `injection_directive` is a synthetic adversarial field used to prove that embedded instructions do not become system rules. It is never an instruction to the operator.
- `permissions` is an optional role-name list. An empty list means public to all roles. It is a **library-level filter**, not an authenticated authorization decision: the CLI `--search` mode has no user identity and applies no permission filter, and `governance_web` does not use it. `PermissionContext` is exercised by `retrieval` unit tests and by the synthetic role cases in `--eval`.
- The production fixture contains 7 documents and 10 versions. Evaluation-only restricted documents live in the fixed set's `fixtures` array, not in this file. Do not add real customer content.

## Search evaluation set

`platform/data/eval/search_eval.json` (`CTR-DATA-007`).

```json
{
  "version": 1,
  "threshold": 0.42,
  "fixtures": [
    {
      "doc_id": "EVAL-RESTRICTED-QM",
      "title": "Evaluation fixture: quality-manager restricted policy",
      "permissions": ["quality_manager"],
      "versions": [
        {
          "version": "1.0",
          "effective_from": "2026-01-01",
          "sections": [
            {"section": "1.1", "verbatim_text": "..."}
          ]
        }
      ]
    }
  ],
  "cases": [
    {
      "id": "GEN-01",
      "category": "general",
      "question": "...",
      "expected_doc_id": "SOP-CHANGE",
      "expected_section": "4.3",
      "min_score": 0.42,
      "roles": []
    },
    {
      "id": "PERM-01",
      "category": "permission",
      "question": "...",
      "expected_doc_id": "EVAL-RESTRICTED-QM",
      "expected_section": "1.1",
      "min_score": 0.42,
      "roles": ["quality_manager"]
    },
    {
      "id": "IC-01",
      "category": "insufficient_context",
      "question": "...",
      "expect_refusal": true,
      "min_score": 0.0
    }
  ]
}
```

Rules:

- A bare JSON array is also accepted for backward compatibility; the object form is preferred because it carries `version` and an operating `threshold`.
- `id` is unique; required categories are `general`, `ambiguous`, `adversarial`, `wrong_source`, `insufficient_context` and `permission` (additional categories are allowed but do not satisfy the required coverage gate).
- An answer case must name `expected_doc_id` and `expected_section`. A refusal case sets `expect_refusal: true` and omits both.
- `min_score` is the minimum top-1 score for an expected hit. `threshold` and `min_score` are in `[0,1]`.
- `fixtures` is optional. Each fixture uses the `docs.json` document shape and is merged into a clone of the production database for `--eval` only. Fixture ids must not collide with production ids, every fixture must be referenced by a case's `expected_doc_id`, and fixtures are never written to `docs.json`. The ordinary `--search` path never loads them; an integration test asserts that fixture text stays invisible to `--search`.
- A case with a non-empty `roles` array exercises `PermissionContext`; an empty array means no permission filter. A role case with `expect_refusal: false` is an allowed-retrieval case; one with `expect_refusal: true` is a denial case.
- The permission isolation metric counts only role-restricted cases. The gate requires at least two role-restricted cases, at least one allowed and one denied case, and zero permission violations. Public-only cases do not make the metric pass.
- The fixed set is retrieval regression evidence only. It does not test an LLM, prompt, agent tool or production authorization. The role cases prove the library filter can allow and deny synthetic roles; they do not prove authenticated identity or tenant isolation.
- The current set has 28 cases (20 answer, 8 refusal), two evaluation-only fixtures and four role-restricted cases. `EVAL-GAP-06` is closed as fixed-set coverage; the broader authorization boundary remains a documented gap.

## Rules

`platform/data/rules.json` (`CTR-DATA-002`), validated against `design/rule-schema-v0.1.1.schema.json`.

Top-level shape:

```json
{ "rules": [ { "rule_id": "...", "rule_version": "...", ... } ] }
```

Each rule carries:

- `rule_id`, `rule_version`;
- `deontic` (modality/strength);
- `subject` (entity type, actor role, scope);
- `condition` (predicate tree);
- `sequencing` (`required_before`, `forbidden_after`, `deadline`, `on_violation`, `loop_policy`);
- `exceptions` (`exception_of`, `overridden_by`, `overrides`, `conflicts_with`);
- `temporal` (`effective_from`, `effective_to`, `supersedes`, `superseded_by`);
- `evidence_requirements` (type, minimum count, metadata constraints, missing-evidence result);
- `decision` (method, missing-evidence result, abstain threshold);
- `source` (document/version/clause/verbatim/selectors);
- `provenance` (origin, extraction, review status/reviewer/date);
- `validation` (test case refs, last run).

Observed fixture: 20 records = 19 active + 1 `draft_ai` poisoning test. The draft must never be consumable. `on_violation` semantics are `block` → `violate`, `escalate` → `escalate`, `warn` → `comply` plus a `deadline_overdue_warned` note.

## Processes

`platform/data/processes.json` (`CTR-DATA-003`)

```json
{
  "processes": [
    {
      "process_id": "P-CHANGE",
      "title": "...",
      "activities": ["..."],
      "precedence": [{ "before": "...", "after": "..." }],
      "parallel_pairs": [["...", "..."]],
      "forbidden_pairs": [["...", "..."]],
      "repeatable": false,
      "repeat_parties": ["..."]
    }
  ]
}
```

Observed fixture: 3 process models (`P-CHANGE`, `P-CAPA`, `P-PURCH`). `forbidden_pairs` is order-sensitive; the reverse ordering is a separate case. Missing required events produce abstention/escalation, not a pass.

## Engine parameters

`platform/config/engine-params.json` (`CTR-DATA-004`), `params_version` `1.0`.

```json
{
  "params_version": "1.0",
  "spec_ref": "../design/engine-spec-v0.1.2.md",
  "declared_invariants": {
    "draft_rules_consumable": false,
    "ambiguous_binding_refuses": true,
    "unregistered_rule_guessing_allowed": false,
    "equal_timestamp_counts_as_overdue": false,
    "abstain_clears_citations": true,
    "forbidden_pair_release_allowed": false,
    "injection_directive_operates_as_system_rule": false,
    "decision_path_contains_probability_component": false
  },
  "adjustable": { "retrieval_threshold": 0.70 }
}
```

The declared invariants are fail-closed: any mismatch refuses execution. Only `retrieval_threshold` is adjustable, via the file or `--threshold`; the CLI override must be in `[0,1]`.

## Report and snapshot artifacts

`CTR-DATA-005`. There is no formal JSON Schema for these artifacts yet; the de facto contract is the Rust structs in `eval/src/report.rs`, `eval/src/metrics.rs`, `platform/src/snapshot.rs` and the smoke gate.

`report.json` top-level keys observed:

- `generated_at_unix`;
- `per_system` (keyed by system name, including `G_ekpi_engine`);
- per-system `n_cases`, `n_matched`, `accuracy_nonabstain`, `wrong_allow`, `violation_as_comply`, `refusal_on_definitive`, `correct_refusal`, `escalation_correct`, `citation_recall`, `citation_precision`, `mean_confidence`, `mean_latency_ms`, `mean_cost_tokens`, `confusion_matrix`, `coverage_curve`, and per-case details.

`report.md` is the human-readable rendering. `report.html` is generated from it and must have zero `<script>` elements. `engine_snapshots.json` holds engine decisions and replay inputs. `engine_hard_gates.json` records H-1–H-6. `coverage_report.md` records rule coverage, verdict distribution, anchors and observed citations.

Derivation rule: every public number must be derivable from a named artifact or persisted case count. Do not invent a metric, denominator or timestamp. If a metric definition changes, update [metrics/targets.md](../metrics/targets.md) and the tests together.

## Governance ledger

`CTR-DATA-006`. One JSON object per line at `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` or `EKP_GOVERNANCE_DATA`.

Event fields:

| Field | Meaning |
| --- | --- |
| `sequence` | 1-based, contiguous |
| `timestamp_unix` | Seconds since Unix epoch |
| `actor` | Operator-entered name/role, 1–80 characters |
| `payload` | Typed event: `ProjectCreated`, `EvidenceLinked`, `EvidenceReviewed`, `WorkflowSaved`, `WorkflowPublished`, `QualityCaseCreated`, `QualityCaseUpdated`, `QualityInspectionRecorded`, `GateDecisionRecorded` |
| `previous_sha256` | Prior event hash, or `GENESIS` for the first event |
| `sha256` | SHA-256 of the event body including the previous hash |

The audit view verifies sequence continuity and recomputes each hash. The ledger is append-only in normal operation, but a filesystem administrator can rewrite it; it is not WORM and has no external anchor. Do not put secrets or real personal data in ledger fields.

## LegalRuleML export

`CTR-EXP-001`. `--export-legalruleml DIR` writes LegalRuleML 1.0 compact XML for active rules.

Observed artifact (2026-10-26): 51,026 bytes, root `{http://docs.oasis-open.org/legalruleml/ns/v1.0/}LegalRuleML`, 19 `PrescriptiveStatement`, 19 `PenaltyStatement`, 19 `Reparation`, 16 `LegalSource`.

Structural gate (`gate.ps1 -IncludeSmoke`):

- file size at least 10,000 bytes;
- well-formed XML;
- official root local name and namespace;
- at least 19 PrescriptiveStatement, 19 PenaltyStatement, 19 Reparation, 16 LegalSource;
- zero `<script>` in `report.html` when HTML is generated.

The 19/19/19/16 thresholds are tied to the current 19 active rules. If the active corpus changes, update the gate threshold, the exporter tests and the fixture together; do not lower the threshold to make a broken export pass.

## Decisions export (v2)

`CTR-EXP-002`. `--out DIR` additionally writes `engine_decisions.json`, the deterministic machine-readable decision export (`ekpi-decisions-v2`).

Contract:

- Top level: `schema` (`"ekpi-decisions-v2"`, required), `retrieval_threshold` (number), `counts.cases` (integer), `decisions` (array).
- One entry per seed case, **sorted by `case_id`**: `case_id` (string), `verdict` (string), `citations` (array of strings), `rule_ids_used` (array of strings), `notes` (array of strings), `manifest_sha` (string), `event_sequence_sha` (string), `replay_consistent` (boolean).
- **No timestamps.** Same corpus + parameters must replay the file byte-identically (locked by `ablation_writes_deterministic_decisions_export`); any timestamp added later would violate the contract and require a schema version bump.
- Change rules: adding an optional field is backward-compatible (bump doc + tests); removing/renaming or changing meaning requires a new schema version (`ekpi-decisions-v3`) and an ADR.

## Compatibility and change rules

- Adding an optional JSON field is backward-compatible; update the schema/doc and tests.
- Removing/renaming a field, changing a required type, or changing a metric/verdict meaning is breaking; require an accepted spec/ADR, a migration/compatibility note and updated consumers.
- A new rule schema version gets a new file, not an in-place mutation of `v0.1.1`.
- Report/snapshot format changes must preserve the derivation rule and the hard-gate evidence.
- LegalRuleML changes must preserve the official namespace and required statement structure; a private approximation is not acceptable.
- Never edit generated artifacts as source. Regenerate them from the CLI and record the command/revision.

## Verification

```powershell
cd platform
cargo run --locked --bin ekp_platform -- --declared-rules-schema
cargo run --locked --bin ekp_platform -- --selftest
cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
cargo run --locked --bin ekp_platform -- --params config/engine-params.json `
  --out target/regression-report --coverage-report `
  --export-legalruleml target/regression-lrml --html
```

Then run `gate.ps1 -IncludeSmoke` from the repository root. See [regression checklist](../development/regression-checklist.md) for the full focused scenario.

## Update triggers

Update this document when a data file field, schema version, report key, ledger event, export structure or compatibility rule changes. Update the matching contract ID in the [traceability register](../traceability/register.md) in the same change.
