"""Optional pandas/polars export tests."""

from __future__ import annotations

import pytest


def _sample_taxonomy() -> dict:
    return {
        "subsumption_count": 2,
        "subsumptions": [
            ("http://example.org/A", "http://example.org/B"),
            ("http://example.org/B", "http://example.org/C"),
        ],
        "equivalences": [],
        "unsatisfiable": [],
    }


def test_subsumptions_to_pandas() -> None:
    pd = pytest.importorskip("pandas")
    from ontologos import subsumptions_to_pandas

    frame = subsumptions_to_pandas(_sample_taxonomy())
    assert list(frame.columns) == ["subclass", "superclass"]
    assert len(frame) == 2
    assert frame.iloc[0]["subclass"] == "http://example.org/A"


def test_subsumptions_to_polars() -> None:
    pl = pytest.importorskip("polars")
    from ontologos import subsumptions_to_polars

    frame = subsumptions_to_polars(_sample_taxonomy())
    assert frame.columns == ["subclass", "superclass"]
    assert frame.height == 2
    assert frame.row(0) == ("http://example.org/A", "http://example.org/B")
