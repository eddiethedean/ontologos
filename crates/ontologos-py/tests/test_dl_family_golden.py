"""Family DL golden — Python classify parity with Rust CLI Tier C gate."""

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
FAMILY_OWL = REPO_ROOT / "benchmarks" / "data" / "family.owl"
DL_GOLDEN = REPO_ROOT / "benchmarks" / "data" / "dl-taxonomy-golden.json"


def test_family_dl_matches_cli_golden() -> None:
    from ontologos import Reasoner

    assert FAMILY_OWL.is_file(), f"missing corpus: {FAMILY_OWL}"
    assert DL_GOLDEN.is_file(), f"missing golden: {DL_GOLDEN}"

    doc = json.loads(DL_GOLDEN.read_text())
    golden = doc["corpora"]["family.owl"]

    reasoner = Reasoner(path=str(FAMILY_OWL), profile="dl")
    result = reasoner.classify()

    assert result["subsumption_count"] == golden["subsumption_count"]
    actual = set(map(tuple, result["subsumptions"]))
    expected = set(map(tuple, golden["subsumptions"]))
    missing = expected - actual
    extra = actual - expected
    assert not missing, f"missing {len(missing)} subsumptions from golden"
    assert not extra, f"extra {len(extra)} subsumptions vs golden"
