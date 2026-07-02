"""Multi-pass incremental mutation tests."""

from __future__ import annotations

from ontologos import Ontology, OntologyBuilder, Reasoner


def _chain_ontology() -> Ontology:
    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.add_class("http://example.org/C")
    builder.add_class("http://example.org/D")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    builder.subclass_of("http://example.org/B", "http://example.org/C")
    return builder.build()


def test_reasoner_mutations_sync_to_ontology() -> None:
    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    ontology = builder.build()
    assert ontology.axiom_count == 0

    reasoner = Reasoner(ontology=ontology, profile="el")
    reasoner.add_subclass_of("http://example.org/A", "http://example.org/B")
    assert ontology.axiom_count == 1


def test_incremental_matches_full_classify() -> None:
    ontology = _chain_ontology()

    full = Reasoner(ontology=ontology, profile="el", incremental=False)
    full_result = set(map(tuple, full.classify()["subsumptions"]))

    incr = Reasoner(ontology=ontology, profile="el", incremental=True)
    incr_result = set(map(tuple, incr.classify()["subsumptions"]))

    assert incr_result == full_result

    incr.add_subclass_of("http://example.org/C", "http://example.org/D")
    incr_second = set(map(tuple, incr.classify()["subsumptions"]))

    full_fresh = Reasoner(ontology=ontology, profile="el", incremental=False)
    full_second = set(map(tuple, full_fresh.classify()["subsumptions"]))

    assert incr_second == full_second
    assert ("http://example.org/C", "http://example.org/D") in incr_second


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

    full = Reasoner(ontology=ontology, profile="el", incremental=False)
    full_pairs = set(map(tuple, full.classify()["subsumptions"]))
    assert second_pairs == full_pairs


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


def test_remove_subclass_of_incremental() -> None:
    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    ontology = builder.build()

    reasoner = Reasoner(ontology=ontology, profile="el", incremental=True)
    reasoner.classify()
    reasoner.remove_subclass_of("http://example.org/A", "http://example.org/B")
    result = reasoner.classify()
    pairs = set(map(tuple, result["subsumptions"]))
    assert ("http://example.org/A", "http://example.org/B") not in pairs

    full = Reasoner(ontology=ontology, profile="el", incremental=False)
    full_pairs = set(map(tuple, full.classify()["subsumptions"]))
    assert pairs == full_pairs


def test_reasoner_taxonomy_matches_classify_result() -> None:
    ontology = _chain_ontology()
    reasoner = Reasoner(ontology=ontology, profile="el")
    result = reasoner.classify()
    taxonomy = reasoner.taxonomy
    assert taxonomy is not None
    assert taxonomy["subsumption_count"] == result["subsumption_count"]
    assert set(map(tuple, taxonomy["subsumptions"])) == set(
        map(tuple, result["subsumptions"])
    )
