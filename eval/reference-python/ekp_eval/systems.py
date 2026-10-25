"""System adapters. Real systems plug in by subclassing SystemAdapter.

Stub systems in this file are deterministic and exist ONLY to validate the
metrics and hard-gate plumbing (selftest). They are not baselines.
"""

from __future__ import annotations

import abc

from .schemas import Decision, TestCase


class SystemAdapter(abc.ABC):
    name: str = "adapter"

    @abc.abstractmethod
    def decide(self, case: TestCase) -> Decision:
        """Produce one Decision for one test case."""


class StubAllowAll(SystemAdapter):
    """Always answers comply — must fail hard gates. Not a baseline."""

    name = "stub:allow_all"

    def decide(self, case: TestCase) -> Decision:
        return Decision(
            case_id=case.case_id,
            system=self.name,
            verdict="comply",
            confidence=1.0,
            cited_ids=[],
            rule_ids_used=[],
            latency_ms=1,
            cost_tokens=0,
            notes="stub: unconditional comply",
        )


class StubRefuseAll(SystemAdapter):
    """Always refuses — acceptable only where refusal is required."""

    name = "stub:refuse_all"

    def decide(self, case: TestCase) -> Decision:
        return Decision(
            case_id=case.case_id,
            system=self.name,
            verdict="insufficient_evidence",
            confidence=0.95,
            cited_ids=[],
            rule_ids_used=[],
            latency_ms=1,
            cost_tokens=0,
            notes="stub: unconditional refusal",
        )


class StubOracle(SystemAdapter):
    """Reads annotations — proves the harness can reach 100% when verdicts are right."""

    name = "stub:oracle"

    def decide(self, case: TestCase) -> Decision:
        conf = 0.99 if case.abstain_required else 1.0
        return Decision(
            case_id=case.case_id,
            system=self.name,
            verdict=case.expected_verdict,
            confidence=conf,
            cited_ids=list(case.citations),
            rule_ids_used=list(case.rules),
            latency_ms=2,
            cost_tokens=1,
            notes="stub: annotated answer",
        )


def build_registry(additional: dict[str, SystemAdapter] | None = None) -> dict[str, SystemAdapter]:
    registry: dict[str, SystemAdapter] = {}
    for adapter in (StubAllowAll(), StubRefuseAll(), StubOracle()):
        registry[adapter.name] = adapter
    if additional:
        registry.update(additional)
    return registry
