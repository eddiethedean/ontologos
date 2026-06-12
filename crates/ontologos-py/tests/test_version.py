"""Smoke tests for the ontologos PyPI placeholder."""

from __future__ import annotations

from pathlib import Path

FIXTURE = (
    Path(__file__).resolve().parents[2]
    / "ontologos-parser"
    / "tests"
    / "fixtures"
    / "minimal_subclass.owl"
)


def test_version_matches_release() -> None:
    import ontologos

    assert ontologos.__version__ == "0.3.0"


def test_reasoner_import() -> None:
    import ontologos

    assert ontologos.Reasoner is not None


def test_classify_not_implemented() -> None:
    import pytest
    from ontologos import Reasoner

    assert FIXTURE.is_file(), f"missing fixture: {FIXTURE}"
    reasoner = Reasoner(str(FIXTURE))
    with pytest.raises(RuntimeError, match="reasoning not yet implemented"):
        reasoner.classify()
