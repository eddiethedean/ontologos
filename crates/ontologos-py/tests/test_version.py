"""Smoke tests for the ontologos PyPI placeholder."""

from __future__ import annotations


def test_version_matches_release() -> None:
    import ontologos

    assert ontologos.__version__ == "0.2.0"


def test_reasoner_import() -> None:
    import ontologos

    assert ontologos.Reasoner is not None
