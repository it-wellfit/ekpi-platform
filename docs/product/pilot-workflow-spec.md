# Pilot workflow specification (P-PILOT) — proposal, not started

- **Status:** `Planned` — no code, data or validation exists for this workflow yet
- **Owner:** unassigned (`DEC-01` open) — a named pilot owner is a precondition, not a formality
- **Blocked by:** `DEC-01` (pilot workflow owner) in the [decision register](../governance/08-decision-register.md)
- **Gate impact:** [Phase A row A1](../quality/phase-gates.md) stays `Not met` until a representative user completes this workflow on authorized non-sensitive data and the session is recorded

## Why this document exists

An earlier revision shipped a "pilot workflow report" that described reviews, validation and
performance numbers that never happened. That content was removed. This document replaces it
with the honest artifact: a specification of what the pilot will run, what evidence it must
produce, and what it may not claim. Nothing in this document is evidence of completion.

## Scope (proposed)

One complete document lifecycle executed by a representative user through the local governance
workbench, with every state transition recorded in the append-only ledger:

1. `document_created` — a project and its first evidence record are created
2. `document_reviewed` — a second, different user reviews the evidence (reviewer recorded
   from the session, never from a form field)
3. `document_approved` — an approver role approves; the decision links to the evidence version
4. `document_archived` — the workflow version is frozen; the ledger replay confirms the chain

## Process model (to be added to `platform/data/processes.json` only with rules, docs and cases)

- Activities: `document_created` → `document_reviewed` → `document_approved` → `document_archived`
- Precedence only (no parallel pairs — a parallel pair that is also a precedence pair is a
  contradiction; the earlier draft of this workflow contained exactly that defect)
- No forbidden pairs; not repeatable

## Acceptance criteria (measurable, all must hold)

| ID | Criterion | Evidence |
|---|---|---|
| PW-1 | The four activities execute in order via the web workbench by two distinct named users | Ledger event sequence with session-derived actors |
| PW-2 | `events_sha256` over the recorded sequence is stable across two replays | `platform.decide` / replay output, recorded twice |
| PW-3 | Every mutating step derives its actor from the authenticated session (form-supplied actor fields are ignored) | `ledger_actor_is_the_authenticated_session_not_the_form_value` test + manual run log |
| PW-4 | The pilot session record (who, when, data scope, outcome) is written to the traceability register | Register row marked `Validated` |
| PW-5 | A fresh-clone restore of the post-pilot revision replays the same ledger state | Backup drill row in `docs/operations/backup-log.md` |

## Explicit non-claims

- This specification is not evidence of a working pilot.
- No performance, accuracy or adoption number may be published for the pilot until PW-1…PW-5
  hold with linked, dated evidence.
- The local session auth used by the workbench is not enterprise identity; the pilot does not
  satisfy `DEC-12` and must not be described as SSO/MFA-capable.

## Next actions

1. Resolve `DEC-01`: name the pilot owner and the authorized non-sensitive dataset.
2. Land the process model via TDD (schema + process verification tests first, then data).
3. Run the pilot session; record evidence per PW-1…PW-5.
