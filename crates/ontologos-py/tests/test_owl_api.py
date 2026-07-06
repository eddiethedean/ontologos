"""Tests for OWL API facade bindings (is_consistent, is_entailed, query)."""

from __future__ import annotations


def test_is_consistent_el_ontology() -> None:
    from ontologos import OntologyBuilder, Reasoner

    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    reasoner = Reasoner(ontology=builder.build(), profile="el")
    assert reasoner.is_consistent() is True


def test_is_entailed_subclass_chain() -> None:
    from ontologos import OntologyBuilder, Reasoner

    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.add_class("http://example.org/C")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    builder.subclass_of("http://example.org/B", "http://example.org/C")
    reasoner = Reasoner(ontology=builder.build(), profile="el")
    assert reasoner.is_entailed("http://example.org/A", "http://example.org/C") is True


def test_is_entailed_class_assertion() -> None:
    from ontologos import OntologyBuilder, Reasoner

    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.individual("http://example.org/x")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    builder.class_assertion("http://example.org/x", "http://example.org/A")
    reasoner = Reasoner(ontology=builder.build(), profile="el")
    assert (
        reasoner.is_entailed(
            individual="http://example.org/x",
            class_="http://example.org/B",
        )
        is True
    )


def test_query_direct_subclasses_after_classify() -> None:
    from ontologos import OntologyBuilder, Reasoner

    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.subclass_of("http://example.org/A", "http://example.org/B")
    reasoner = Reasoner(ontology=builder.build(), profile="el")
    answers = reasoner.query("Type(?x, http://example.org/B)")
    assert {"x": "http://example.org/A"} in answers


def test_query_unsatisfiable_class_returns_empty() -> None:
    from ontologos import OntologyBuilder, Reasoner

    nothing = "http://www.w3.org/2002/07/owl#Nothing"
    builder = OntologyBuilder()
    builder.add_class("http://example.org/A")
    builder.add_class("http://example.org/B")
    builder.subclass_of("http://example.org/A", nothing)
    builder.subclass_of("http://example.org/B", "http://example.org/A")
    reasoner = Reasoner(ontology=builder.build(), profile="el")
    answers = reasoner.query("Type(?x, http://example.org/A)")
    assert answers == []
