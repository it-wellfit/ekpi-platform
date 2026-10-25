"""Runner: evaluates adapters over a JSONL case set and writes a report.

Usage (from the eval/ directory):

    python -m ekp_eval.runner --selftest
    python -m ekp_eval.runner --systems stub:oracle,my_adapter --out report_dir

Real adapters subclass SystemAdapter (see systems.py); external adapters can
be registered in build_registry (wired via entry points in R2-7).
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

from .metrics import summarize
from .schemas import load_case, decision_from_raw
from .systems import SystemAdapter, build_registry

DEFAULT_CASES = Path(__file__).with_name("cases") / "seed_cases.jsonl"


def load_cases(path: Path) -> list:
    cases = []
    with path.open("r", encoding="utf-8") as fh:
        for lineno, line in enumerate(fh, start=1):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            try:
                cases.append(load_case(json.loads(line)))
            except Exception as exc:  # noqa: BLE001 - report location then re-raise
                raise ValueError(f"{path}:{lineno}: {exc}") from exc
    if not cases:
        raise ValueError(f"no cases loaded from {path}")
    return cases


def run_system(adapter: SystemAdapter, cases) -> list:
    decisions = []
    for case in cases:
        raw = adapter.decide(case)
        payload = raw if isinstance(raw, dict) else {
            "case_id": raw.case_id,
            "verdict": raw.verdict,
            "confidence": raw.confidence,
            "cited_ids": raw.cited_ids,
            "rule_ids_used": raw.rule_ids_used,
            "evidence_refs": raw.evidence_refs,
            "latency_ms": raw.latency_ms,
            "cost_tokens": raw.cost_tokens,
            "notes": raw.notes,
        }
        decisions.append(decision_from_raw(adapter.name, payload))
    return decisions


def write_report(out_dir: Path, per_system: dict) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    report_payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "per_system": per_system,
    }
    (out_dir / "report.json").write_text(
        json.dumps(report_payload, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    lines = [
        "# EKPI eval report",
        "",
        "| system | n_cases | accuracy(nonabstain) | wrong_allow | viol_as_comply | refusal_on_definitive | correct_refusal | citation_recall | hard_gates_violated |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for name, s in per_system.items():
        lines.append(
            "| {name} | {n} | {acc} | {wa} | {vo} | {ro} | {cr} | {cir} | {gate} |".format(
                name=name,
                n=s.get("n_cases"),
                acc=_fmt(s.get("verdict_accuracy_nonabstain")),
                wa=s.get("wrong_allow"),
                vo=s.get("violation_as_comply"),
                ro=_fmt(s.get("refusal_on_definitive_rate")),
                cr=_fmt(s.get("correct_refusal_rate")),
                cir=_fmt(s.get("citation_recall")),
                gate=s.get("hard_gates_violated"),
            )
        )
    (out_dir / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _fmt(value) -> str:
    if value is None:
        return "-"
    if isinstance(value, bool):
        return str(value)
    if isinstance(value, float):
        return f"{value:.3f}"
    return str(value)


def selftest() -> int:
    cases = load_cases(DEFAULT_CASES)
    registry = build_registry()
    failures: list[str] = []

    def check(name: str, cond: bool, detail: str) -> None:
        status = "PASS" if cond else "FAIL"
        print(f"[{status}] {name}: {detail}")
        if not cond:
            failures.append(name)

    oracle = summarize(cases, run_system(registry["stub:oracle"], cases))
    allow = summarize(cases, run_system(registry["stub:allow_all"], cases))
    refuse = summarize(cases, run_system(registry["stub:refuse_all"], cases))

    check(
        "oracle verdict accuracy == 1.0",
        oracle["verdict_accuracy_nonabstain"] == 1.0,
        f"got {oracle['verdict_accuracy_nonabstain']}",
    )
    check(
        "oracle wrong_allow == 0 (H-1/H-2 plumbing)",
        oracle["wrong_allow"] == 0,
        f"got {oracle['wrong_allow']}",
    )
    check(
        "oracle refusal_on_definitive == 0.0",
        oracle["refusal_on_definitive_rate"] == 0.0,
        f"got {oracle['refusal_on_definitive_rate']}",
    )
    check(
        "oracle correct_refusal == 1.0",
        oracle["correct_refusal_rate"] == 1.0,
        f"got {oracle['correct_refusal_rate']}",
    )
    check(
        "oracle escalation_correct == 1.0",
        oracle["escalation_correct_rate"] == 1.0,
        f"got {oracle['escalation_correct_rate']}",
    )
    check(
        "oracle citation_recall == 1.0",
        oracle["citation_recall"] == 1.0,
        f"got {oracle['citation_recall']}",
    )
    check(
        "allow_all trips hard gate (violation_as_comply > 0)",
        allow["violation_as_comply"] > 0 and allow["hard_gates_violated"] is True,
        f"violation_as_comply={allow['violation_as_comply']}, "
        f"gate={allow['hard_gates_violated']}",
    )
    check(
        "allow_all wrong_allow counts every stop-expected case",
        allow["wrong_allow"] > 0,
        f"got {allow['wrong_allow']}",
    )
    check(
        "refuse_all refuses ALL definitive cases",
        refuse["refusal_on_definitive_rate"] == 1.0,
        f"got {refuse['refusal_on_definitive_rate']}",
    )
    check(
        "refuse_all correct_refusal == 1.0 (it obeys the `stop` cases)",
        refuse["correct_refusal_rate"] == 1.0,
        f"got {refuse['correct_refusal_rate']}",
    )

    if failures:
        print(f"\nselftest FAILED ({len(failures)} checks)")
        return 2
    print("\nselftest PASSED")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="ekp_eval")
    parser.add_argument("--cases", default=str(DEFAULT_CASES))
    parser.add_argument(
        "--systems",
        default="stub:oracle,stub:allow_all,stub:refuse_all",
        help="comma-separated adapter names registered in build_registry",
    )
    parser.add_argument("--out", default="report_out")
    parser.add_argument("--selftest", action="store_true")
    args = parser.parse_args(argv)

    if args.selftest:
        return selftest()

    cases = load_cases(Path(args.cases))
    registry = build_registry()
    per_system: dict[str, dict] = {}
    for name in [s.strip() for s in args.systems.split(",") if s.strip()]:
        adapter = registry.get(name)
        if adapter is None:
            print(f"unknown system adapter: {name}", file=sys.stderr)
            return 2
        decisions = run_system(adapter, cases)
        per_system[name] = summarize(cases, decisions)
    write_report(Path(args.out), per_system)
    print(f"report written to {os.path.abspath(args.out)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
