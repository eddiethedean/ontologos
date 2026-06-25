#!/usr/bin/env python3
"""Regression tests for OWL WG catalog extraction helpers."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from generate_catalog import (
    detect_wg_test_type,
    extract_wg_embedded_content,
    wg_test_block,
    write_wg_fixture,
)

NEGATIVE_ENTAILMENT_BLOCK = """
<rdf:Description rdf:about="http://owl.semanticweb.org/id/TestCase-3AWebOnt-2DI4.6-2D004">
  <rdf:type rdf:resource="&test;NegativeEntailmentTest"/>
  <test:status rdf:resource="&test;Approved"/>
  <test:species rdf:resource="&test;DL"/>
  <test:rdfXmlPremiseOntology>&lt;?xml version="1.0"?&gt;
&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"&gt;
  &lt;owl:Ontology/&gt;
  &lt;owl:Class rdf:about="http://ex.org/A"/&gt;
&lt;/rdf:RDF&gt;</test:rdfXmlPremiseOntology>
  <test:rdfXmlNonConclusionOntology>&lt;?xml version="1.0"?&gt;
&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"&gt;
  &lt;owl:Ontology/&gt;
  &lt;owl:Class rdf:about="http://ex.org/B"/&gt;
&lt;/rdf:RDF&gt;</test:rdfXmlNonConclusionOntology>
</rdf:Description>
<rdf:Description rdf:about="http://owl.semanticweb.org/id/Next-Case">
</rdf:Description>
"""

INCONSISTENCY_BLOCK = """
<rdf:Description rdf:about="http://owl.semanticweb.org/id/Inconsistent-2Dpattern-2Ddisjointness">
  <rdf:type rdf:resource="&test;InconsistencyTest"/>
  <test:status rdf:resource="&test;Approved"/>
  <test:species rdf:resource="&test;DL"/>
  <test:rdfXmlPremiseOntology>&lt;?xml version="1.0"?&gt;
&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"&gt;
  &lt;owl:Ontology/&gt;
  &lt;owl:Class rdf:about="http://ex.org/X"/&gt;
&lt;/rdf:RDF&gt;</test:rdfXmlPremiseOntology>
</rdf:Description>
<rdf:Description rdf:about="http://owl.semanticweb.org/id/Next-Case">
</rdf:Description>
"""

FS_PREMISE_BLOCK = """
<rdf:Description rdf:about="http://owl.semanticweb.org/id/FS-Only-Case">
  <rdf:type rdf:resource="&test;ConsistencyTest"/>
  <test:status rdf:resource="&test;Approved"/>
  <test:species rdf:resource="&test;DL"/>
  <test:fsPremiseOntology>Prefix(:=&lt;http://example.org/&gt;)
Ontology(
  Declaration(Class(:A))
)</test:fsPremiseOntology>
</rdf:Description>
"""


class WgExtractionTests(unittest.TestCase):
    def test_wg_test_block_stops_at_next_case(self) -> None:
        text = NEGATIVE_ENTAILMENT_BLOCK
        start = text.index("TestCase-3AWebOnt-2DI4.6-2D004") - 50
        block = wg_test_block(text, start)
        self.assertIn("TestCase-3AWebOnt-2DI4.6-2D004", block)
        self.assertNotIn("Next-Case", block)

    def test_detect_negative_entailment(self) -> None:
        tt, ent, cons = detect_wg_test_type(NEGATIVE_ENTAILMENT_BLOCK)
        self.assertEqual(tt, "negative_entailment")
        self.assertFalse(ent)
        self.assertIsNone(cons)

    def test_detect_inconsistency_before_consistency_bleed(self) -> None:
        tt, ent, cons = detect_wg_test_type(INCONSISTENCY_BLOCK)
        self.assertEqual(tt, "inconsistency")
        self.assertIsNone(ent)
        self.assertFalse(cons)

    def test_extract_non_conclusion_for_negative_entailment(self) -> None:
        prem = extract_wg_embedded_content(
            NEGATIVE_ENTAILMENT_BLOCK, "rdfXmlPremiseOntology"
        )
        conc = extract_wg_embedded_content(
            NEGATIVE_ENTAILMENT_BLOCK, "rdfXmlNonConclusionOntology"
        )
        self.assertIsNotNone(prem)
        self.assertTrue(prem.startswith("<"))
        self.assertIsNotNone(conc)
        self.assertIn("ex.org/B", conc)

    def test_write_wg_fixture_negative_entailment(self) -> None:
        import generate_catalog as gc

        with tempfile.TemporaryDirectory() as tmp:
            gc.OUT_WG_DATA = Path(tmp)
            prem_rel, conc_rel = write_wg_fixture(
                "TestCase-3AWebOnt-2DI4.6-2D004",
                NEGATIVE_ENTAILMENT_BLOCK,
                negative_entailment=True,
            )
            self.assertEqual(prem_rel, "wg/TestCase-3AWebOnt-2DI4.6-2D004/premise.rdf")
            self.assertEqual(
                conc_rel, "wg/TestCase-3AWebOnt-2DI4.6-2D004/conclusion.rdf"
            )
            prem_path = Path(tmp) / "TestCase-3AWebOnt-2DI4.6-2D004" / "premise.rdf"
            conc_path = Path(tmp) / "TestCase-3AWebOnt-2DI4.6-2D004" / "conclusion.rdf"
            self.assertTrue(prem_path.is_file())
            self.assertTrue(conc_path.is_file())

    def test_write_wg_fixture_fs_premise(self) -> None:
        import generate_catalog as gc

        with tempfile.TemporaryDirectory() as tmp:
            gc.OUT_WG_DATA = Path(tmp)
            prem_rel, conc_rel = write_wg_fixture("FS-Only-Case", FS_PREMISE_BLOCK)
            self.assertEqual(prem_rel, "wg/FS-Only-Case/premise.ofn")
            self.assertIsNone(conc_rel)
            prem_path = Path(tmp) / "FS-Only-Case" / "premise.ofn"
            self.assertTrue(prem_path.is_file())
            self.assertIn("Prefix", prem_path.read_text())


    def test_merge_rdf_xml_drops_imports(self) -> None:
        from generate_catalog import merge_rdf_xml

        main = """<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:Ontology><owl:imports rdf:resource="imports007"/></owl:Ontology>
  <owl:Thing/>
</rdf:RDF>"""
        imported = """<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:ObjectProperty rdf:ID="p"/>
</rdf:RDF>"""
        merged = merge_rdf_xml(main, imported)
        self.assertIn("ObjectProperty", merged)
        self.assertNotIn("owl:imports", merged)
        self.assertIn("owl:Thing", merged)


if __name__ == "__main__":
    unittest.main()
