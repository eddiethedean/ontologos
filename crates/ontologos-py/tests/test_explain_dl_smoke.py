"""DL explain smoke — parity with Rust CLI explain path."""

from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
FAMILY_OWL = REPO_ROOT / "benchmarks" / "data" / "family.owl"


def test_explain_dl_family_corpus() -> None:
    from ontologos import Reasoner

    assert FAMILY_OWL.is_file(), (
        f"missing family corpus at {FAMILY_OWL} (run ./benchmarks/scripts/download.sh)"
    )
    reasoner = Reasoner(path=str(FAMILY_OWL), profile="dl")
    reasoner.classify()
    graph = reasoner.explain()
    assert graph["node_count"] > 0
    assert isinstance(graph["nodes"], list)
    assert any("rule" in node for node in graph["nodes"])
