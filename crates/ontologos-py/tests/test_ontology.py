"""Tests for in-memory Ontology construction."""

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
PIZZA_MINIMAL_JSON = (
    REPO_ROOT
    / "crates"
    / "ontologos-core"
    / "tests"
    / "fixtures"
    / "pizza_minimal.json"
)


def test_ontology_from_json_round_trip() -> None:
    from ontologos import Ontology

    raw = PIZZA_MINIMAL_JSON.read_text()
    ontology = Ontology.from_json(raw)
    assert ontology.axiom_count == 2
    assert ontology.entity_count == 4
    restored = json.loads(ontology.to_json())
    assert restored["format_version"] == 3
    assert len(restored["axioms"]) == 2


def test_ontology_from_dict() -> None:
    from ontologos import Ontology

    data = json.loads(PIZZA_MINIMAL_JSON.read_text())
    ontology = Ontology.from_dict(data)
    assert ontology.axiom_count == 2


def test_ontology_builder_constructs_el_taxonomy() -> None:
    from ontologos import OntologyBuilder, Reasoner

    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    ontology = builder.build()

    reasoner = Reasoner(ontology=ontology, profile="el")
    result = reasoner.classify()
    assert result["subsumption_count"] >= 1
    pairs = set(map(tuple, result["subsumptions"]))
    assert ("http://example.org/A", "http://example.org/B") in pairs
