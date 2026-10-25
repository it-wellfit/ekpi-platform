"""Metrics for EKPI evaluation (addendum A5/A7).

Verdict classes: comply | violate | insufficient_evidence | escalate.

Hard-gate quantities (must be zero for a deliverable):
  wrong_allow          - expected != comply but system answered comply
  violation_as_comply  - expected violate answered comply (subset of wrong_allow)

Calibration quantities:
  refusal_on_definitive_rate - refused a case that required a verdict
  correct_refusal_rate       - refused exactly where required
  escalation_correct_rate    - escalate expected and answered escalate
"""

from __future__ import annotations

from collections import Counter
from typing import Iterable, Sequence

from .schemas import Decision, TestCase

_REFUSALS = {"insufficient_evidence"}


def recall_at_k(expected: Sequence[str], retrieved: Sequence[str], k: int) -> float:
    if not expected:
        return 0.0
    hit = len(set(expected) & set(retrieved[:k]))
    return hit / len(set(expected))


def mrr(expected: Sequence[str], retrieved: Sequence[str]) -> float:
    expected_set = set(expected)
    for rank, item in enumerate(retrieved, start=1):
        if item in expected_set:
            return 1.0 / rank
    return 0.0


def citation_scores(
    expected: Sequence[str], cited: Sequence[str]
) -> tuple[float | None, float | None]:
    """Return (recall, precision); None when the denominator family is empty."""

    def norm(xs: Iterable[str]) -> set[str]:
        return {x.strip() for x in xs}

    exp, cit = norm(expected), norm(cited)
    recall = len(exp & cit) / len(exp) if exp else None
    precision = len(exp & cit) / len(cit) if cit else None
    return recall, precision


def _is_correct(case: TestCase, decision: Decision) -> bool:
    if decision.verdict == case.expected_verdict:
        return True
    # escalate may substitute for insufficient_evidence (both are "stop").
    if (
        case.expected_verdict == "insufficient_evidence"
        and decision.verdict == "escalate"
    ):
        return True
    return False


def risk_coverage(
    rows: Sequence[tuple[float, bool]], n_bins: int = 10
) -> list[dict[str, float]]:
    """Risk–coverage curve from (confidence, is_correct) rows.

    Rows are sorted by confidence descending; coverage is the fraction of
    cases accepted so far and risk is the observed error rate under coverage.
    """

    if not rows:
        return []
    ordered = sorted(rows, key=lambda r: r[0], reverse=True)
    total = len(ordered)
    points: list[dict[str, float]] = []
    errors = 0
    for i, (_, ok) in enumerate(ordered, start=1):
        if not ok:
            errors += 1
        points.append(
            {"coverage": i / total, "risk": errors / i}
        )
    return points


def summarize(cases: Sequence[TestCase], decisions: Sequence[Decision]) -> dict:
    by_case: dict[str, Decision] = {d.case_id: d for d in decisions}
    definitive = [c for c in cases if not c.abstain_required]
    abstain_cases = [c for c in cases if c.abstain_required]

    wrong_allow = 0
    violation_as_comply = 0
    definitive_correct = 0
    refusal_on_definitive = 0
    correct_refusals = 0
    escalation_expected = [c for c in definitive if c.expected_verdict == "escalate"]
    escalation_correct = 0

    cite_recalls: list[float] = []
    cite_precisions: list[float] = []
    rcs: list[tuple[float, bool]] = []
    latencies: list[int] = []
    costs: list[int] = []

    confusion: Counter[str] = Counter()

    for case in cases:
        dec = by_case.get(case.case_id)
        if dec is None:
            confusion[f"{case.expected_verdict}|missing"] += 1
            continue
        confusion[f"{case.expected_verdict}|{dec.verdict}"] += 1
        ok = _is_correct(case, dec)
        rcs.append((dec.confidence, ok))
        latencies.append(dec.latency_ms)
        costs.append(dec.cost_tokens)

        if dec.verdict == "comply" and case.expected_verdict != "comply":
            wrong_allow += 1
            if case.expected_verdict == "violate":
                violation_as_comply += 1

        if not case.abstain_required:
            if dec.verdict == case.expected_verdict:
                definitive_correct += 1
            if dec.verdict in _REFUSALS:
                refusal_on_definitive += 1
            if (
                case.expected_verdict == "escalate"
                and dec.verdict == "escalate"
            ):
                escalation_correct += 1

        else:
            if dec.verdict == case.expected_verdict or (
                case.expected_verdict == "insufficient_evidence"
                and dec.verdict == "escalate"
            ):
                correct_refusals += 1

        if case.citations:
            rec, prec = citation_scores(case.citations, dec.cited_ids)
            if rec is not None:
                cite_recalls.append(rec)
            if prec is not None:
                cite_precisions.append(prec)

    n_def = len(definitive)
    n_abs = len(abstain_cases)
    summary = {
        "n_cases": len(cases),
        "n_matched": len(by_case),
        "verdict_accuracy_nonabstain": (
            definitive_correct / n_def if n_def else None
        ),
        "wrong_allow": wrong_allow,
        "violation_as_comply": violation_as_comply,
        "refusal_on_definitive_rate": (
            refusal_on_definitive / n_def if n_def else None
        ),
        "correct_refusal_rate": (
            correct_refusals / n_abs if n_abs else None
        ),
        "escalation_correct_rate": (
            escalation_correct / len(escalation_expected)
            if escalation_expected
            else None
        ),
        "citation_recall": (
            sum(cite_recalls) / len(cite_recalls) if cite_recalls else None
        ),
        "citation_precision": (
            sum(cite_precisions) / len(cite_precisions) if cite_precisions else None
        ),
        "mean_confidence": (
            sum(c for c, _ in rcs) / len(rcs) if rcs else None
        ),
        "mean_latency_ms": (
            sum(latencies) / len(latencies) if latencies else None
        ),
        "mean_cost_tokens": (
            sum(costs) / len(costs) if costs else None
        ),
        "confusion_matrix": dict(sorted(confusion.items())),
        "risk_coverage": risk_coverage(rcs),
    }
    summary["hard_gates_violated"] = bool(
        summary["violation_as_comply"] or summary["wrong_allow"]
    )
    return summary
