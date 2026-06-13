"""Smoke tests for the ontologos Python bindings."""

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

    assert ontologos.__version__ == "0.6.1"


def test_reasoner_import() -> None:
    import ontologos

    assert ontologos.Reasoner is not None


def test_classify_el_profile_returns_taxonomy() -> None:
    from ontologos import Reasoner

    assert FIXTURE.is_file(), f"missing fixture: {FIXTURE}"
    reasoner = Reasoner(str(FIXTURE), profile="el")
    result = reasoner.classify()
    assert "subsumption_count" in result
    assert "subsumptions" in result
    assert result["subsumption_count"] >= 1


def test_classify_auto_profile_routes_el_fixture() -> None:
    from ontologos import Reasoner

    assert FIXTURE.is_file(), f"missing fixture: {FIXTURE}"
    reasoner = Reasoner(str(FIXTURE), profile="auto")
    result = reasoner.classify()
    assert "subsumption_count" in result
    assert result["subsumption_count"] >= 1


def test_classify_rdfs_profile_materializes() -> None:
    from ontologos import Reasoner

    assert FIXTURE.is_file(), f"missing fixture: {FIXTURE}"
    reasoner = Reasoner(str(FIXTURE), profile="rdfs")
    result = reasoner.classify()
    assert "inferred_axioms" in result


def test_parse_meta_exposes_warnings_for_kind_clash() -> None:
    from ontologos import Reasoner

    clash = (
        Path(__file__).resolve().parents[2]
        / "ontologos-parser"
        / "tests"
        / "fixtures"
        / "class_individual_kind_clash.ttl"
    )
    assert clash.is_file(), f"missing fixture: {clash}"
    reasoner = Reasoner(str(clash), profile="rdfs")
    meta = reasoner.parse_meta
    assert meta["skipped_axiom_count"] == 1
    assert meta["logical_axiom_count"] == 1
    assert any("entity kind mismatch" in warning for warning in meta["warnings"])
