"""EKPI eval harness.

Ablation-style evaluation skeleton for the Enterprise Knowledge & Process
Intelligence platform.  Pure stdlib; real system adapters plug in later
(addendum A7/A12).  Run:

    python -m ekp_eval.runner --selftest
"""

__version__ = "0.1.0"

VERDICTS = frozenset(
    {"comply", "violate", "insufficient_evidence", "escalate"}
)
