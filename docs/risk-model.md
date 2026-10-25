# Risk Model

## Threats

- T-1 Prompt injection embedded in documents (H-5): logged, never executed.
- T-2 Forbidden process transitions (H-1): hard gate, zero release.
- T-3 Unanchored conclusions (H-4): every definitive verdict cites a resolvable anchor.
- T-4 Draft rule poisoning (A1/A8): drafts never enter engine space.
- T-5 Unregistered rule ids (D-02): conservative abstention.

## Mitigations

- Version-pinned document bindings.
- Active-only rule layer.
- Conservative abstention on missing evidence.
- WORM audit trail.