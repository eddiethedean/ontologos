# Planned backlog triage

**Generated:** 2026-06-22 (UTC) via `benchmarks/scripts/audit-planned-backlog.sh`

Do not edit by hand — regenerate after catalog or engine changes.

## Summary

| Catalog | Planned |
|---------|--------:|
| HermiT Java (`cases.json`) | 322 |
| OWL WG (`wg_cases.json`) | 67 |

### Java by category

| Category | Count |
|----------|------:|
| `engine_gap` | 72 |
| `manual_port` | 207 |
| `missing_ofn` | 4 |
| `promotion_candidate` | 39 |

### WG by category

| Category | Count |
|----------|------:|
| `missing_expectations` | 15 |
| `missing_premise` | 52 |

## Promotion candidates (Java)

- `reasoner.DatalogEngineTest.testEquality` (dl)
- `reasoner.DatalogEngineTest.testQueryWithIndividualsAndEquality` (dl)
- `reasoner.DatalogEngineTest.testQueryWithIndividuals` (dl)
- `reasoner.DatatypesTest.testFreshEntitiesQuery` (dl)
- `reasoner.EntailmentTest.testHasKey` (dl)
- `reasoner.EntailmentTest.testBlankNodes1` (dl)
- `reasoner.EntailmentTest.testValidBlankNodesWithNominals` (dl)
- `reasoner.EntailmentTest.testValidBlankNodesInPremise` (dl)
- `reasoner.EntailmentTest.testValidBlankNodes` (dl)
- `reasoner.EntailmentTest.testBlankWithDTs2` (dl)
- `reasoner.EntailmentTest.testBlankWithDTs3` (dl)
- `reasoner.OWLReasonerTest.testBottomObjectPropertySubs` (rdfs)
- `reasoner.OWLReasonerTest.testTopObjectPropertySupers` (rdfs)
- `reasoner.ReasonerTest.testIsEntailed` (dl)
- `reasoner.ReasonerTest.testIncrementalWithNegatedHasValue` (dl)
- `reasoner.ReasonerTest.testIncrementalWithHasValue` (dl)
- `reasoner.ReasonerTest.testIncrementalWithNegatedHasSelf` (dl)
- `reasoner.ReasonerTest.testObjectPropertySubsumptionsNoNominals` (dl)
- `reasoner.ReasonerTest.testDataPropertyEntailment` (dl)
- `reasoner.ReasonerTest.testPropertyInstanceRetrieval` (dl)
- … and 19 more (see JSON)

## Engine gaps (sample Java)

- `reasoner.ComplexConceptTest.testConceptWithDatatypes` — reasoner.ComplexConceptTest.testConceptWithDatatypes: missing class file:/c/test.owl#desc
- `reasoner.ComplexConceptTest.testConceptWithDatatypes2` — reasoner.ComplexConceptTest.testConceptWithDatatypes2: missing class file:/c/test.owl#desc
- `reasoner.ComplexConceptTest.testConceptWithNominals` — reasoner.ComplexConceptTest.testConceptWithNominals: missing class file:/c/test.owl#desc
- `reasoner.ComplexConceptTest.testConceptWithNominals2` — reasoner.ComplexConceptTest.testConceptWithNominals2: missing class file:/c/test.owl#desc
- `reasoner.ComplexConceptTest.testConceptWithNominals3` — reasoner.ComplexConceptTest.testConceptWithNominals3: consistency expected false, got true
- `reasoner.ComplexConceptTest.testConceptWithNominals4` — reasoner.ComplexConceptTest.testConceptWithNominals4: consistency expected false, got true
- `reasoner.ComplexConceptTest.testConceptWithNominals5` — reasoner.ComplexConceptTest.testConceptWithNominals5: missing file:/c/test.owl#A
- `reasoner.ComplexConceptTest.testJustifications` — reasoner.ComplexConceptTest.testJustifications: class file:/c/test.owl#Matt satisfiability expected false, got true
- `reasoner.DatalogEngineTest.testBasic` — reasoner.DatalogEngineTest.testBasic: datalog class query :A expected [], got {":a"}
- `reasoner.DatatypesTest.testLiteralCustomDatatype` — reasoner.DatatypesTest.testLiteralCustomDatatype: expected ontology load to fail
- `reasoner.DatatypesTest.testParsingError` — reasoner.DatatypesTest.testParsingError: expected ontology load to fail
- `reasoner.EntailmentTest.testInvalidBlankNodes` — reasoner.EntailmentTest.testInvalidBlankNodes: entailment expected false, got true
- `reasoner.EntailmentTest.testBlankWithDTs` — reasoner.EntailmentTest.testBlankWithDTs: entailment expected false, got true
- `reasoner.OWLReasonerTest.testIncrementalAddition2` — reasoner.OWLReasonerTest.testIncrementalAddition2: consistency expected false, got true
- `reasoner.ReasonerCoreBlockingTest.testIanT6` — reasoner.ReasonerCoreBlockingTest.testIanT6: class file:/c/test.owl#c satisfiability expected false, got true
- … and 57 more (see JSON)
