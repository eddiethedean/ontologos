"""Smoke tests for the ontologos Python bindings."""

from __future__ import annotations

import json
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
PIZZA_OWL = REPO_ROOT / "benchmarks" / "data" / "pizza.owl"
PIZZA_GOLDEN = REPO_ROOT / "benchmarks" / "data" / "pizza-el-golden.json"
FAMILY_OWL = REPO_ROOT / "benchmarks" / "data" / "family.owl"
PIZZA_MINIMAL_JSON = (
    REPO_ROOT
    / "crates"
    / "ontologos-core"
    / "tests"
    / "fixtures"
    / "pizza_minimal.json"
)


def test_version_matches_release() -> None:
    import ontologos

    assert ontologos.__version__ == "1.0.1"


def test_reasoner_import() -> None:
    import ontologos

    assert ontologos.Reasoner is not None
    assert ontologos.Ontology is not None
    assert ontologos.OntologyBuilder is not None


def test_classify_el_profile_returns_taxonomy() -> None:
    from ontologos import Reasoner

    assert FIXTURE.is_file(), f"missing fixture: {FIXTURE}"
    reasoner = Reasoner(path=str(FIXTURE), profile="el")
    result = reasoner.classify()
    pairs = set(map(tuple, result["subsumptions"]))
    assert (
        "http://example.org/test#A",
        "http://example.org/test#B",
    ) in pairs
    assert result["subsumption_count"] >= 1


def test_classify_auto_profile_routes_el_fixture() -> None:
    from typing import cast

    from ontologos import Reasoner
    from ontologos.types import TaxonomyResult

    assert FIXTURE.is_file(), f"missing fixture: {FIXTURE}"
    reasoner = Reasoner(path=str(FIXTURE), profile="auto")
    result = cast(TaxonomyResult, reasoner.classify())
    pairs = set(map(tuple, result["subsumptions"]))
    assert (
        "http://example.org/test#A",
        "http://example.org/test#B",
    ) in pairs


def test_classify_rdfs_profile_materializes() -> None:
    from ontologos import Reasoner

    assert FIXTURE.is_file(), f"missing fixture: {FIXTURE}"
    reasoner = Reasoner(path=str(FIXTURE), profile="rdfs")
    result = reasoner.classify()
    assert result["inferred_axioms"] >= 0
    assert result["final_axiom_count"] >= result["initial_axiom_count"]


def test_parse_meta_exposes_warnings_for_kind_clash() -> None:
    from ontologos import Reasoner

    clash = (
        REPO_ROOT
        / "crates"
        / "ontologos-parser"
        / "tests"
        / "fixtures"
        / "subclass_data_property_decl_first.ttl"
    )
    assert clash.is_file(), f"missing fixture: {clash}"
    reasoner = Reasoner(path=str(clash), profile="rdfs")
    meta = reasoner.parse_meta
    assert meta["skipped_axiom_count"] == 1
    assert meta["logical_axiom_count"] == 1
    assert any("entity kind mismatch" in warning for warning in meta["warnings"])
