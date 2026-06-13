"""Multi-pass incremental mutation tests."""

from __future__ import annotations

from ontologos import OntologyBuilder, Reasoner


def test_incremental_add_subclass_of() -> None:
    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.add_class("http://example.org/C")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    ontology = builder.build()

    reasoner = Reasoner(ontology=ontology, profile="el", incremental=True)
    first = reasoner.classify()
    first_pairs = set(map(tuple, first["subsumptions"]))
    assert ("http://example.org/A", "http://example.org/B") in first_pairs

    reasoner.add_subclass_of("http://example.org/B", "http://example.org/C")
    second = reasoner.classify()
    second_pairs = set(map(tuple, second["subsumptions"]))
    assert ("http://example.org/B", "http://example.org/C") in second_pairs
    assert len(second_pairs) >= len(first_pairs)


def test_add_axiom_json_subclass_of() -> None:
    builder = OntologyBuilder()
    builder.add_class("http://example.org/X")
    builder.add_class("http://example.org/Y")
    ontology = builder.build()

    reasoner = Reasoner(ontology=ontology, profile="el")
    reasoner.add_axiom_json(
        {
            "SubClassOf": {
                "subclass": "http://example.org/X",
                "superclass": "http://example.org/Y",
            }
        }
    )
    result = reasoner.classify()
    assert ("http://example.org/X", "http://example.org/Y") in map(
        tuple, result["subsumptions"]
    )


def test_remove_subclass_of() -> None:
    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    ontology = builder.build()

    reasoner = Reasoner(ontology=ontology, profile="el")
    reasoner.classify()
    reasoner.remove_subclass_of("http://example.org/A", "http://example.org/B")
    result = reasoner.classify()
    pairs = set(map(tuple, result["subsumptions"]))
    assert ("http://example.org/A", "http://example.org/B") not in pairs
