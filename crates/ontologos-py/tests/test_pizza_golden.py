"""Pizza EL golden integration test — matches Rust CLI output."""

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
PIZZA_OWL = REPO_ROOT / "benchmarks" / "data" / "pizza.owl"
PIZZA_GOLDEN = REPO_ROOT / "benchmarks" / "data" / "pizza-el-golden.json"


def test_pizza_el_matches_cli_golden() -> None:
    from ontologos import Reasoner

    assert PIZZA_OWL.is_file(), (
        f"missing pizza corpus at {PIZZA_OWL} (run ./benchmarks/scripts/download.sh)"
    )
    assert PIZZA_GOLDEN.is_file(), f"missing golden file: {PIZZA_GOLDEN}"

    golden = json.loads(PIZZA_GOLDEN.read_text())
    reasoner = Reasoner(path=str(PIZZA_OWL), profile="el")
    result = reasoner.classify()

    assert result["subsumption_count"] == golden["subsumption_count"]
    actual = set(map(tuple, result["subsumptions"]))
    expected = set(map(tuple, golden["subsumptions"]))
    missing = expected - actual
    extra = actual - expected
    assert not missing, f"missing {len(missing)} subsumptions from golden"
    assert not extra, f"extra {len(extra)} subsumptions vs golden"
