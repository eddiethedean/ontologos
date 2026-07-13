"""Python binding checks against shared semantic fixtures."""

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
FIXTURES = REPO_ROOT / "benchmarks" / "data" / "semantic-fixtures.json"


def _load_fixtures() -> dict:
    return json.loads(FIXTURES.read_text(encoding="utf-8"))


def test_semantic_fixture_el_minimal() -> None:
    from ontologos import Reasoner

    spec = _load_fixtures()["el_minimal_subclass"]
    path = REPO_ROOT / spec["fixture"]
    assert path.is_file(), f"missing {path}"
    reasoner = Reasoner(path=str(path), profile="el", trusted=True, lenient=True)
    result = reasoner.classify()
    pairs = set(map(tuple, result["subsumptions"]))
    for sub, sup in spec["expected_subsumptions"]:
        assert (sub, sup) in pairs, f"missing {sub} ⊑ {sup}"


def test_semantic_fixture_pizza_el_pair() -> None:
    from ontologos import Reasoner

    spec = _load_fixtures()["pizza_el"]
    path = REPO_ROOT / spec["fixture"]
    assert path.is_file(), f"missing {path} — run benchmarks/scripts/download.sh"
    reasoner = Reasoner(path=str(path), profile="el", trusted=True, lenient=True)
    result = reasoner.classify()
    pairs = set(map(tuple, result["subsumptions"]))
    for sub, sup in spec["expected_subsumptions"]:
        assert (sub, sup) in pairs, f"missing {sub} ⊑ {sup}"
