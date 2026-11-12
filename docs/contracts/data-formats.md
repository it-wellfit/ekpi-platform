# Data and export formats

- **Status:** Active
- **Owner:** Engineering
- **Last reviewed:** 2026-11-12
- **Contract IDs:** `CTR-DATA-001`–`CTR-DATA-009`, `CTR-EXP-001`–`CTR-EXP-005`

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

## Evidence pack export

`CTR-EXP-003`. `--export-evidence-pack DIR [--governance-data PATH] [--approval-data PATH]` writes `DIR/evidence_pack.json` — a deterministic digest of the two runtime hash chains plus platform corpus counts. Standalone data-layer mode: it cannot be combined with `--out`/`--eval`/`--search`/`--import-wellfit`/`--audit-log`/`--export-legalruleml`/`--coverage-report`/`--html`/`--cases` (usage error, exit 2), and the two path overrides require this flag.

Contract:

- Top level: `schema` (`"ekpi-evidence-pack-v1"`), `governance`, `approvals`, `platform`.
- `governance`: `event_count`, `chain_verified` (bool), `head_sha256` (string|null), `payload_counts` (label → count, BTree order); on load failure additionally `load_ok: false` with zeroed counts.
- `approvals`: `event_count`, `chain_verified`, `head_sha256`, `payload_counts` (serde snake_case tags); same `load_ok` rule.
- `platform`: `docs`, `active_rules`, `processes`, `load_ok`.
- **No timestamps and no paths** — the pack must be byte-identical for the same inputs (locked by `evidence_pack_export_is_deterministic_and_records_chain_state`).
- Exit codes: `0` when both chains load and verify; `1` when a chain fails (the pack is still written with `chain_verified: false` so the failure is auditable); `2` for usage errors.
- This pack summarises chain state; it is not a substitute for the runtime-data restore drill (`IMP-021`) or off-host backup (Phase B).

## Data map export

`CTR-DATA-008`. `--data-map DIR` writes `DIR/data_map.json` — the field→event→write-surface→page lineage table, generated from **`'static` constant tables** in `platform/src/data_map.rs` (type-system-driven, not text scanning).

Contract:

- Top level: `schema` (`"ekpi-data-map-v1"`), `governance` (9 entries — one per `platform::governance::EventPayload` variant), `approvals` (5 entries — one per `ekp_domain::workflow::ApprovalEventPayload` variant).
- Each entry: `event` (snake_case tag, matches the chain JSON event type), `fields` (domain struct field names), `write_surfaces` (form/API routes that can produce the event), `pages` (workbench pages that render it).
- **No timestamps and no paths.** The table is a compile-time constant: byte-identical across runs (locked by `data_map_export_is_deterministic`).
- **Coverage lock:** `governance_lineage_for`/`approval_lineage_for` are wildcard-free exhaustive matches — adding an event variant breaks the compile until the lineage table is updated. Table-size drift is locked by `lineage_table_sizes_match_the_event_enums` (9/5).
- Note: `platform::governance::EventPayload` (the ledger's live type) and `ekp_domain::governance::EventPayload` are two independent enums today; the data map targets the live ledger type. Consolidation is tracked as refactor candidate C6 (see `docs/development/refactor-criteria.md`).
- Exit codes: `0` on success; `2` for usage errors (conflicting modes).

## Compliance report export

`CTR-EXP-004`. `--export-compliance-report DIR [--governance-data PATH] [--approval-data PATH]` writes `DIR/compliance_report.json` (`ekpi-compliance-report-v1`) plus `DIR/compliance_report.md` (rendered from the same JSON — single source).

Contract:

- `governance`: `projects`; `evidence` (`total`, `reviewed`, `unreviewed_ids` sorted); `quality_cases` (`total`, `by_status` BTree order, `non_closed_ids` sorted — "non-closed" means `status != "closed"`); `inspections`; `gate_decisions` (`total`, `by_decision`); `chain_verified`; `event_count`; `head_sha256`. On load failure: zeroed counts plus `load_ok: false`.
- `approvals`: `workflows_total`; `tasks` (`total`, `by_state`, `per_task` sorted by id — each with `id`/`workflow`/`stage_id`/`state`/`due_at`); `chain_verified`; `event_count`; `head_sha256`. Same `load_ok` rule.
- `platform`: `docs`, `active_rules`, `processes`, `load_ok`.
- **No timestamps, no paths, no time-relative computations** (overdue is deliberately NOT computed — timeliness is the consumer's job against `due_at`). Byte-identical replays locked by `compliance_report_export_is_deterministic`.
- The `.md` twin renders the same JSON with a mandatory disclaimer (狀態事實，非合規認證). It adds no facts.
- Exit codes: `0` when both chains load and verify; `1` when a chain fails (report still written, `chain_verified: false`); `2` usage errors. Conflicts mirror CTR-EXP-003 plus this flag; the two path overrides are shared with CTR-EXP-003.

## Audit package manifest

`CTR-EXP-005`. `scripts/audit-package.ps1` produces an auditable package: git bundle (full history), runtime chain copies (governance/approvals), `evidence_pack.json` (CTR-EXP-003), `data_map.json` (CTR-DATA-008), and `manifest.json` (`ekpi-audit-manifest-v1`: `head_commit` + sorted `files[{name, sha256, bytes}]`). The manifest content is deterministic given the same packaged bytes (no timestamps inside the file; the output directory name carries the wall-clock stamp). Self-verification re-hashes every manifest entry. The operational restore drill (`scripts/drill-runtime-data.ps1`, IMP-021) is packaged alongside the evidence.

## Process conformance report (library)

`CTR-DATA-009`. `platform::conformance` computes fitness and precision for one process model and one case event log. It is a **library-only** contract: no CLI flag or report field consumes it yet; wiring it to a served surface is a later slice.

- `Metric` is an exact rational (`u128` numerator/denominator, gcd-reduced) — no floating point anywhere. Fractions compare structurally (`3/3 == 1/1`); denominators are never 0.
- `fitness`: `1 − (missing + remaining) / total`, where `missing` counts log tokens whose required predecessor never occurred, `remaining` counts model activities never observed, and `total` is observed model-activity events plus `remaining`. `None` only when the model is empty and the log has no model activities (nothing measurable — never reported as 0 or 1).
- `precision`: `1 − escaping / observed_pairs` over adjacent event pairs whose ends are both model activities. Allowed edges: directional `precedence`, **symmetric** `parallel_pairs` (both orders), and self-loops only when `repeatable`, or when the activity is in `repeat_parties` and the two observations come from **different actors** (multi-party double-sign). Adjacency is never fabricated across unknown events — pairs touching an unknown activity are skipped, and `Unconfirmed` remains the replay layer's concern. `None` when no pair is evaluable.
- `ConformanceReport` keeps every raw count (`missing_tokens`, `remaining_tokens`, `observed_pairs`, `escaping_pairs`, `log_events`, `model_activities`) so every ratio is derivable from named inputs.
- Determinism: same model + same events → identical report (BTreeMap iteration; pure function).

Verification: `cargo test --locked -p ekp_platform conformance` — 12 unit tests including exact-rational equality, empty-input non-measurement, parallel symmetry and multi-actor self-loops.

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
