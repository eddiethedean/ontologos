#!/usr/bin/env python3
"""Regression tests for HermiT Java assertion harvest extractors."""

from __future__ import annotations

import unittest

from assertion_extractors import (
    extract_assert_satisfiable,
    extract_ce_satisfiability_fallback,
    extract_datalog_queries,
    extract_entailment_metadata,
    extract_equivalence_subsumptions,
    extract_has_key_entailment,
    extract_incremental_ofn,
    extract_individual_types,
    extract_property_hierarchy,
    extract_subproperty_chain_entailment,
)


class AssertionExtractorTests(unittest.TestCase):
    def test_assert_satisfiable_string(self) -> None:
        body = 'assertSatisfiable("file:/c/test.owl#test", true);'
        out = extract_assert_satisfiable(body)
        self.assertEqual(len(out), 1)
        self.assertEqual(out[0]["class"], ":test")
        self.assertTrue(out[0]["expected"])

    def test_ce_satisfiability_fallback(self) -> None:
        body = """
        OWLClassExpression desc1 = df.getOWLClass(IRI.create("file:/c/test.owl#p1"));
        assertSatisfiable(desc1, false);
        """
        out = extract_ce_satisfiability_fallback(body)
        self.assertEqual(out[0]["class"], ":p1")
        self.assertFalse(out[0]["expected"])

    def test_entailment_metadata(self) -> None:
        body = """
        loadReasonerWithAxioms(axioms);
        axioms = "ClassAssertion(ObjectSomeValuesFrom(:p owl:Thing) :a)";
        OWLOntology conlusions=getOntologyWithAxioms(axioms);
        assertEntails(conlusions.getLogicalAxioms(), true);
        """
        conclusion, expected = extract_entailment_metadata(body)
        self.assertIn("ClassAssertion", conclusion or "")
        self.assertTrue(expected)

    def test_has_key_entailment(self) -> None:
        body = """
        OWLClass C=m_dataFactory.getOWLClass(IRI.create(ReasonerTest.NS + "Man"));
        OWLObjectProperty p=m_dataFactory.getOWLObjectProperty(IRI.create(ReasonerTest.NS + "hasSSN"));
        assertEntails(m_dataFactory.getOWLHasKeyAxiom(C, p), true);
        """
        conclusion, expected = extract_has_key_entailment(body)
        self.assertIn("HasKey", conclusion or "")
        self.assertTrue(expected)

    def test_property_hierarchy(self) -> None:
        body = 'assertDirectSuperObjectProperties("op1", EQ("op2"));'
        obj, data = extract_property_hierarchy(body)
        self.assertEqual(obj[0]["sub"], ":op1")
        self.assertEqual(obj[0]["sup"], ":op2")
        self.assertFalse(data)

    def test_incremental_ofn(self) -> None:
        body = """
        addAxioms(m_ontology, "SubClassOf(:A :B)");
        assertFalse(m_reasoner.isConsistent());
        """
        inc = extract_incremental_ofn(body)
        self.assertIn("SubClassOf", inc)

    def test_individual_types(self) -> None:
        body = """
        assertTrue(m_reasoner.hasType(NS_NI("a"), NS_C("B"), false));
        """
        out = extract_individual_types(body)
        self.assertEqual(out[0]["individual"], ":a")
        self.assertEqual(out[0]["class"], ":B")

    def test_datalog_class_query(self) -> None:
        body = """
        new ConjunctiveQuery(datalogEngine,
            AS(
                A(CN("A"),V("X"))
            ),
            TS(
                V("X")
            )
        ).evaluate(queryChecker);
        queryChecker.add(I("a")).add(I("b")).assertEquals();
        """
        queries = extract_datalog_queries(body)
        self.assertEqual(len(queries), 1)
        self.assertEqual(queries[0]["atoms"][0]["class"], ":A")
        self.assertEqual(queries[0]["answers"], [":a", ":b"])

    def test_equivalence_subsumptions(self) -> None:
        body = 'assertEquivalentClasses(NS_C("A"), NS_C("B"));'
        subs = extract_equivalence_subsumptions(body)
        self.assertEqual(len(subs), 2)

    def test_subproperty_chain_entailment(self) -> None:
        body = """
        String axioms = "TransitiveObjectProperty( :p)";
        loadReasonerWithAxioms(axioms);
        assertEntails(ax, true);
        SubObjectPropertyOf(ObjectPropertyChain(:p :p) :p)
        """
        conclusion, expected = extract_subproperty_chain_entailment(body)
        self.assertIn("ObjectPropertyChain", conclusion or "")
        self.assertTrue(expected)


if __name__ == "__main__":
    unittest.main()
