"""Tests for explain() bindings."""

from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
FIXTURE = (
    REPO_ROOT
    / "crates"
    / "ontologos-parser"
    / "tests"
    / "fixtures"
    / "minimal_subclass.owl"
)
FAMILY_OWL = REPO_ROOT / "benchmarks" / "data" / "family.owl"


def test_explain_el_minimal_fixture() -> None:
    from ontologos import Reasoner

    assert FIXTURE.is_file(), f"missing fixture: {FIXTURE}"
    reasoner = Reasoner(path=str(FIXTURE), profile="el")
    graph = reasoner.explain()
    assert graph["node_count"] > 0
    assert any("rule" in node for node in graph["nodes"])


def test_explain_rdfs_family_corpus() -> None:
    from ontologos import Reasoner

    assert FAMILY_OWL.is_file(), (
        f"missing family corpus at {FAMILY_OWL} (run ./benchmarks/scripts/download.sh)"
    )
    reasoner = Reasoner(path=str(FAMILY_OWL), profile="el")
    graph = reasoner.explain()
    assert graph["node_count"] > 0
    assert any("rule" in node for node in graph["nodes"])
