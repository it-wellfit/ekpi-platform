"""Data schema types for cases and decisions."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from . import VERDICTS


@dataclass
class CaseEvent:
    activity: str
    actor: str
    ts: str


@dataclass
class TestCase:
    """One evaluation case.

    `doc_versions` binds the case to explicit document versions (addendum A7:
    test-set versioning follows document versions).
    """

    case_id: str
    category: str
    task: str
    doc_versions: dict[str, str]
    events: list[CaseEvent]
    expected_verdict: str
    abstain_required: bool
    rules: list[str] = field(default_factory=list)
    citations: list[str] = field(default_factory=list)
    loop: dict[str, Any] | None = None
    notes: str = ""

    def __post_init__(self) -> None:
        if self.expected_verdict not in VERDICTS:
            raise ValueError(
                f"case {self.case_id}: unknown expected_verdict {self.expected_verdict!r}"
            )


@dataclass
class Decision:
    """One system answer for one case."""

    case_id: str
    system: str
    verdict: str
    confidence: float
    retrieved_chunk_ids: list[str] = field(default_factory=list)
    cited_ids: list[str] = field(default_factory=list)
    rule_ids_used: list[str] = field(default_factory=list)
    evidence_refs: list[str] = field(default_factory=list)
    latency_ms: int = 0
    cost_tokens: int = 0
    notes: str = ""

    def __post_init__(self) -> None:
        if self.verdict not in VERDICTS:
            raise ValueError(
                f"decision for {self.case_id}/{self.system}: "
                f"unknown verdict {self.verdict!r}"
            )


def load_case(raw: dict[str, Any]) -> TestCase:
    missing = [k for k in ("case_id", "expected_verdict") if k not in raw]
    if missing:
        raise ValueError(f"case row missing keys: {missing}")
    events = [
        CaseEvent(
            activity=str(e.get("activity", "")),
            actor=str(e.get("actor", "")),
            ts=str(e.get("ts", "")),
        )
        for e in raw.get("events", [])
    ]
    return TestCase(
        case_id=str(raw["case_id"]),
        category=str(raw.get("category", "")),
        task=str(raw.get("task", "")),
        doc_versions={str(k): str(v) for k, v in raw.get("doc_versions", {}).items()},
        events=events,
        expected_verdict=str(raw["expected_verdict"]),
        abstain_required=bool(raw.get("abstain_required", False)),
        rules=[str(r) for r in raw.get("rules", [])],
        citations=[str(c) for c in raw.get("citations", [])],
        loop=raw.get("loop"),
        notes=str(raw.get("notes", "")),
    )


def decision_from_raw(system: str, raw: dict[str, Any]) -> Decision:
    missing = [k for k in ("case_id", "verdict") if k not in raw]
    if missing:
        raise ValueError(f"decision from {system} missing keys: {missing}")
    return Decision(
        case_id=str(raw["case_id"]),
        system=system,
        verdict=str(raw["verdict"]),
        confidence=float(raw.get("confidence", 0.0)),
        retrieved_chunk_ids=[str(x) for x in raw.get("retrieved_chunk_ids", [])],
        cited_ids=[str(x) for x in raw.get("cited_ids", [])],
        rule_ids_used=[str(x) for x in raw.get("rule_ids_used", [])],
        evidence_refs=[str(x) for x in raw.get("evidence_refs", [])],
        latency_ms=int(raw.get("latency_ms", 0)),
        cost_tokens=int(raw.get("cost_tokens", 0)),
        notes=str(raw.get("notes", "")),
    )
