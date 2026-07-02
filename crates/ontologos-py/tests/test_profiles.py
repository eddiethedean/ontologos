"""Profile routing and constructor validation tests."""

from __future__ import annotations

from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
FAMILY_OWL = REPO_ROOT / "benchmarks" / "data" / "family.owl"
FIXTURE = (
    REPO_ROOT
    / "crates"
    / "ontologos-parser"
    / "tests"
    / "fixtures"
    / "minimal_subclass.owl"
)


def test_classify_rl_profile_infers_on_family() -> None:
    from ontologos import Reasoner

    assert FAMILY_OWL.is_file(), f"missing corpus: {FAMILY_OWL}"
    reasoner = Reasoner(path=str(FAMILY_OWL), profile="rl")
    result = reasoner.classify()
    assert result["inferred_axioms"] > 0
    assert result["final_axiom_count"] > result["initial_axiom_count"]


def test_classify_alc_profile_on_minimal_chain() -> None:
    from ontologos import OntologyBuilder, Reasoner

    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    ontology = builder.build()

    reasoner = Reasoner(ontology=ontology, profile="alc")
    result = reasoner.classify()
    assert ("http://example.org/A", "http://example.org/B") in map(
        tuple, result["subsumptions"]
    )


def test_classify_dl_profile_family() -> None:
    from ontologos import Reasoner

    assert FAMILY_OWL.is_file(), f"missing corpus: {FAMILY_OWL}"
    reasoner = Reasoner(path=str(FAMILY_OWL), profile="dl")
    result = reasoner.classify()
    assert result["subsumption_count"] > 0
    assert len(result["subsumptions"]) == result["subsumption_count"]


def test_classify_dl_preview_profile() -> None:
    from ontologos import OntologyBuilder, Reasoner

    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    ontology = builder.build()

    reasoner = Reasoner(ontology=ontology, profile="dl-preview")
    result = reasoner.classify()
    assert ("http://example.org/A", "http://example.org/B") in map(
        tuple, result["subsumptions"]
    )


def test_classify_swrl_profile_without_rules() -> None:
    from ontologos import OntologyBuilder, Reasoner

    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    ontology = builder.build()

    reasoner = Reasoner(ontology=ontology, profile="swrl")
    result = reasoner.classify()
    assert result["subsumption_count"] >= 1


def test_classify_el_asserts_known_subsumptions() -> None:
    from ontologos import Reasoner

    assert FIXTURE.is_file(), f"missing fixture: {FIXTURE}"
    reasoner = Reasoner(path=str(FIXTURE), profile="el")
    result = reasoner.classify()
    pairs = set(map(tuple, result["subsumptions"]))
    assert (
        "http://example.org/test#A",
        "http://example.org/test#B",
    ) in pairs


def test_reasoner_requires_exactly_one_source() -> None:
    from ontologos import OntologyBuilder, Reasoner

    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    ontology = builder.build()

    with pytest.raises(RuntimeError, match="exactly one of"):
        Reasoner(path=str(FIXTURE), ontology=ontology, profile="el")

    with pytest.raises(RuntimeError, match="exactly one of"):
        Reasoner(profile="el")


def test_invalid_profile_raises() -> None:
    from ontologos import Reasoner

    with pytest.raises(RuntimeError, match="unsupported profile"):
        Reasoner(path=str(FIXTURE), profile="bogus")
