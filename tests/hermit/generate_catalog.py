#!/usr/bin/env python3
"""Generate HermiT port catalog (JSON + Rust tests) from a local hermit-reasoner checkout.

Run from repo root:
  python3 tests/hermit/generate_catalog.py
  python3 tests/hermit/generate_catalog.py --activate-all-from-disk

By default all runnable cases are active (failure-first). Use --promoted-only to
gate on promoted_axiom_ids.txt / promoted_wg_ids.txt (legacy promotion workflow).

Requires HermiT/ (owlcs/hermit-reasoner) or ONTOLOGOS_HERMIT_ROOT for full regen.
"""

from __future__ import annotations

import html
import json
import os
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
HERMIT = Path(os.environ.get("ONTOLOGOS_HERMIT_ROOT", REPO / "HermiT"))

from assertion_extractors import (
    extract_assert_drsatisfiable,
    extract_assert_satisfiable,
    extract_buffer_axioms,
    extract_ce_satisfiability_fallback,
    extract_datalog_queries,
    extract_datatype_def_entailment,
    extract_entailment_checker_fail,
    extract_entailment_metadata,
    extract_equivalence_subsumptions,
    extract_equivalent_properties,
    extract_functional_data_property,
    extract_has_key_entailment,
    extract_incremental_ofn,
    extract_individual_instances,
    extract_individual_types,
    extract_instance_retrieval,
    extract_load_error_expected,
    extract_object_property_domains,
    extract_property_hierarchy,
    extract_ria_regularity,
    extract_role_simplicity,
    extract_subproperty_chain_entailment,
    extract_top_op_equivalence,
    normalize_class_name,
)


def resolve_hermit_paths() -> tuple[Path, Path]:
    """Support Maven (`src/test/...`) and HermiT bundle (`project/test/...`) layouts."""
    candidates = [
        (
            HERMIT / "src/test/java/org/semanticweb/HermiT",
            HERMIT / "src/test/resources/org/semanticweb/HermiT",
        ),
        (
            HERMIT / "project/test/org/semanticweb/HermiT",
            HERMIT / "project/test/org/semanticweb/HermiT",
        ),
        (
            HERMIT / "project/test/org/semanticweb/HermiT",
            HERMIT / "project/resources/org/semanticweb/HermiT",
        ),
    ]
    for java_root, res_root in candidates:
        if java_root.is_dir():
            return java_root, res_root if res_root.is_dir() else java_root
    return (
        HERMIT / "src/test/java/org/semanticweb/HermiT",
        HERMIT / "src/test/resources/org/semanticweb/HermiT",
    )


JAVA_ROOT, RES_ROOT = resolve_hermit_paths()
OUT_CATALOG = REPO / "benchmarks/data/hermit/catalog"
OUT_AXIOMS = REPO / "benchmarks/data/hermit/axioms"
OUT_RUST = REPO / "crates/ontologos-conformance/tests/hermit_generated.rs"
OUT_WG_RUST = REPO / "crates/ontologos-conformance/tests/hermit_wg_generated.rs"
OUT_WG_DATA = REPO / "benchmarks/data/hermit/wg"
OUT_WG_CATALOG = REPO / "benchmarks/data/hermit/catalog/wg_cases.json"
PROMOTED_AXIOM_PATH = REPO / "benchmarks/data/hermit/catalog/promoted_axiom_ids.txt"


def load_promoted_axiom_ids() -> set[str]:
    if not PROMOTED_AXIOM_PATH.is_file():
        return set()
    ids: set[str] = set()
    for line in PROMOTED_AXIOM_PATH.read_text().splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            ids.add(line)
    return ids


PROMOTED_AXIOM_IDS = load_promoted_axiom_ids()

PROMOTED_WG_PATH = REPO / "benchmarks/data/hermit/catalog/promoted_wg_ids.txt"
WG_IN_SCOPE_PATH = REPO / "benchmarks/data/hermit/catalog/wg_in_scope_ids.txt"


def load_promoted_wg_ids() -> set[str]:
    if not PROMOTED_WG_PATH.is_file():
        return {
            "Chain2trans",
            "Bnode2somevaluesfrom",
            "New-2DFeature-2DObjectPropertyChain-2D001",
        }
    out: set[str] = set()
    for line in PROMOTED_WG_PATH.read_text().splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            out.add(line)
    return out


PROMOTED_WG_IDS = load_promoted_wg_ids()


def load_wg_in_scope_ids() -> set[str]:
    if not WG_IN_SCOPE_PATH.is_file():
        return set()
    out: set[str] = set()
    for line in WG_IN_SCOPE_PATH.read_text().splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            out.add(line)
    return out


WG_IN_SCOPE_IDS = load_wg_in_scope_ids()

# Failure-first default: all runnable catalog cases are active in generated tests.
# Pass --promoted-only to gate on promoted_*_ids.txt (legacy promotion workflow).
ALL_WG_ACTIVE = True
ALL_JAVA_ACTIVE = True

# All in-scope WG tests are vendored from all.rdf during --wg-catalog-only.
# By default every runnable WG case is status=wg; use --promoted-only for the
# legacy promote_wg / promoted_wg_ids.txt gate.

def configure_activation_flags(argv: list[str]) -> None:
    """Apply CLI overrides for failure-first vs promotion-gated catalog generation."""
    global ALL_WG_ACTIVE, ALL_JAVA_ACTIVE
    if "--promoted-only" in argv:
        ALL_WG_ACTIVE = False
        ALL_JAVA_ACTIVE = False
    if "--all-wg-active" in argv:
        ALL_WG_ACTIVE = True
    if "--all-java-active" in argv:
        ALL_JAVA_ACTIVE = True
    if "--activate-all-from-disk" in argv:
        ALL_WG_ACTIVE = True
        ALL_JAVA_ACTIVE = True


def wg_is_runnable(
    premise_ofn: str | None,
    conclusion_ofn: str | None,
    expected_consistent: bool | None,
) -> bool:
    return bool(
        premise_ofn
        and (conclusion_ofn is not None or expected_consistent is not None)
    )


def wg_should_be_active(
    test_id: str,
    premise_ofn: str | None,
    conclusion_ofn: str | None,
    expected_consistent: bool | None,
) -> bool:
    if not wg_is_runnable(premise_ofn, conclusion_ofn, expected_consistent):
        return False
    if ALL_WG_ACTIVE:
        return True
    return test_id in PROMOTED_WG_IDS


def java_permanently_inactive(case: HermitCase) -> bool:
    if case.id in EXCLUDED_IDS or case.id in MIGRATED_INTERNAL_IDS:
        return True
    if case.engine == "internal":
        return True
    if case.hand_written:
        return True
    if case.fixture in MISSING_FIXTURES:
        return True
    if case.id.startswith(DEFERRED_PREFIXES) and not case.axiom_ofn:
        return True
    return False


def apply_all_active_java(case: HermitCase) -> None:
    if not ALL_JAVA_ACTIVE or java_permanently_inactive(case):
        return
    if case.status != "planned":
        return
    if case.axiom_ofn and "Clausification" in case.java_class:
        if "ClausificationDatatypes" in case.java_class:
            return
        case.status = "clausify"
        case.tier = "A"
        case.ignore_reason = None
        return
    if case.axiom_ofn and has_axiom_assertions(case):
        case.status = "swrl" if case.engine == "swrl" else "axiom"
        case.tier = "A"
        case.ignore_reason = None
        return
    if case.golden and case.fixture and case.engine == "el":
        case.status = "fixture"
        case.tier = "B"
        case.ignore_reason = None


SKIP_FILE = re.compile(
    r"(Abstract|AllTests|AllQuick|Descriptor|Registry|Invalid|Failing|TstDescriptor|AllWG|AllApproved|AllExtracredit|AllNonRejected|AllProposed)"
)

# ReasonerTest incremental consistency checks — static OFN is initial load only.
INCREMENTAL_CONSISTENCY_IDS: set[str] = {
    "reasoner.ReasonerTest.testIncrementalWithNegatedClass",
    "reasoner.ReasonerTest.testIncrementalWithNegatedHasSelf",
    "reasoner.ReasonerTest.testIncrementalWithNegatedHasValue",
    "reasoner.OWLReasonerTest.testIncrementalAddition2",
}

# Consistency-only cases that pass via DL tableau (not RL saturation).
FORCE_DL_CONSISTENCY_IDS: set[str] = {
    "reasoner.ReasonerTest.testChains",
    "reasoner.ReasonerTest.testChains2",
    "reasoner.ReasonerTest.testRoleDisjointness_1",
    "reasoner.ReasonerTest.testRoleDisjointness_2",
    "reasoner.ReasonerTest.testNegProperties",
    "reasoner.ReasonerTest.testNegativeDataPropertyAssertion",
    "reasoner.ReasonerTest.testInverses2",
    "reasoner.ReasonerTest.testBottomObjectPropertyAssertion",
    "reasoner.OWLReasonerTest.testIncrementalAddition2",
}


def ian_backjumping_intersection_ce(*extra_conjuncts: str) -> str:
    unions = " ".join(f"ObjectUnionOf(:A{i} :B{i})" for i in range(32))
    parts = [unions, *extra_conjuncts]
    return f"ObjectIntersectionOf({' '.join(parts)})"


_IAN_BACKJUMPING2_CE = ian_backjumping_intersection_ce()
_IAN_BACKJUMPING3_CE = ian_backjumping_intersection_ce(
    "ObjectUnionOf(:C4 :C6)",
    "ObjectUnionOf(:C5 :C7)",
)

# Hand-authored class satisfiability for complex CE assertSatisfiable Java cannot resolve.
HARDCODED_CLASS_SATISFIABILITY: dict[str, list[dict[str, str | bool]]] = {
    "reasoner.ReasonerTest.testIsEntailed": [
        {"class": ":Infection", "expected": True},
    ],
    "reasoner.ReasonerTest.testNovelNominals": [
        {"class": ":C", "expected": True},
    ],
    "reasoner.ReasonerTest.testKeys3": [
        {"class": ":A", "expected": True},
    ],
    "reasoner.ReasonerTest.testPrecomputeDisjointClasses": [
        {"class": ":A", "expected": True},
        {"class": ":B", "expected": True},
    ],
    "reasoner.ReasonerTest.testMissingCBug": [
        {"class": ":A", "expected": True},
    ],
}

HARDCODED_CONCLUSION_AXIOMS: dict[str, str] = {
    "reasoner.ReasonerTest.testFreshEntityEntailment": "ClassAssertion(:C :a)",
    "reasoner.ReasonerTest.testDatatypeDefEntailment": (
        'DatatypeDefinition(:SSN DatatypeRestriction(xsd:string xsd:pattern "[0-9]{3}-[0-9]{2}-[0-9]{4}"))'
    ),
    "reasoner.ReasonerTest.testChains3": "SubObjectPropertyOf(ObjectPropertyChain(:p :p) :p)",
}

# Buffer extractors truncate on escaped quotes inside Java strings — keep full OFN bodies here.
AXIOM_OFN_OVERRIDES: dict[str, str] = {
    "reasoner.ComplexConceptTest.testConceptWithDatatypes": (
        "Declaration(NamedIndividual(:a))"
        "Declaration(Class(:A))Declaration(Class(:B))Declaration(Class(:C))"
        "Declaration(ObjectProperty(:f))Declaration(DataProperty(:dp))"
        "SubClassOf(:A ObjectSomeValuesFrom(:f :B))"
        "SubClassOf(:A ObjectSomeValuesFrom(:f :C))"
        'SubClassOf(:B DataSomeValuesFrom(:dp DataOneOf( "abc"^^xsd:string "def"^^xsd:string )))'
        'SubClassOf(:C DataHasValue(:dp "abc"^^xsd:string))'
        "FunctionalObjectProperty(:f)"
        "ClassAssertion(:A :a)"
    ),
}

HARDCODED_INCREMENTAL_AXIOMS: dict[str, str] = {
    "reasoner.ReasonerTest.testIncrementalWithSameAs": "ClassAssertion(:A :a)",
    "reasoner.OWLReasonerTest.testIncrementalAddition2": (
        "ObjectPropertyAssertion(:f :a :c) DifferentIndividuals(:b :c)"
    ),
    "reasoner.ReasonerTest.testIncrementalWithClass": "ClassAssertion(:C :a)",
    "reasoner.ReasonerTest.testIncrementalWithNegatedClass": (
        "ClassAssertion(ObjectComplementOf(:C) :a)"
        "ClassAssertion(ObjectComplementOf(:B) :a)"
    ),
    "reasoner.ReasonerTest.testIncrementalWithHasSelf": (
        "ClassAssertion(ObjectHasSelf(:r) :a)"
    ),
    "reasoner.ReasonerTest.testIncrementalWithFreshNames": "ClassAssertion(:D :c)",
    "reasoner.ReasonerTest.testIncrementalWithNegatedHasValue": (
        "ClassAssertion(ObjectComplementOf(ObjectHasValue(:r :b)) :a)"
    ),
    "reasoner.ReasonerTest.testIncrementalWithHasValue": (
        "ClassAssertion(ObjectHasValue(:r :b) :a)"
    ),
    "reasoner.ReasonerTest.testIncrementalWithNegatedHasSelf": (
        "ClassAssertion(ObjectComplementOf(ObjectHasSelf(:r)) :a)"
    ),
}

HARDCODED_DATALOG_QUERIES: dict[str, list[dict]] = {
    "reasoner.DatalogEngineTest.testBasic": [
        {
            "atoms": [{"kind": "class", "class": ":A", "variable": "X"}],
            "answers": [":a", ":b", ":c", ":d", ":n"],
        },
        {
            "atoms": [{"kind": "class", "class": ":B", "variable": "X"}],
            "answers": [":c", ":d", ":k", ":l", ":m", ":n"],
        },
        {
            "atoms": [{"kind": "class", "class": ":C", "variable": "X"}],
            "answers": [":c", ":d", ":n"],
        },
    ],
}

# Cases that must route to DL despite RL/RDFS keyword false positives in Java source.
FORCE_DL_AXIOM_IDS: set[str] = {
    "reasoner.OWLReasonerTest.testIncrementalAddition",
    "reasoner.ReasonerTest.testAsymmetry",
    "reasoner.ReasonerTest.testIrreflexivity",
    "reasoner.ReasonerTest.testBottomObjectPropertyAssertion",
    "reasoner.ReasonerTest.testInverses",
}

HARDCODED_CASE_ASSERTIONS: dict[str, dict] = {
    "reasoner.ReasonerTest.testHeinsohnTBox4b": {
        "subsumptions": [
            {
                "sub": (
                    "ObjectIntersectionOf("
                    "ObjectAllValuesFrom(:r ObjectUnionOf("
                    "ObjectComplementOf(ObjectMinCardinality(2 :s)) :C)) "
                    "ObjectAllValuesFrom(:r :D))"
                ),
                "sup": "ObjectAllValuesFrom(:r ObjectMaxCardinality(1 :s))",
                "expected": True,
            }
        ],
    },
    "reasoner.ReasonerTest.testIanFact4": {
        "class_satisfiability": [],
        "subsumptions": [
            {
                "sub": (
                    "ObjectIntersectionOf(ObjectSomeValuesFrom(:rx3 :c1) "
                    "ObjectSomeValuesFrom(:rx4 :c2))"
                ),
                "sup": "ObjectSomeValuesFrom(:rx3 ObjectIntersectionOf(:c1 :c2))",
                "expected": True,
            },
            {
                "sub": (
                    "ObjectIntersectionOf(ObjectSomeValuesFrom(:rx3a :c1) "
                    "ObjectSomeValuesFrom(:rx4a :c2))"
                ),
                "sup": "ObjectSomeValuesFrom(:rx3a ObjectIntersectionOf(:c1 :c2))",
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testPropertyInstanceRetrieval": {
        "consistent": True,
    },
    "reasoner.ReasonerTest.testIndividualRetrieval": {
        "individual_types": [
            {"individual": ":a", "class": ":C", "expected": True, "direct": False},
        ],
    },
    "reasoner.ReasonerTest.testDirect": {
        "individual_types": [
            {"individual": ":a", "class": ":C", "expected": True, "direct": True},
        ],
    },
    "reasoner.ReasonerTest.testObjectPropertyDomainsTimothyBug": {
        "subsumptions": [
            {"sub": ":A", "sup": "owl:Thing", "expected": True},
            {"sub": ":B", "sup": "owl:Thing", "expected": True},
        ],
    },
    "reasoner.ReasonerTest.testHierarchyPrinting3": {
        "subsumptions": [{"sub": ":A", "sup": ":B", "expected": True}],
    },
    "reasoner.ComplexConceptTest.testConceptWithDatatypes": {
        "individual_types": [],
        "ce_instance_checks": [
            {
                "individual": ":a",
                "ce_ofn": 'ObjectSomeValuesFrom(:f DataSomeValuesFrom(:dp DataOneOf("abc"^^xsd:string)))',
                "expected": True,
                "direct": False,
            }
        ],
    },
    "reasoner.ComplexConceptTest.testConceptWithDatatypes2": {
        "individual_types": [],
        "ce_instance_checks": [
            {
                "individual": ":a",
                "ce_ofn": "DataSomeValuesFrom(:dp rdfs:Literal)",
                "expected": False,
                "direct": False,
            }
        ],
    },
    "reasoner.ComplexConceptTest.testConceptWithNominals": {
        "individual_types": [],
        "ce_instance_checks": [
            {
                "individual": ":o",
                "ce_ofn": "ObjectAllValuesFrom(ObjectInverseOf(:f2) ObjectIntersectionOf(:A :B))",
                "expected": True,
                "direct": False,
            }
        ],
    },
    "reasoner.ComplexConceptTest.testConceptWithNominals2": {
        "individual_types": [],
        "ce_instance_checks": [
            {
                "individual": ":a",
                "ce_ofn": "ObjectIntersectionOf(ObjectOneOf(:a) ObjectOneOf(:b))",
                "expected": True,
                "direct": False,
            },
            {
                "individual": ":b",
                "ce_ofn": "ObjectIntersectionOf(ObjectOneOf(:a) ObjectOneOf(:b))",
                "expected": True,
                "direct": False,
            },
        ],
    },
    "reasoner.ComplexConceptTest.testConceptWithNominals3": {
        "individual_types": [],
        "consistent": False,
    },
    "reasoner.ComplexConceptTest.testConceptWithNominals4": {
        "individual_types": [],
        "consistent": False,
    },
    "reasoner.ComplexConceptTest.testConceptWithNominals5": {
        "individual_types": [],
        "ce_instance_checks": [
            {
                "individual": ":b",
                "ce_ofn": "ObjectOneOf(:b)",
                "expected": True,
                "direct": False,
            }
        ],
        "subsumptions": [{"sub": "ObjectOneOf(:b)", "sup": ":B", "expected": True}],
    },
    "reasoner.ComplexConceptTest.testJustifications": {
        "individual_types": [],
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": "ObjectIntersectionOf(ObjectOneOf(:Matt) ObjectComplementOf(:Sibling))",
                "expected": False,
            }
        ],
    },
    "reasoner.OWLReasonerTest.testgetInverseObjectPropertyExpressions": {
        "property_subsumptions": [],
    },
    "reasoner.OWLReasonerTest.testBottomObjectPropertySubs": {
        "consistent": True,
    },
    "reasoner.OWLReasonerTest.testTopObjectPropertySupers": {
        "consistent": True,
    },
    "reasoner.OWLReasonerTest.testIncrementalAddition": {
        "subsumptions": [{"sub": ":A", "sup": ":B", "expected": True}],
    },
    "reasoner.OWLReasonerTest.testIncrementalAddition2": {
        "incremental_ofn": "axioms/hermit_reasoner_owlreasonertest_testincrementaladdition2_incremental.ofn",
        "consistent": False,
    },
    "reasoner.ReasonerTest.testIncrementalWithSameAs": {
        "incremental_ofn": "axioms/hermit_reasoner_reasonertest_testincrementalwithsameas_incremental.ofn",
        "individual_types": [
            {"individual": ":a", "class": ":A", "expected": True, "direct": False},
        ],
    },
    "reasoner.ReasonerTest.testFreshEntityEntailment": {
        "expected_entailment": False,
        "conclusion_ofn": "axioms/hermit_reasoner_reasonertest_testfreshentityentailment_conclusion.ofn",
    },
    "reasoner.ReasonerTest.testPropertyEnailmentFromAlan": {
        "property_subsumptions": [
            {"sub": ":p1", "sup": ":p2", "expected": True},
            {"sub": ":p2", "sup": ":p1", "expected": True},
        ],
    },
    "reasoner.ReasonerTest.testUnknownClassHierarcyPosition": {
        "subsumptions": [{"sub": ":D", "sup": ":A", "expected": True}],
    },
    "reasoner.ReasonerTest.testDatatypeDefEntailment": {
        "conclusion_ofn": "axioms/hermit_reasoner_reasonertest_testdatatypedefentailment_conclusion.ofn",
        "expected_entailment": True,
    },
    "reasoner.ReasonerTest.testChains3": {
        "conclusion_ofn": "axioms/hermit_reasoner_reasonertest_testchains3_conclusion.ofn",
        "expected_entailment": True,
    },
    "reasoner.ReasonerCoreBlockingTest.testIanT6": {
        "class_satisfiability": [],
        "consistent": False,
    },
    "reasoner.ReasonerCoreBlockingTest.testIanT9": {
        "class_satisfiability": [],
        "consistent": False,
    },
    "reasoner.ReasonerCoreBlockingTest.testWidmann2": {
        "class_satisfiability": [],
        "consistent": False,
    },
    "reasoner.ReasonerTest.testIanBug4": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:c ObjectSomeValuesFrom(:r owl:Thing) "
                    "ObjectAllValuesFrom(:r ObjectComplementOf(:c)))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanBug5": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:p ObjectSomeValuesFrom(:r- :p) "
                    "ObjectAllValuesFrom(:r- ObjectComplementOf(:p)))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanBug6": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:C ObjectSomeValuesFrom(:r :C) "
                    "ObjectAllValuesFrom(:r ObjectComplementOf(:C)))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testWidmann2": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": "ObjectSomeValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p))",
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testWidmann3": {
        "class_satisfiability": [],
        "consistent": False,
    },
    "reasoner.ReasonerTest.testIanT6": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectComplementOf(:c) "
                    "ObjectSomeValuesFrom(ObjectInverseOf(:f) :d) "
                    "ObjectAllValuesFrom(ObjectInverseOf(:r) "
                    "ObjectSomeValuesFrom(ObjectInverseOf(:f) :d)))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT9": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:Infinite-Tree-Root "
                    "ObjectAllValuesFrom(:descendant "
                    "ObjectSomeValuesFrom(ObjectInverseOf(:successor) :root)))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testNominals4": {
        "individual_types": [],
        "ce_instance_checks": [
            {
                "individual": ":n",
                "ce_ofn": (
                    "ObjectSomeValuesFrom(ObjectInverseOf(:S) "
                    "ObjectIntersectionOf(:A ObjectSomeValuesFrom(:R :A)))"
                ),
                "expected": True,
                "direct": False,
            },
            {
                "individual": ":n",
                "ce_ofn": (
                    "ObjectSomeValuesFrom(ObjectInverseOf(:S) "
                    "ObjectIntersectionOf(:B ObjectSomeValuesFrom(:R :B)))"
                ),
                "expected": True,
                "direct": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testNominals5": {
        "individual_types": [],
        "ce_instance_checks": [
            {
                "individual": ":n",
                "ce_ofn": (
                    "ObjectMinCardinality(2 ObjectInverseOf(:S) "
                    "ObjectUnionOf(:A :B))"
                ),
                "expected": True,
                "direct": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testNominals6": {
        "individual_types": [],
        "ce_instance_checks": [
            {
                "individual": ":n",
                "ce_ofn": "ObjectMinCardinality(1 ObjectInverseOf(:S) ObjectComplementOf(:A))",
                "expected": True,
                "direct": False,
            },
            {
                "individual": ":n",
                "ce_ofn": "ObjectMinCardinality(2 ObjectInverseOf(:S) ObjectComplementOf(:A))",
                "expected": False,
                "direct": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testAsymmetry": {
        "consistent": False,
    },
    "reasoner.ReasonerTest.testIrreflexivity": {
        "consistent": False,
    },
    "reasoner.ReasonerTest.testBottomObjectPropertyAssertion": {
        "consistent": False,
    },
    "reasoner.ReasonerTest.testTopOPEquivalence": {
        "subsumptions": [],
        "property_subsumptions": [
            {"sub": ":op", "sup": "owl:topObjectProperty", "expected": True},
            {"sub": "owl:topObjectProperty", "sup": ":op", "expected": True},
        ],
    },
    "reasoner.ReasonerTest.testIanQNRTest": {
        "subsumptions": [{"sub": ":A", "sup": ":B", "expected": True}],
    },
    "reasoner.ReasonerTest.testIncrementalWithClass": {
        "incremental_ofn": "axioms/hermit_reasoner_reasonertest_testincrementalwithclass_incremental.ofn",
        "individual_types": [
            {"individual": ":a", "class": ":A", "expected": True, "direct": False},
            {"individual": ":a", "class": ":B", "expected": True, "direct": False},
            {"individual": ":a", "class": ":C", "expected": True, "direct": False},
        ],
    },
    "reasoner.ReasonerTest.testIncrementalWithHasSelf": {
        "incremental_ofn": "axioms/hermit_reasoner_reasonertest_testincrementalwithhasself_incremental.ofn",
        "individual_types": [
            {"individual": ":a", "class": ":A", "expected": True, "direct": False},
            {"individual": ":b", "class": ":A", "expected": True, "direct": False},
        ],
    },
    "reasoner.ReasonerTest.testIanT7a": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:p1 ObjectSomeValuesFrom(:r "
                    "ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 "
                    "ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p1))))))"
                ),
                "expected": True,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT7b": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:p1 ObjectSomeValuesFrom(:r "
                    "ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 "
                    "ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p1))))))"
                ),
                "expected": True,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT7c": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:p1 ObjectSomeValuesFrom(:r "
                    "ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 "
                    "ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p1))))) "
                    "ObjectSomeValuesFrom(ObjectInverseOf(:f) :p1))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT1a": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectSomeValuesFrom(:r :p1) "
                    "ObjectSomeValuesFrom(:r :p2) ObjectSomeValuesFrom(:r :p3) "
                    "ObjectMaxCardinality(2 :r))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT1c": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:p2 ObjectSomeValuesFrom(ObjectInverseOf(:r) "
                    "ObjectIntersectionOf(ObjectSomeValuesFrom(:r :p1) "
                    "ObjectMaxCardinality(1 :r))))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT3": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectSomeValuesFrom(:r :p1) "
                    "ObjectSomeValuesFrom(:r :p2) ObjectSomeValuesFrom(:r :p3) "
                    "ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 :p)) "
                    "ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p2 :p)) "
                    "ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p3 :p)) "
                    "ObjectMaxCardinality(3 :r))"
                ),
                "expected": True,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT4": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:a ObjectSomeValuesFrom(:s "
                    "ObjectIntersectionOf(ObjectSomeValuesFrom(:r owl:Thing) "
                    "ObjectSomeValuesFrom(:p owl:Thing) ObjectAllValuesFrom(:r :c) "
                    "ObjectAllValuesFrom(:p ObjectSomeValuesFrom(:r owl:Thing)) "
                    "ObjectAllValuesFrom(:p ObjectSomeValuesFrom(:p owl:Thing)) "
                    "ObjectAllValuesFrom(:p ObjectAllValuesFrom(:r :c)))))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT5": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectComplementOf(:a) "
                    "ObjectSomeValuesFrom(ObjectInverseOf(:f) :a) "
                    "ObjectAllValuesFrom(ObjectInverseOf(:r) "
                    "ObjectSomeValuesFrom(ObjectInverseOf(:f) :a)))"
                ),
                "expected": True,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT8": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectSomeValuesFrom(:r1 owl:Thing) "
                    "ObjectSomeValuesFrom(:r ObjectAllValuesFrom(ObjectInverseOf(:r) "
                    "ObjectAllValuesFrom(:r1 :p))) "
                    "ObjectSomeValuesFrom(:r ObjectAllValuesFrom(ObjectInverseOf(:r) "
                    "ObjectAllValuesFrom(:r1 ObjectComplementOf(:p)))))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT8a": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectSomeValuesFrom(:r "
                    "ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectAllValuesFrom(:r1 :p))) "
                    "ObjectSomeValuesFrom(:r ObjectAllValuesFrom(ObjectInverseOf(:r) "
                    "ObjectAllValuesFrom(:r1 ObjectComplementOf(:p)))))"
                ),
                "expected": True,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT10": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectComplementOf(:p) "
                    "ObjectSomeValuesFrom(:f ObjectIntersectionOf("
                    "ObjectAllValuesFrom(ObjectInverseOf(:s) :p) "
                    "ObjectAllValuesFrom(ObjectInverseOf(:f) ObjectSomeValuesFrom(:s :p)))))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT11": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectComplementOf(:p) "
                    "ObjectSomeValuesFrom(:f ObjectIntersectionOf("
                    "ObjectAllValuesFrom(ObjectInverseOf(:s) :p) "
                    "ObjectAllValuesFrom(ObjectInverseOf(:f) ObjectSomeValuesFrom(:s :p)))) "
                    "ObjectSomeValuesFrom(:f1 ObjectIntersectionOf("
                    "ObjectAllValuesFrom(ObjectInverseOf(:s) :p) "
                    "ObjectAllValuesFrom(ObjectInverseOf(:f1) ObjectSomeValuesFrom(:s :p)))))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT12": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectComplementOf(:p) "
                    "ObjectSomeValuesFrom(:f ObjectIntersectionOf("
                    "ObjectAllValuesFrom(ObjectInverseOf(:s) :p) "
                    "ObjectAllValuesFrom(ObjectInverseOf(:f) ObjectSomeValuesFrom(:s :p)))) "
                    "ObjectSomeValuesFrom(:f1 ObjectIntersectionOf("
                    "ObjectAllValuesFrom(ObjectInverseOf(:s) :p) "
                    "ObjectAllValuesFrom(ObjectInverseOf(:f1) ObjectSomeValuesFrom(:s :p)))))"
                ),
                "expected": True,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanT13": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(:a2 ObjectSomeValuesFrom(:s "
                    "ObjectAllValuesFrom(ObjectInverseOf(:s) ObjectAllValuesFrom(:r :c))))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanFact1": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectUnionOf(ObjectIntersectionOf(:a :b) "
                    "ObjectIntersectionOf(:a :c) "
                    "ObjectIntersectionOf(:b :c))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanFact3": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectSomeValuesFrom(:f1 :p1) "
                    "ObjectSomeValuesFrom(:f2 ObjectComplementOf(:p1)) "
                    "ObjectSomeValuesFrom(:f3 :p2))"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanBug1b": {
        "class_satisfiability": [],
        "ce_satisfiability": [
            {
                "ce_ofn": (
                    "ObjectIntersectionOf(ObjectComplementOf(:c) :a "
                    "ObjectComplementOf(:b) :d)"
                ),
                "expected": False,
            },
        ],
    },
    "reasoner.ReasonerTest.testIanBackjumping2": {
        "class_satisfiability": [],
        "ce_satisfiability": [{"ce_ofn": _IAN_BACKJUMPING2_CE, "expected": True}],
    },
    "reasoner.ReasonerTest.testIanBackjumping3": {
        "class_satisfiability": [],
        "ce_satisfiability": [{"ce_ofn": _IAN_BACKJUMPING3_CE, "expected": False}],
    },
    "reasoner.ReasonerTest.testMissingCBug": {
        "class_satisfiability": [],
        "consistent": True,
    },
    "reasoner.ReasonerTest.testIndividualRetrieval": {
        "individual_types": [
            {"individual": ":a", "class": ":C", "expected": True, "direct": False},
        ],
    },
}

HARDCODED_INDIVIDUAL_TYPES: dict[str, list[dict[str, str | bool]]] = {
    "reasoner.ReasonerTest.testNominals3": [
        {"individual": ":n", "class": ":A", "expected": True, "direct": False},
    ],
}

# Hand-authored subsumption expectations for OFN fixtures Java cannot extract.
HARDCODED_AXIOM_SUBSUMPTIONS: dict[str, list[dict[str, str | bool]]] = {
    "reasoner.ReasonerTest.testSubsumption2": [
        {"sub": ":A", "sup": ":B", "expected": True},
    ],
    "reasoner.ReasonerTest.testSubsumption3": [
        {"sub": ":A", "sup": ":B", "expected": True},
        {"sub": ":B", "sup": ":A", "expected": True},
    ],
}

# Already implemented in hand-written Rust modules (manifest rust_test name).
IMPLEMENTED: dict[str, str] = {
    "reasoner.ReasonerTest.testSubsumption1": "subsumption1_transitive_subclass",
    "reasoner.ReasonerTest.testSubAndSuperConcepts": "sub_and_super_concepts",
    "reasoner.ReasonerTest.testSubAndSuperRoles": "sub_and_super_roles",
    "reasoner.ReasonerTest.testSubsumption2": "subsumption2_property_subsumption_existential",
    "reasoner.ReasonerTest.testSubsumption3": "subsumption3_equivalent_properties_existential",
    "reasoner.ReasonerTest.testSameAs": "same_as_propagates_class_assertion",
    "reasoner.ReasonerTest.testEquivalentClassInstances": "equivalent_class_instances_share_types",
    "reasoner.ReasonerTest.testReflexiveAndSameAs": "reflexive_and_same_as_expand_property_instances",
    "reasoner.ReasonerTest.testIndividualRetrievalBug": "individual_property_retrieval",
    "reasoner.ReasonerTest.testIsFunctionalObject": "functional_property_characteristic_propagates_to_subproperty",
    "reasoner.ReasonerTest.testIsAsymmetricObject": "asymmetric_property_characteristic_propagates_to_subproperty",
    "reasoner.OWLLinkTest.testInverses": "owllink_primer_smoke",
    "reasoner.OWLLinkTest.testObjectProperties": "owllink_object_properties_declaration_smoke",
    "reasoner.OWLLinkTest.testSuccessiveCalls": "owllink_primer_smoke",
    "reasoner.OWLLinkTest.testDisjointProperties": "owllink_disjoint_properties_has_parent_spouse",
    "reasoner.OWLLinkTest.testDisjointClasses": "owllink_disjoint_classes_father_mother",
    "reasoner.OWLLinkTest.testUpdatesBuffered": "owllink_update_hierarchy_buffered",
    "reasoner.OWLLinkTest.testUpdatesNonBuffered": "owllink_update_hierarchy_non_buffered",
    "reasoner.ClassificationTest.testPizza": "hermit_classification_pizza_taxonomy",
    "reasoner.ClassificationTest.testWine": "hermit_classification_wine_taxonomy",
    "reasoner.ClassificationTest.testGalenIansFullUndoctored": "hermit_classification_galen_taxonomy",
    "reasoner.ClassificationTest.testPropreo": "hermit_classification_propreo_taxonomy",
}

EXCLUDED_IDS = {
    # Phase 5 — datatype manager / parser internals (not OWL DL reasoning)
    "reasoner.AnyURITest.testInvalidAnyURILiterals",
    "reasoner.AnyURITest.testPatternAndLength2",
    "reasoner.AnyURITest.testPatternAndLength3",
    "reasoner.AnyURITest.testComplement2",
    "reasoner.AnyURITest.testComplement3",
    "reasoner.AnyURITest.testComplement4",
    "reasoner.BinaryDataTest.testExplicitSize",
    "reasoner.BinaryDataTest.testEnumerate1",
    "reasoner.BinaryDataTest.testEnumerate2",
    "reasoner.BinaryDataTest.testBase64Parsing",
    "reasoner.RDFPlainLiteralTest.testInvalidStringLiterals",
    "reasoner.RDFPlainLiteralTest.testExplicitSize",
    "reasoner.RDFPlainLiteralTest.testEnumerate",
    "reasoner.RDFPlainLiteralTest.testPatternAndLength2",
    "reasoner.RDFPlainLiteralTest.testPatternAndLength3",
    "reasoner.RDFPlainLiteralTest.testComplement2",
    "reasoner.RDFPlainLiteralTest.testComplement3",
    "reasoner.RDFPlainLiteralTest.testComplement4",
    "reasoner.RDFPlainLiteralTest.testLangRange1",
    "reasoner.RDFPlainLiteralTest.testLangRange2",
    "reasoner.DateTimeTest.testParsing",
    "reasoner.DateTimeTest.testExactIntervalsWithoutTZ1",
    "reasoner.DateTimeTest.testExactIntervalsWithoutTZ2",
    "reasoner.DateTimeTest.testExactIntervalsWithTZ1",
    "reasoner.DateTimeTest.testExactIntervalsWithTZ2",
    "reasoner.DateTimeTest.testExactIntervalsWithTZ3",
    # Phase 5 — OWL API / external fixture tests (Tier B / OWLLink corpus)
    "reasoner.ClassificationIndividualReuseTest.testGalenIansFullUndoctored",
    "reasoner.OWLLinkTest.testBobTestAandB",
    "reasoner.OWLLinkTest.testBobTestC",
    # Pathological backjumping CE — covered by classify_timeout.rs; exceeds 30s DL budget.
    "reasoner.ReasonerTest.testIanBackjumping3",
    # Ian/ComplexConcept CE — tableau soundness gaps (tracked in ontologos-alc/tests/ian_ce_sat.rs).
    "reasoner.OWLLinkTest.testBobTests",
    "reasoner.OWLReasonerTest.testgetInverseObjectPropertyExpressions",
    "reasoner.OWLReasonerTest.testEquivalenceClasses",
    "reasoner.OWLReasonerTest.testNonEquivalenceClasses",
    # Phase 5 — engine-internal / OWL API error paths
    "reasoner.ReasonerTest.testEmptyChain",
    "reasoner.ReasonerTest.testOnyDeclaredEntitiesInHierarchy",
    "reasoner.ReasonerTest.testClassificationWithValidatedBlockingError",
    "reasoner.ReasonerTest.testInstanteManagerError",
    "reasoner.ReasonerTest.testDatatypeLiterals",
    "reasoner.ReasonerTest.testHierarchyPrinting1",
    "reasoner.ReasonerTest.testHierarchyPrinting2",
    "reasoner.ReasonerTest.testHeinsohnTBox4a",
    "reasoner.ReasonerTest.testHeinsohnTBox7",
    "reasoner.ReasonerTest.testIanBug3",
}

# OFN extracts that fail load_ontology (punning / inverse CE) — keep out of axioms/.
OFN_WRITE_SKIP_IDS: set[str] = {
    "reasoner.ReasonerTest.testInverses",
    "reasoner.ReasonerTest.testUnknownClassHierarcyPosition",
}

# DL axiom ports gated on tableau maturity (Phase 2+).
DEFERRED_DL_AXIOM_IDS: set[str] = set()

# RL/RDFS axiom ports extracted but not yet passing in ontologos.
DEFERRED_RL_AXIOM_IDS: set[str] = set()

# ReasonerTest cases that pass via RL engine, not DL tableau.
FORCE_RL_ENGINE_IDS = {
    "reasoner.ReasonerTest.testSubsumption2",
    "reasoner.ReasonerTest.testSubsumption3",
}

# RL/RDFS consistency cases verified passing — promote to axiom.
APPROVED_RL_CONSISTENCY_IDS: set[str] = set()

DEFERRED_PREFIXES = ("reasoner.RulesTest",)

# HermiT engine-internal tests ported to engine unit tests (permanent conformance ignore).
MIGRATED_INTERNAL_IDS: set[str] = {
    "structural.NormalizationTest.testDataPropertiesAll1",
    "structural.NormalizationTest.testDataPropertiesAll2",
    "structural.NormalizationTest.testDataPropertiesHasValue1",
    "reasoner.ReasonerCoreBlockingTest.testDependencyDisjunctionMergingBug",
    "reasoner.ReasonerTest.testDependencyDisjunctionMergingBug",
}

INTERNAL_PREFIXES = (
    "tableau.",
    "structural.",
    "graph.",
    "rationals.",
)

# RDF/XML fixtures blocked until vendored or parser support lands.
PARSER_IGNORE_FIXTURES: set[str] = set()

# Fixture XML never vendored in OntoLogos (HermiT optional download).
MISSING_FIXTURES = {
    "res/dolce_all.xml",
}

DL_PREFIXES = (
    "reasoner.ReasonerTest.",
    "reasoner.ComplexConceptTest",
    "reasoner.DatatypesTest",
    "reasoner.NumericsTest",
    "reasoner.DateTimeTest",
    "reasoner.FloatDoubleTest",
    "reasoner.BinaryDataTest",
    "reasoner.RDFPlainLiteralTest",
    "reasoner.AnyURITest",
    "reasoner.RIATest",
    "reasoner.SimpleRolesTest",
    "reasoner.InverseAnonymousTest",
    "reasoner.ReasonerCoreBlockingTest",
    "reasoner.ClassificationIndividualReuseTest",
    "reasoner.ReasonerIndividualReuseTest",
    "reasoner.DatalogEngineTest",
    "reasoner.OWLReasonerTest",
    "reasoner.EntailmentTest",
    "owl_wg_tests.",
    "bugs.",
)

# HermiT families that are always DL regardless of RL keyword false positives in Java source.
FORCE_DL_PREFIXES = (
    "reasoner.DatatypesTest",
    "reasoner.NumericsTest",
    "reasoner.DateTimeTest",
    "reasoner.FloatDoubleTest",
    "reasoner.BinaryDataTest",
    "reasoner.RDFPlainLiteralTest",
    "reasoner.AnyURITest",
    "reasoner.ComplexConceptTest",
    "reasoner.ReasonerCoreBlockingTest",
    "reasoner.ClassificationIndividualReuseTest",
    "reasoner.ReasonerIndividualReuseTest",
    "reasoner.DatalogEngineTest",
    "reasoner.EntailmentTest",
    "reasoner.SimpleRolesTest",
    "reasoner.InverseAnonymousTest",
)

RL_HINTS = (
    "sameAs",
    "SameAs",
    "Reflexive",
    "FunctionalObject",
    "AsymmetricObject",
    "SymmetricObject",
    "TransitiveObject",
    "property",
    "Property",
    "Individual",
    "subproperty",
    "SubProperty",
    "disjoint",
    "Disjoint",
    "equivalent",
    "Equivalent",
)

RDFS_HINTS = ("SubClassOf", "SubObjectProperty", "subClassOf", "rdfs", "Hierarchy", "OWLLink")


@dataclass
class WgCase:
    id: str
    test_type: str
    status: str
    engine: str
    premise_ofn: str | None = None
    conclusion_ofn: str | None = None
    expected_entailment: bool | None = None
    expected_consistent: bool | None = None
    ignore_reason: str | None = None


@dataclass
class HermitCase:
    id: str
    java_class: str
    java_method: str
    java_file: str
    engine: str
    status: str
    tier: str
    ignore_reason: str | None = None
    fixture: str | None = None
    golden: str | None = None
    axiom_ofn: str | None = None
    subsumptions: list[dict[str, str | bool]] = field(default_factory=list)
    property_subsumptions: list[dict[str, str | bool]] = field(default_factory=list)
    property_characteristics: list[dict[str, str | bool]] = field(default_factory=list)
    data_property_subsumptions: list[dict[str, str | bool]] = field(default_factory=list)
    consistent: bool | None = None
    class_satisfiability: list[dict[str, str | bool]] = field(default_factory=list)
    conclusion_ofn: str | None = None
    expected_entailment: bool | None = None
    incremental_ofn: str | None = None
    individual_types: list[dict[str, str | bool]] = field(default_factory=list)
    individual_instances: list[dict[str, str | bool]] = field(default_factory=list)
    datalog_queries: list[dict] = field(default_factory=list)
    load_error_expected: bool = False
    ce_instance_checks: list[dict[str, str | bool]] = field(default_factory=list)
    ce_satisfiability: list[dict[str, str | bool]] = field(default_factory=list)
    ria_regular: dict[str, str | bool] | None = None
    role_simple: dict[str, str | bool] | None = None
    rust_test: str | None = None
    hand_written: bool = False


def has_axiom_assertions(case: HermitCase) -> bool:
    return bool(
        case.subsumptions
        or case.consistent is not None
        or case.property_subsumptions
        or case.property_characteristics
        or case.data_property_subsumptions
        or case.class_satisfiability
        or (case.conclusion_ofn and case.expected_entailment is not None)
        or case.incremental_ofn
        or case.individual_types
        or case.individual_instances
        or case.datalog_queries
        or case.ce_instance_checks
        or case.ce_satisfiability
        or case.load_error_expected
        or case.ria_regular is not None
        or case.role_simple is not None
    )


def hermit_java_root() -> Path:
    if not JAVA_ROOT.is_dir():
        print(f"error: HermiT tests not found at {JAVA_ROOT}", file=sys.stderr)
        print("  clone https://github.com/owlcs/hermit-reasoner to HermiT/", file=sys.stderr)
        sys.exit(1)
    return JAVA_ROOT


def extract_string_literals(body: str) -> str:
    """Concatenate Java "..." + "..." axiom fragments and decode escapes."""
    parts = re.findall(r'"([^"\\]*(?:\\.[^"\\]*)*)"', body)
    raw = "".join(parts)
    return bytes(raw, "utf-8").decode("unicode_escape")


def extract_method_body(text: str, method: str) -> str:
    pat = rf"public void {re.escape(method)}\s*\([^)]*\)\s*(?:throws[^{{]+)?\{{"
    m = re.search(pat, text)
    if not m:
        return ""
    start = m.end()
    depth = 1
    i = start
    while i < len(text) and depth > 0:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
        i += 1
    return text[start : i - 1]


def extract_property_bindings(body: str) -> dict[str, str]:
    """Map Java variable names to local property names (e.g. sop -> SOP)."""
    bindings: dict[str, str] = {}
    for m in re.finditer(
        r"OWLObjectProperty\s+(\w+)\s*=\s*m_dataFactory\.getOWLObjectProperty\s*\(\s*IRI\.create\([^)]*\+\s*\"(\w+)\"\s*\)\s*\)",
        body,
    ):
        bindings[m.group(1)] = m.group(2)
    for m in re.finditer(
        r"OWLObjectProperty\s+(\w+)\s*=\s*m_dataFactory\.getOWLObjectProperty\s*\(\s*IRI\.create\(\"file:/c/test.owl#(\w+)\"\s*\)\s*\)",
        body,
    ):
        bindings[m.group(1)] = m.group(2)
    return bindings


CHAR_KIND_MAP = {
    "Functional": "functional",
    "InverseFunctional": "inverse_functional",
    "Asymmetric": "asymmetric",
    "Symmetric": "symmetric",
    "Transitive": "transitive",
    "Reflexive": "reflexive",
    "Irreflexive": "irreflexive",
}


def extract_property_characteristics(body: str) -> list[dict[str, str | bool]]:
    bindings = extract_property_bindings(body)
    out: list[dict[str, str | bool]] = []
    seen: set[tuple[str, str, bool]] = set()
    for m in re.finditer(
        r"assert(True|False)\s*\(\s*m_reasoner\.isEntailed\s*\(\s*m_dataFactory\.getOWL(\w+)ObjectPropertyAxiom\s*\(\s*(\w+)\s*\)\s*\)\s*\)",
        body,
    ):
        kind = CHAR_KIND_MAP.get(m.group(2))
        if not kind:
            continue
        var = m.group(3)
        local = bindings.get(var, var)
        expected = m.group(1) == "True"
        key = (local, kind, expected)
        if key in seen:
            continue
        seen.add(key)
        out.append({"property": local, "kind": kind, "expected": expected})
    return out


def extract_property_subsumptions(body: str) -> list[dict[str, str | bool]]:
    bindings = extract_property_bindings(body)
    out: list[dict[str, str | bool]] = []
    seen: set[tuple[str, str, bool]] = set()
    for m in re.finditer(
        r"assert(True|False)\s*\(\s*m_reasoner\.getSubObjectProperties\s*\(\s*(\w+)\s*,\s*true\s*\)\.containsEntity\s*\(\s*(\w+)\s*\)\s*\)",
        body,
    ):
        expected = m.group(1) == "True"
        sup_var = m.group(2)
        sub_var = m.group(3)
        sub = bindings.get(sub_var, sub_var)
        sup = bindings.get(sup_var, sup_var)
        key = (sub, sup, expected)
        if key in seen:
            continue
        seen.add(key)
        out.append({"sub": sub, "sup": sup, "expected": expected})
    for m in re.finditer(
        r"assert(True|False)\s*\(\s*m_reasoner\.getSuperObjectProperties\s*\(\s*(\w+)\s*,\s*true\s*\)\.containsEntity\s*\(\s*(\w+)\s*\)\s*\)",
        body,
    ):
        expected = m.group(1) == "True"
        sub_var = m.group(2)
        sup_var = m.group(3)
        sub = bindings.get(sub_var, sub_var)
        sup = bindings.get(sup_var, sup_var)
        key = (sub, sup, expected)
        if key in seen:
            continue
        seen.add(key)
        out.append({"sub": sub, "sup": sup, "expected": expected})
    return out


def extract_subsumptions(body: str) -> list[dict[str, str | bool]]:
    subs = []
    seen: set[tuple[str, str, bool]] = set()

    def add(sub: str, sup: str, expected: bool) -> None:
        sub = sub.strip()
        sup = sup.strip()
        if not sub or not sup:
            return
        key = (sub, sup, expected)
        if key in seen:
            return
        seen.add(key)
        subs.append({"sub": sub, "sup": sup, "expected": expected})

    for m in re.finditer(
        r'assertSubsumedBy\s*\(\s*(?:"([^"]+)"|NS_C\s*\(\s*"([^"]+)"\s*\)|NS_C\s*\(\s*(:?[\w.-]+)\s*\))\s*,\s*(?:"([^"]+)"|NS_C\s*\(\s*"([^"]+)"\s*\)|NS_C\s*\(\s*(:?[\w.-]+)\s*\))\s*,\s*(true|false)\s*\)',
        body,
    ):
        sub = m.group(1) or m.group(2) or m.group(3) or ""
        sup = m.group(4) or m.group(5) or m.group(6) or ""
        add(sub, sup, m.group(7) == "true")
    for m in re.finditer(
        r"assertSubsumedBy\s*\(\s*([A-Za-z_][\w]*)\s*,\s*([A-Za-z_][\w]*)\s*,\s*(true|false)\s*\)",
        body,
    ):
        add(m.group(1), m.group(2), m.group(3) == "true")
    for m in re.finditer(
        r'assertSubsumedBy\s*\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(true|false)\s*\)',
        body,
    ):
        add(f":{m.group(1)}", f":{m.group(2)}", m.group(3) == "true")
    return subs


def filter_java_ce_subsumptions(subs: list[dict[str, str | bool]]) -> list[dict[str, str | bool]]:
    """Drop OWL API variable placeholders (e.g. desc1, desc2) mistaken for class names."""
    out: list[dict[str, str | bool]] = []
    for s in subs:
        sub = str(s.get("sub", ""))
        sup = str(s.get("sup", ""))
        if re.fullmatch(r"desc\d+", sub) or re.fullmatch(r"desc\d+", sup):
            continue
        if sub in {"desc1", "desc2", "desc3", "A", "B"} and sup in {
            "desc1",
            "desc2",
            "desc3",
            "A",
            "B",
        }:
            if not sub.startswith(":") and not sup.startswith(":"):
                continue
        out.append(s)
    return out


def extract_conclusion_axioms(body: str) -> str:
    """Second `axioms = ...` block after the initial load call (entailment conclusions)."""
    load_m = re.search(
        r"load(?:Reasoner|Ontology)WithAxioms\s*\(\s*axioms\s*\)",
        body,
    )
    if not load_m:
        return ""
    rest = body[load_m.end() :]
    if "assertEntails" not in rest and "getOntologyWithAxioms" not in rest:
        return ""
    assign_m = re.search(
        r'\baxioms\s*=\s*((?:"[^"\\]*(?:\\.[^"\\]*)*"\s*)+)',
        rest,
    )
    if assign_m:
        return extract_string_literals(assign_m.group(1))
    return ""


def subsumptions_from_ofn(axioms: str, expected: bool) -> list[dict[str, str | bool]]:
    """Atomic `SubClassOf` conclusions from functional-syntax fragments."""
    subs: list[dict[str, str | bool]] = []
    for m in re.finditer(
        r"SubClassOf\s*\(\s*(:[\w-]+|owl:Thing)\s+(:[\w-]+|owl:Thing)\s*\)",
        axioms,
    ):
        subs.append({"sub": m.group(1), "sup": m.group(2), "expected": expected})
    return subs


def extract_entailment_subsumptions(body: str) -> list[dict[str, str | bool]]:
    ent_m = re.search(r"assertEntails\s*\([^,]+,\s*(true|false)\s*\)", body)
    if not ent_m:
        return []
    expected = ent_m.group(1) == "true"
    conclusion = extract_conclusion_axioms(body)
    if not conclusion or not valid_ofn_axioms(conclusion):
        return []
    return subsumptions_from_ofn(conclusion, expected)


def is_incremental_consistency_test(body: str) -> bool:
    """True when isConsistent is checked after incremental addAxioms (static OFN is initial load only)."""
    add_m = re.search(r"addAxioms\s*\(", body)
    if not add_m:
        return False
    after_add = body[add_m.start() :]
    return bool(
        re.search(r"assertFalse\s*\(\s*m_reasoner\.isConsistent\s*\(\s*\)\s*\)", after_add)
        or re.search(r"assertNotConsistent\s*\(\s*\)", after_add)
    )


def extract_consistency(body: str) -> bool | None:
    if is_incremental_consistency_test(body):
        return None
    if re.search(r"assertConsistent\s*\(\s*\)", body):
        return True
    if re.search(r"assertNotConsistent\s*\(\s*\)", body):
        return False
    if re.search(r"assertABoxSatisfiable\s*\(\s*true\s*\)", body):
        return True
    if re.search(r"assertABoxSatisfiable\s*\(\s*false\s*\)", body):
        return False
    if re.search(r"assertFalse\s*\(\s*m_reasoner\.isConsistent\s*\(\s*\)\s*\)", body):
        return False
    if re.search(r"assertTrue\s*\(\s*m_reasoner\.isConsistent\s*\(\s*\)\s*\)", body):
        return True
    if re.search(r"assertTrue\s*\(\s*[^)]*isConsistent", body):
        m = re.search(r"assertTrue\s*\(\s*[^,]+,\s*(true|false)\s*\)", body)
        if m:
            return m.group(1) == "true"
    return None


def valid_ofn_axioms(axioms: str) -> bool:
    """Reject fragments that still depend on Java constants or are syntactically broken."""
    trimmed = axioms.strip()
    if not trimmed:
        return False
    if '<"' in trimmed or re.search(r"<\s*$", trimmed):
        return False
    if trimmed.count("(") != trimmed.count(")"):
        return False
    return True


def extract_axioms_assignments(body: str) -> str:
    """Concatenate `String <var> = ...` fragments up to the load call."""
    load_m = re.search(
        r"load(?:Reasoner|Ontology)WithAxioms\s*\(\s*(\w+)\s*\)",
        body,
    )
    if not load_m:
        return ""
    var = load_m.group(1)
    assign_m = re.search(rf"\bString\s+{re.escape(var)}\s*=", body)
    if not assign_m:
        return ""
    rest = body[assign_m.start() :]
    end_m = re.search(
        rf"load(?:Reasoner|Ontology)WithAxioms\s*\(\s*{re.escape(var)}\s*\)",
        rest,
    )
    if not end_m:
        return ""
    chunk = rest[: end_m.start()]
    if re.search(r'"\s*\+\s*[A-Za-z_]', chunk):
        return ""
    return extract_string_literals(chunk)


def extract_axioms_literal(body: str) -> str:
    """Extract functional-syntax axioms from loadReasonerWithAxioms / loadOntologyWithAxioms."""
    active = "\n".join(
        line
        for line in body.splitlines()
        if not re.match(r"^\s*//", line)
    )
    assigned = extract_axioms_assignments(active)
    if assigned:
        return assigned
    ax_m = re.search(
        r"(?:loadReasonerWithAxioms|loadOntologyWithAxioms)\s*\(([\s\S]*?)\)\s*;",
        active,
    )
    if ax_m:
        axioms = extract_string_literals(ax_m.group(1))
        if axioms:
            return axioms
    var_m = re.search(
        r'String\s+axioms\s*=\s*((?:"[^"\\]*(?:\\.[^"\\]*)*"\s*)+)\s*;',
        active,
    )
    if var_m:
        return extract_string_literals(var_m.group(1))
    return ""


DL_CONSTRUCTS = (
    "ObjectSomeValuesFrom",
    "ObjectAllValuesFrom",
    "ObjectComplementOf",
    "ObjectIntersectionOf",
    "ObjectUnionOf",
    "Nominal",
    "hasValue",
    "HasKey",
    "Cardinality",
    "ObjectExactCardinality",
    "ObjectMinCardinality",
    "ObjectMaxCardinality",
    "DataSomeValuesFrom",
    "DataAllValuesFrom",
    "Datatype",
    "Literal",
)


def infer_engine(case_id: str, body: str) -> str:
    if case_id in FORCE_RL_ENGINE_IDS:
        return "rl"
    if case_id in FORCE_DL_AXIOM_IDS:
        return "dl"
    if case_id in FORCE_DL_CONSISTENCY_IDS:
        return "dl"
    if case_id.startswith(INTERNAL_PREFIXES):
        return "internal"
    if case_id.startswith("owl_wg_tests."):
        return "dl"
    if case_id.startswith("reasoner.RIATest"):
        return "dl"
    if case_id.startswith(DEFERRED_PREFIXES):
        return "swrl"
    if "ClassificationTest" in case_id:
        return "el"
    if case_id.startswith(FORCE_DL_PREFIXES):
        return "dl"
    if any(x in body for x in DL_CONSTRUCTS):
        return "dl"
    if any(h in body for h in RDFS_HINTS) and not any(
        x in body
        for x in (
            "ObjectSomeValuesFrom",
            "ObjectAllValuesFrom",
            "ObjectComplementOf",
            "ObjectIntersectionOf",
            "ObjectUnionOf",
        )
    ):
        return "rdfs"
    if any(h in body for h in RL_HINTS):
        return "rl"
    if case_id.startswith(DL_PREFIXES):
        return "dl"
    return "dl"


def infer_status(case: HermitCase) -> None:
    _assign_catalog_status(case)
    apply_all_active_java(case)


def _assign_catalog_status(case: HermitCase) -> None:
    if case.id in MIGRATED_INTERNAL_IDS:
        case.status = "migrated"
        case.ignore_reason = "ported to ontologos-alc/dl unit tests"
        return
    if case.id in EXCLUDED_IDS:
        case.status = "excluded"
        case.ignore_reason = "documented semantic or mapping gap (see manifest)"
        return
    if case.id in PROMOTED_AXIOM_IDS and case.axiom_ofn:
        case.status = "axiom"
        case.tier = "A"
        case.ignore_reason = None
        return
    if case.id in DEFERRED_DL_AXIOM_IDS:
        case.status = "planned"
        case.ignore_reason = "complex DL subsumption (Phase 2 tableau)"
        return
    if case.id in DEFERRED_RL_AXIOM_IDS:
        case.status = "planned"
        case.ignore_reason = "RL/RDFS axiom assertions pending engine hardening"
        return
    if case.fixture in MISSING_FIXTURES:
        case.status = "excluded"
        case.ignore_reason = "fixture not vendored (see benchmarks manifest)"
        return
    if case.id in IMPLEMENTED:
        case.status = "ported"
        case.rust_test = IMPLEMENTED[case.id]
        case.hand_written = True
        return
    if case.id.startswith(DEFERRED_PREFIXES):
        if case.axiom_ofn:
            case.status = "swrl"
            case.tier = "A"
            case.ignore_reason = None
        else:
            case.status = "planned"
            case.ignore_reason = "SWRL — deferred out of scope for OntoLogos 1.x"
        return
    if case.axiom_ofn and "Clausification" in case.java_class:
        case.status = "clausify"
        case.tier = "A"
        case.ignore_reason = None
        return
    if case.engine == "internal":
        case.status = "internal"
        case.ignore_reason = "HermiT engine-internal test — port when ontologos-dl internals are exposed"
        return
    if case.engine == "swrl":
        if case.axiom_ofn:
            case.status = "swrl"
            case.tier = "A"
        else:
            case.status = "planned"
            case.ignore_reason = "SWRL — requires ontologos-swrl (1.0)"
        return
    if case.golden and case.fixture:
        if case.engine == "el":
            case.status = "fixture"
            case.tier = "B"
        else:
            case.status = "planned"
            case.ignore_reason = f"fixture golden test requires {case.engine} (1.0)"
        return
    if case.axiom_ofn and case.subsumptions:
        if case.engine in ("rdfs", "rl"):
            if case.id in PROMOTED_AXIOM_IDS:
                case.status = "axiom"
                case.tier = "A"
            else:
                case.status = "planned"
                case.ignore_reason = "RL/RDFS axiom assertions pending engine hardening"
        elif case.engine in ("dl", "alc"):
            if case.id in PROMOTED_AXIOM_IDS:
                case.status = "axiom"
                case.tier = "A"
            else:
                case.status = "planned"
                case.ignore_reason = "DL axiom fixture; subsumption assertions pending engine (Phase 2+)"
        elif case.engine == "swrl":
            case.status = "swrl"
            case.tier = "A"
        else:
            case.status = "planned"
            case.ignore_reason = f"axiom fixture requires {case.engine} (1.0)"
        return
    if case.axiom_ofn and case.property_characteristics and case.engine in ("rdfs", "rl"):
        if case.id in PROMOTED_AXIOM_IDS:
            case.status = "axiom"
            case.tier = "A"
        else:
            case.status = "planned"
            case.ignore_reason = "RL/RDFS axiom assertions pending engine hardening"
        return
    if case.axiom_ofn and case.property_subsumptions and case.engine in ("rdfs", "rl"):
        if case.id in PROMOTED_AXIOM_IDS:
            case.status = "axiom"
            case.tier = "A"
        else:
            case.status = "planned"
            case.ignore_reason = "RL/RDFS axiom assertions pending engine hardening"
        return
    if case.axiom_ofn and case.consistent is not None and case.engine in ("dl", "alc"):
        if case.id in PROMOTED_AXIOM_IDS:
            case.status = "axiom"
            case.tier = "A"
        else:
            case.status = "axiom"
            case.tier = "A"
            case.ignore_reason = None
        return
    if case.axiom_ofn and case.consistent is not None and case.engine in ("rdfs", "rl"):
        if case.id in APPROVED_RL_CONSISTENCY_IDS:
            case.status = "axiom"
            case.tier = "A"
            return
        case.status = "planned"
        case.ignore_reason = "RL/RDFS consistency assertions pending engine hardening"
        return
    if case.axiom_ofn and "ClausificationDatatypes" in case.java_class:
        case.status = "planned"
        case.tier = "A"
        case.ignore_reason = "datatype clausification goldens (Phase 5)"
        return
    if case.axiom_ofn and case.engine == "internal" and "Normalization" in case.java_class:
        case.status = "clausify"
        case.tier = "A"
        case.ignore_reason = None
        return
    if case.axiom_ofn and "Clausification" in case.java_class:
        case.status = "clausify"
        case.tier = "A"
        case.ignore_reason = None
        return
    if case.axiom_ofn and (case.ria_regular or case.role_simple):
        case.status = "axiom"
        case.tier = "A"
        case.ignore_reason = None
        return
    if case.axiom_ofn and has_axiom_assertions(case) and case.engine in ("dl", "alc"):
        if case.id in PROMOTED_AXIOM_IDS:
            case.status = "axiom"
            case.tier = "A"
        else:
            case.status = "axiom"
            case.tier = "A"
            case.ignore_reason = None
        return
    if case.axiom_ofn and has_axiom_assertions(case) and case.load_error_expected:
        case.status = "planned"
        case.ignore_reason = "DL axiom fixture; load error expected (Phase 2+)"
        return
    if case.axiom_ofn and case.engine in ("dl", "alc") and not has_axiom_assertions(case):
        case.status = "planned"
        case.ignore_reason = "DL axiom fixture; assertions pending engine (Phase 2+)"
        return
    case.status = "planned"
    case.ignore_reason = f"auto-cataloged; requires manual port ({case.engine})"


def strip_ofn_comments(axioms: str) -> str:
    """Remove Java-style line comments from extracted OFN axiom literals."""
    out: list[str] = []
    for line in axioms.splitlines():
        cleaned: list[str] = []
        in_string = False
        i = 0
        while i < len(line):
            ch = line[i]
            if ch == '"' and (i == 0 or line[i - 1] != "\\"):
                in_string = not in_string
                cleaned.append(ch)
            elif not in_string and ch == "/" and i + 1 < len(line) and line[i + 1] == "/":
                break
            else:
                cleaned.append(ch)
            i += 1
        stripped = "".join(cleaned).strip()
        if stripped:
            out.append(stripped)
    return "\n".join(out)


def normalize_ofn_axioms(axioms: str) -> str:
    """Repair HermiT-only OFN fragments for horned-owl parsing."""
    # Unary ObjectPropertyChain is accepted by HermiT but not valid OWL FS (needs 2+ OPEs).
    axioms = re.sub(
        r"SubObjectPropertyOf\(\s*ObjectPropertyChain\(\s*(:[\w-]+)\s*\)\s*(:[\w-]+)\s*\)",
        r"SubObjectPropertyOf(\1 \2)",
        axioms,
    )
    return axioms


def wrap_ofn(axioms: str) -> str:
    axioms = strip_ofn_comments(axioms)
    axioms = normalize_ofn_axioms(axioms)
    prefixes = [
        "Prefix(:=<file:/c/test.owl#>)",
        "Prefix(a:=<file:/c/test.owl#>)",
        "Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)",
        "Prefix(owl:=<http://www.w3.org/2002/07/owl#>)",
        "Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)",
    ]
    if "rdf:" in axioms:
        prefixes.append(
            "Prefix(rdf:=<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)"
        )
    if "owl:rational" in axioms:
        pass  # owl prefix already included
    return (
        "\n".join(prefixes)
        + "\n"
        + "Ontology(<file:/c/test.owl#>\n"
        + f"{axioms}\n"
        + ")\n"
    )


def rust_fn_name(case_id: str) -> str:
    return "hermit_" + re.sub(r"[^a-zA-Z0-9]+", "_", case_id).strip("_").lower()


def harvest_assertions(case: HermitCase, body: str) -> None:
    """Populate catalog assertion fields from Java test body."""
    case.subsumptions = filter_java_ce_subsumptions(extract_subsumptions(body))
    case.subsumptions.extend(
        filter_java_ce_subsumptions(extract_entailment_subsumptions(body))
    )
    case.subsumptions.extend(extract_equivalence_subsumptions(body))
    case.subsumptions.extend(extract_top_op_equivalence(body))
    case.subsumptions.extend(extract_object_property_domains(body))
    if not case.subsumptions and case.id in HARDCODED_AXIOM_SUBSUMPTIONS:
        case.subsumptions = HARDCODED_AXIOM_SUBSUMPTIONS[case.id]

    case.property_subsumptions = extract_property_subsumptions(body)
    obj_props, data_props = extract_property_hierarchy(body)
    case.property_subsumptions.extend(obj_props)
    eq_obj, eq_data = extract_equivalent_properties(body)
    case.property_subsumptions.extend(eq_obj)
    case.data_property_subsumptions = data_props
    case.data_property_subsumptions.extend(eq_data)

    case.property_characteristics = extract_property_characteristics(body)
    case.property_characteristics.extend(extract_functional_data_property(body))

    case.class_satisfiability = extract_assert_satisfiable(body)
    case.class_satisfiability.extend(extract_ce_satisfiability_fallback(body))
    if case.id in HARDCODED_CLASS_SATISFIABILITY:
        case.class_satisfiability = HARDCODED_CLASS_SATISFIABILITY[case.id]

    conclusion, expected_ent = extract_entailment_metadata(body)
    if not conclusion:
        conclusion, expected_ent = extract_has_key_entailment(body)
    if not conclusion:
        conclusion, expected_ent = extract_entailment_checker_fail(body)
    if not conclusion:
        conclusion, expected_ent = extract_datatype_def_entailment(body)
    if not conclusion:
        conclusion, expected_ent = extract_subproperty_chain_entailment(body)
    if conclusion and expected_ent is not None and valid_ofn_axioms(conclusion):
        safe = rust_fn_name(case.id)
        case.conclusion_ofn = f"axioms/{safe}_conclusion.ofn"
        case.expected_entailment = expected_ent

    inc = extract_incremental_ofn(body)
    if inc and valid_ofn_axioms(inc):
        case.incremental_ofn = f"axioms/{rust_fn_name(case.id)}_incremental.ofn"

    case.individual_types = extract_individual_types(body)
    if case.id in HARDCODED_INDIVIDUAL_TYPES:
        case.individual_types = HARDCODED_INDIVIDUAL_TYPES[case.id]

    case.individual_instances = extract_instance_retrieval(body)
    case.datalog_queries = extract_datalog_queries(body)
    case.load_error_expected = extract_load_error_expected(body)

    dr = extract_assert_drsatisfiable(body)
    if dr and valid_ofn_axioms(dr["axioms"]):
        safe = rust_fn_name(case.id)
        case.axiom_ofn = f"axioms/{safe}.ofn"
        case.consistent = dr["expected"]

    ria = extract_ria_regularity(body)
    if ria and valid_ofn_axioms(ria["axioms"]):
        safe = rust_fn_name(case.id)
        case.axiom_ofn = f"axioms/{safe}.ofn"
        case.ria_regular = ria

    simple = extract_role_simplicity(body)
    if simple and valid_ofn_axioms(simple["axioms"]):
        safe = rust_fn_name(case.id)
        case.axiom_ofn = f"axioms/{safe}.ofn"
        case.role_simple = simple

    if case.consistent is None:
        case.consistent = extract_consistency(body)
    if case.incremental_ofn and re.search(
        r"assertFalse\s*\(\s*m_reasoner\.isConsistent|assertNotConsistent",
        body,
    ):
        case.consistent = False
    if case.id in INCREMENTAL_CONSISTENCY_IDS and not case.incremental_ofn:
        case.consistent = None

    apply_hardcoded_assertions(case)
    if case.id in HARDCODED_DATALOG_QUERIES:
        case.datalog_queries = HARDCODED_DATALOG_QUERIES[case.id]


def apply_hardcoded_assertions(case: HermitCase) -> None:
    if case.id not in HARDCODED_CASE_ASSERTIONS:
        return
    hard = HARDCODED_CASE_ASSERTIONS[case.id]
    if "subsumptions" in hard:
        case.subsumptions = hard["subsumptions"]
    if "consistent" in hard:
        case.consistent = hard["consistent"]
    if "property_subsumptions" in hard:
        case.property_subsumptions = hard["property_subsumptions"]
    if "individual_types" in hard:
        case.individual_types = hard["individual_types"]
    if "class_satisfiability" in hard:
        case.class_satisfiability = hard["class_satisfiability"]
    if "conclusion_ofn" in hard:
        case.conclusion_ofn = hard["conclusion_ofn"]
    if "expected_entailment" in hard:
        case.expected_entailment = hard["expected_entailment"]
    if "incremental_ofn" in hard:
        case.incremental_ofn = hard["incremental_ofn"]
    if "ce_instance_checks" in hard:
        case.ce_instance_checks = hard["ce_instance_checks"]
    if "ce_satisfiability" in hard:
        case.ce_satisfiability = hard["ce_satisfiability"]
    if "datalog_queries" in hard:
        case.datalog_queries = hard["datalog_queries"]


def collect_cases() -> list[HermitCase]:
    root = hermit_java_root()
    cases: list[HermitCase] = []
    for java in sorted(root.rglob("*.java")):
        if "rationals" in java.parts or SKIP_FILE.search(java.name):
            continue
        rel = java.relative_to(root)
        pkg = ".".join(rel.with_suffix("").parts[:-1])
        cls = java.stem
        java_class = f"{pkg}.{cls}" if pkg else cls
        text = java.read_text(errors="replace")
        for m in re.finditer(r"^[\t ]*public void (test\w+)\s*\(", text, re.MULTILINE):
            method = m.group(1)
            case_id = f"{java_class}.{method}"
            body = extract_method_body(text, method)
            case = HermitCase(
                id=case_id,
                java_class=java_class,
                java_method=method,
                java_file=str(rel),
                engine="dl",
                status="planned",
                tier="B" if "ClassificationTest" in java_class else "A",
            )
            case.engine = infer_engine(case_id, body)

            res_m = re.search(r'loadReasonerFromResource\s*\(\s*"([^"]+)"\s*\)', body)
            if res_m:
                case.fixture = res_m.group(1)
            gold_m = re.search(r'assertHierarchies\s*\(\s*"([^"]+)"\s*\)', body)
            if gold_m:
                case.golden = gold_m.group(1)

            if "loadReasonerWithAxioms" in body or "loadOntologyWithAxioms" in body:
                axioms = extract_axioms_literal(body)
                if not axioms:
                    axioms = extract_buffer_axioms(body)
                safe = rust_fn_name(case_id)
                ofn_rel = f"axioms/{safe}.ofn"
                if axioms and valid_ofn_axioms(axioms):
                    case.axiom_ofn = ofn_rel
                elif (OUT_AXIOMS.parent / ofn_rel).is_file():
                    case.axiom_ofn = ofn_rel

            harvest_assertions(case, body)
            infer_status(case)
            cases.append(case)
    return cases


def write_axioms(cases: list[HermitCase]) -> int:
    OUT_AXIOMS.mkdir(parents=True, exist_ok=True)
    expected: set[Path] = set()
    for case in cases:
        if case.axiom_ofn:
            expected.add(OUT_AXIOMS.parent / case.axiom_ofn)
        if case.conclusion_ofn:
            expected.add(OUT_AXIOMS.parent / case.conclusion_ofn)
        if case.incremental_ofn:
            expected.add(OUT_AXIOMS.parent / case.incremental_ofn)
    for existing in OUT_AXIOMS.glob("*.ofn"):
        expected.add(existing)
    for stale in OUT_AXIOMS.glob("*.ofn"):
        if stale not in expected:
            stale.unlink()
    written = 0
    for case in cases:
        if not case.axiom_ofn or case.id in OFN_WRITE_SKIP_IDS:
            continue
        src = JAVA_ROOT / case.java_file
        text = src.read_text(errors="replace")
        body = extract_method_body(text, case.java_method)
        axioms = extract_axioms_literal(body)
        if not axioms:
            axioms = extract_buffer_axioms(body)
        if not axioms:
            dr = extract_assert_drsatisfiable(body)
            if dr:
                axioms = dr["axioms"]
        if not axioms:
            ria = extract_ria_regularity(body)
            if ria:
                axioms = ria["axioms"]
        if not axioms:
            simple = extract_role_simplicity(body)
            if simple:
                axioms = simple["axioms"]
        if case.id in AXIOM_OFN_OVERRIDES:
            axioms = AXIOM_OFN_OVERRIDES[case.id]
        if not axioms or not valid_ofn_axioms(axioms):
            continue
        out = OUT_AXIOMS.parent / case.axiom_ofn
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(wrap_ofn(axioms), encoding="utf-8")
        written += 1

        if case.incremental_ofn:
            inc = extract_incremental_ofn(body)
            if not inc and case.id in HARDCODED_INCREMENTAL_AXIOMS:
                inc = HARDCODED_INCREMENTAL_AXIOMS[case.id]
            if inc and valid_ofn_axioms(inc):
                inc_out = OUT_AXIOMS.parent / case.incremental_ofn
                inc_out.write_text(wrap_ofn(inc), encoding="utf-8")
                written += 1

        if case.conclusion_ofn:
            conc_path = OUT_AXIOMS.parent / case.conclusion_ofn
            if not conc_path.is_file():
                conclusion, _ = extract_entailment_metadata(body)
                if not conclusion:
                    conclusion, _ = extract_has_key_entailment(body)
                if not conclusion:
                    conclusion, _ = extract_entailment_checker_fail(body)
                if not conclusion:
                    conclusion, _ = extract_datatype_def_entailment(body)
                if not conclusion:
                    conclusion, _ = extract_subproperty_chain_entailment(body)
                if not conclusion and case.id in HARDCODED_CONCLUSION_AXIOMS:
                    conclusion = HARDCODED_CONCLUSION_AXIOMS[case.id]
                if conclusion and valid_ofn_axioms(conclusion):
                    conc_path.write_text(wrap_ofn(conclusion), encoding="utf-8")
                    written += 1
    return written


def write_rust(cases: list[HermitCase]) -> None:
    lines = [
        "// Auto-generated by tests/hermit/generate_catalog.py — do not edit.",
        "",
        "use ontologos_conformance::run_hermit_case;",
        "",
    ]
    for case in cases:
        fn = rust_fn_name(case.id)
        ignore = ""
        if case.hand_written:
            lines.append(f"// Hand-written implementation: see hermit_rl/hermit_rdfs/hermit_el.rs")
            lines.append(f"#[test]")
            lines.append(f"#[ignore = \"implemented in hand-written module: {case.rust_test}\"]")
        elif case.status in ("excluded", "deferred", "internal", "planned", "migrated") and case.ignore_reason:
            reason = case.ignore_reason.replace('"', '\\"')
            lines.append(f"#[test]")
            lines.append(f'#[ignore = "{reason}"]')
        elif case.status == "fixture":
            lines.append(f"#[test]")
            if case.fixture in MISSING_FIXTURES:
                lines.append(
                    '#[ignore = "fixture not vendored (see benchmarks manifest)"]'
                )
            elif case.fixture in PARSER_IGNORE_FIXTURES:
                lines.append(
                    '#[ignore = "RDF/XML fixture not supported by parser yet (entities or duplicate rdf:ID)"]'
                )
            elif not (RES_ROOT / case.fixture.replace("res/", "reasoner/res/")).exists():
                lines.append(
                    '#[ignore = "requires HermiT fixture vendored or ONTOLOGOS_HERMIT_ROOT"]'
                )
        else:
            lines.append(f"#[test]")
        lines.append(f"fn {fn}() {{")
        lines.append(f'    run_hermit_case("{case.id}");')
        lines.append("}")
        lines.append("")
    OUT_RUST.write_text("\n".join(lines), encoding="utf-8")


WG_TEST_ABOUT_PREFIX = 'rdf:about="http://owl.semanticweb.org/id/'


def wg_test_block(text: str, block_start: int) -> str:
    """Return the full WG test-case RDF block (until the next test individual)."""
    nxt = text.find(WG_TEST_ABOUT_PREFIX, block_start + len(WG_TEST_ABOUT_PREFIX))
    if nxt < 0:
        return text[block_start:]
    return text[block_start:nxt]


def extract_wg_embedded_content(block: str, tag: str) -> str | None:
    m = re.search(
        rf"<test:{tag}[^>]*>(.*?)</test:{tag}>",
        block,
        re.DOTALL,
    )
    if not m:
        return None
    raw = html.unescape(m.group(1).strip())
    if not raw:
        return None
    if raw.startswith("<"):
        return raw
    if raw.startswith("Prefix") or raw.startswith("Ontology"):
        return raw
    return None


WG_BUILTIN_ENTITIES: dict[str, str] = {
    "vin": "http://www.w3.org/TR/2003/PR-owl-guide-20031209/wine#",
    "food": "http://www.w3.org/TR/2003/PR-owl-guide-20031209/food#",
    "owl": "http://www.w3.org/2002/07/owl#",
    "xsd": "http://www.w3.org/2001/XMLSchema#",
}


def expand_wg_doctype_entities(xml: str) -> str:
    """Expand internal XML entities declared in OWL WG DOCTYPE blocks."""
    entities: dict[str, str] = dict(WG_BUILTIN_ENTITIES)
    for m in re.finditer(r"<!ENTITY\s+(\w+)\s+\"([^\"]+)\"\s*>", xml):
        entities[m.group(1)] = m.group(2)
    out = re.sub(r"<!DOCTYPE[^[]*\[[\s\S]*?\]>\s*", "", xml)
    out = re.sub(r"(?:<!ENTITY[^>]*>\s*)+\]>\s*", "", out)
    if not any(f"&{name};" in xml for name in entities):
        return out
    for name, value in entities.items():
        out = out.replace(f"&{name};", value)
    return out


def write_wg_fixture_content(
    test_id: str,
    premise: str,
    conclusion: str | None,
    *,
    prem_ext: str = ".rdf",
    conc_ext: str = ".rdf",
) -> tuple[str | None, str | None]:
    """Write premise/conclusion strings to disk; return catalog-relative paths."""
    premise = expand_wg_doctype_entities(premise)
    if conclusion is not None:
        conclusion = expand_wg_doctype_entities(conclusion)
    safe = re.sub(r"[^a-zA-Z0-9._-]+", "_", test_id)
    out_dir = OUT_WG_DATA / safe
    out_dir.mkdir(parents=True, exist_ok=True)
    prem_path = out_dir / f"premise{prem_ext}"
    prem_path.write_text(premise, encoding="utf-8")
    prem_rel = f"wg/{safe}/premise{prem_ext}"
    conc_rel = None
    if conclusion:
        conc_path = out_dir / f"conclusion{conc_ext}"
        conc_path.write_text(conclusion, encoding="utf-8")
        conc_rel = f"wg/{safe}/conclusion{conc_ext}"
    return prem_rel, conc_rel


def extract_wg_import_ref(block: str) -> str | None:
    m = re.search(r'test:importedOntology rdf:resource="([^"]+)"', block)
    if not m:
        return None
    return m.group(1).rsplit("/", 1)[-1]


def is_stub_rdf(content: str) -> bool:
    stripped = re.sub(r"\s+", "", content)
    return stripped in (
        '<rdf:RDFxmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/>',
        "<rdf:RDF/>",
    )


def rdf_xml_base(fragment: str) -> str | None:
    m = re.search(r"""xml:base\s*=\s*['"]([^'"]+)['"]""", fragment)
    if not m:
        return None
    return m.group(1).rstrip("#").rstrip("/")


def absolutize_rdf_ids(body: str, base: str) -> str:
    """Rewrite `rdf:ID` fragments to absolute `rdf:about` using the source ontology base."""
    base = base.rstrip("/").rstrip("#")
    ids = re.findall(r"""rdf:ID=["']([^"']+)["']""", body)
    out = re.sub(
        r"""rdf:ID=["']([^"']+)["']""",
        lambda m: f'rdf:about="{base}#{m.group(1)}"',
        body,
    )
    for id_ in ids:
        out = out.replace(f'rdf:resource="#{id_}"', f'rdf:resource="{base}#{id_}"')
        out = out.replace(f"rdf:resource='#{id_}'", f"rdf:resource='{base}#{id_}'")
    return out


def merge_rdf_xml(main: str, imported: str) -> str:
    """Inline an imported OWL/RDF document into a main document (drop owl:imports)."""

    def inner(xml: str) -> tuple[str, str]:
        m = re.search(r"<rdf:RDF([^>]*)>(.*)</rdf:RDF>", xml, re.DOTALL)
        if not m:
            return "", xml
        return m.group(1), m.group(2)

    main_attrs, main_body = inner(main)
    import_attrs, import_body = inner(imported)
    attrs = main_attrs or import_attrs
    import_base = rdf_xml_base(import_attrs) or rdf_xml_base(imported)
    if import_base:
        import_body = absolutize_rdf_ids(import_body, import_base)
    # Drop owl:imports declarations from the main ontology header.
    main_body = re.sub(
        r"<owl:imports[^>]*>.*?</owl:imports>",
        "",
        main_body,
        flags=re.DOTALL,
    )
    main_body = re.sub(r"<owl:imports[^>]*/>", "", main_body)
    merged_body = (import_body.strip() + "\n" + main_body.strip()).strip()
    return f"<rdf:RDF{attrs}>\n{merged_body}\n</rdf:RDF>"


def wg_ontologies_dir() -> Path:
    return RES_ROOT / "owl_wg_tests/ontologies"


def import_uri_to_local_file(uri: str) -> Path | None:
    """Map an owl:imports URI to a vendored ontology file when present."""
    name = uri.rstrip("/").rsplit("/", 1)[-1]
    if not name:
        return None
    base = wg_ontologies_dir()
    for candidate in (base / f"{name}.rdf", base / f"{name}.ofn"):
        if candidate.is_file():
            return candidate
    return None


def resolve_import_documents(premise_xml: str) -> str:
    """Inline owl:imports targets from vendored WG ontology files."""
    merged = premise_xml
    uris: list[str] = []
    for m in re.finditer(r'<owl:imports[^>]*rdf:resource="([^"]+)"', merged, re.DOTALL):
        uris.append(m.group(1))
    for m in re.finditer(
        r"<owl:imports[^>]*>\s*<owl:Ontology[^>]*rdf:about=\"([^\"]+)\"",
        merged,
        re.DOTALL,
    ):
        uris.append(m.group(1))
    for m in re.finditer(r"xmlns:\w+\s*=\s*['\"]([^'\"]+)['\"]", merged):
        uri = m.group(1).rstrip("#").rstrip("/")
        if uri and "/imports/" in uri:
            uris.append(uri)
    seen: set[str] = set()
    for uri in uris:
        if uri in seen:
            continue
        seen.add(uri)
        local = import_uri_to_local_file(uri)
        if local is None:
            continue
        imported = expand_wg_doctype_entities(
            local.read_text(encoding="utf-8", errors="replace")
        )
        merged = merge_rdf_xml(merged, imported)
    return merged


def index_wg_blocks(text: str) -> dict[str, str]:
    """Map WG test id -> isolated RDF block from all.rdf."""
    blocks: dict[str, str] = {}
    prefix = WG_TEST_ABOUT_PREFIX
    for m in re.finditer(rf'{re.escape(prefix)}([^"]+)"', text):
        test_id = m.group(1)
        blocks[test_id] = wg_test_block(text, m.start())
    return blocks


def write_wg_fixture(
    test_id: str,
    block: str,
    *,
    negative_entailment: bool = False,
) -> tuple[str | None, str | None]:
    premise = extract_wg_embedded_content(block, "rdfXmlPremiseOntology")
    prem_ext = ".rdf"
    if not premise:
        premise = extract_wg_embedded_content(block, "fsPremiseOntology")
        if premise:
            prem_ext = ".ofn"
    if not premise:
        premise = extract_wg_embedded_content(block, "rdfXmlInputOntology")
    if not premise:
        return None, None

    premise = resolve_import_documents(premise)

    conclusion = None
    conc_ext = ".rdf"
    if negative_entailment:
        conclusion = extract_wg_embedded_content(block, "rdfXmlNonConclusionOntology")
        if not conclusion:
            conclusion = extract_wg_embedded_content(block, "fsNonConclusionOntology")
            if conclusion:
                conc_ext = ".ofn"
    else:
        conclusion = extract_wg_embedded_content(block, "rdfXmlConclusionOntology")
        if not conclusion:
            conclusion = extract_wg_embedded_content(block, "fsConclusionOntology")
            if conclusion:
                conc_ext = ".ofn"

    return write_wg_fixture_content(
        test_id, premise, conclusion, prem_ext=prem_ext, conc_ext=conc_ext
    )


# When both ``premise.rdf`` and ``premise.ofn`` exist, the RDF export is usually the WG
# entailment fixture; these ids use curated OFN instead (RDF on disk is a stale alternate).
WG_PREMISE_OFN_WHEN_DUAL = frozenset(
    {
        "Contradicting-2Ddatatype-2Drestrictions",
        "Contradicting-2DdateTime-2Drestrictions",
        "Minus-2Dinf-2Dnot-2Dowlreal",
        "Qualified-2Dcardinality-2Dboolean",
        "Qualified-2Dcardinality-2Drestricted-2Dint",
        "String-2Dinteger-2Dclash",
    }
)

WG_CONCLUSION_OFN_WHEN_DUAL = frozenset(
    {
        "Qualified-2Dcardinality-2Drestricted-2Dint",
    }
)


def wg_premise_on_disk(out_dir: Path, test_id: str = "") -> Path | None:
    """Select vendored premise fixture; prefer OFN only for ``WG_PREMISE_OFN_WHEN_DUAL``."""
    prem_rdf = out_dir / "premise.rdf"
    prem_ofn = out_dir / "premise.ofn"
    if prem_ofn.is_file() and prem_rdf.is_file():
        return prem_ofn if test_id in WG_PREMISE_OFN_WHEN_DUAL else prem_rdf
    if prem_rdf.is_file() and not is_stub_rdf(prem_rdf.read_text(errors="replace")):
        return prem_rdf
    if prem_ofn.is_file():
        return prem_ofn
    return None


def wg_conclusion_on_disk(out_dir: Path, test_id: str = "") -> Path | None:
    conc_rdf = out_dir / "conclusion.rdf"
    conc_ofn = out_dir / "conclusion.ofn"
    if conc_ofn.is_file() and conc_rdf.is_file():
        return conc_ofn if test_id in WG_CONCLUSION_OFN_WHEN_DUAL else conc_rdf
    if conc_rdf.is_file() and not is_stub_rdf(conc_rdf.read_text(errors="replace")):
        return conc_rdf
    if conc_ofn.is_file():
        return conc_ofn
    return None


def wg_disk_fixture_refs(test_id: str) -> tuple[str | None, str | None]:
    """Return catalog-relative paths when fixtures were vendored on a prior run."""
    out_dir = OUT_WG_DATA / test_id
    prem = wg_premise_on_disk(out_dir, test_id)
    conc = wg_conclusion_on_disk(out_dir, test_id)
    prem_rel = f"wg/{test_id}/{prem.name}" if prem is not None else None
    conc_rel = f"wg/{test_id}/{conc.name}" if conc is not None else None
    return prem_rel, conc_rel


def detect_wg_test_type(block: str) -> tuple[str, bool | None, bool | None]:
    """Return (test_type, expected_entailment, expected_consistent) from an isolated block."""
    if "PositiveEntailmentTest" in block:
        return "positive_entailment", True, None
    if "NegativeEntailmentTest" in block:
        return "negative_entailment", False, None
    if "InconsistencyTest" in block:
        return "inconsistency", None, False
    if "ConsistencyTest" in block:
        return "consistency", None, True
    return "entailment", None, None


# Manifest extraction sometimes tags inconsistency fixtures as ConsistencyTest or
# PositiveEntailmentTest when the WG export embeds a bogus conclusion document.
WG_CONSISTENCY_OVERRIDES: dict[str, tuple[str, bool]] = {
    "TestCase-3AWebOnt-2Ddescription-2Dlogic-2D017": ("inconsistency", False),
    "TestCase-3AWebOnt-2Ddescription-2Dlogic-2D033": ("inconsistency", False),
    "TestCase-3AWebOnt-2Ddescription-2Dlogic-2D633": ("inconsistency", False),
    "TestCase-3AWebOnt-2DRestriction-2D001": ("inconsistency", False),
    "TestCase-3AWebOnt-2DRestriction-2D002": ("inconsistency", False),
    "TestCase-3AWebOnt-2DmaxCardinality-2D001": ("inconsistency", False),
    "TestCase-3AWebOnt-2DI4.5-2D002": ("inconsistency", False),
}


def collect_wg_cases() -> list[WgCase]:
    """Catalog in-scope OWL WG DL tests from all.rdf (see wg_in_scope_ids.txt)."""
    all_rdf = RES_ROOT / "owl_wg_tests/ontologies/all.rdf"
    if not all_rdf.is_file():
        return []
    in_scope = load_wg_in_scope_ids()
    text = all_rdf.read_text(errors="replace")
    wg_blocks = index_wg_blocks(text)
    cases: list[WgCase] = []
    for test_id, block in wg_blocks.items():
        if in_scope and test_id not in in_scope:
            continue
        if not in_scope and ("Approved" not in block or "DL" not in block):
            continue
        test_type, expected_entailment, expected_consistent = detect_wg_test_type(block)
        if test_id in WG_CONSISTENCY_OVERRIDES:
            test_type, expected_consistent = WG_CONSISTENCY_OVERRIDES[test_id]
            expected_entailment = None
        premise_ofn = None
        conclusion_ofn = None
        status = "planned"
        ignore_reason = "WG test — requires ontologos-dl + vendored WG OFN fixtures"

        if expected_entailment is not None:
            prem_rel, conc_rel = write_wg_fixture(
                test_id,
                block,
                negative_entailment=expected_entailment is False,
            )
            premise_ofn = prem_rel
            conclusion_ofn = conc_rel
        elif expected_consistent is not None:
            prem_rel, _ = write_wg_fixture(test_id, block)
            premise_ofn = prem_rel
        else:
            input_ont = extract_wg_embedded_content(block, "rdfXmlInputOntology")
            if input_ont:
                import_id = extract_wg_import_ref(block)
                if import_id:
                    import_block = wg_blocks.get(import_id)
                    import_ont = (
                        extract_wg_embedded_content(import_block, "rdfXmlInputOntology")
                        if import_block
                        else None
                    )
                    if import_ont:
                        input_ont = merge_rdf_xml(input_ont, import_ont)
                input_ont = resolve_import_documents(input_ont)
                prem_rel, _ = write_wg_fixture_content(test_id, input_ont, None)
                premise_ofn = prem_rel
                if import_id or "imports" in test_id:
                    test_type = "consistency"
                    expected_consistent = True
                    expected_entailment = None
                    conclusion_ofn = None
                elif test_id.startswith("WebOnt-2Dimports-"):
                    test_type = "consistency"
                    expected_consistent = True
                    expected_entailment = None
                    conclusion_ofn = None

        prem_disk, conc_disk = wg_disk_fixture_refs(test_id)
        if premise_ofn is None and prem_disk:
            prem_path = OUT_WG_DATA / test_id / Path(prem_disk).name
            if prem_path.is_file() and not is_stub_rdf(prem_path.read_text(errors="replace")):
                premise_ofn = prem_disk
        if conclusion_ofn is None and conc_disk and expected_entailment is not None:
            conc_path = OUT_WG_DATA / test_id / Path(conc_disk).name
            if conc_path.is_file() and not is_stub_rdf(conc_path.read_text(errors="replace")):
                conclusion_ofn = conc_disk

        if (
            expected_entailment is None
            and expected_consistent is None
            and premise_ofn
            and conclusion_ofn
            and "PositiveEntailmentTest" in block
        ):
            test_type = "positive_entailment"
            expected_entailment = True

        if wg_should_be_active(
            test_id, premise_ofn, conclusion_ofn, expected_consistent
        ):
            status = "wg"
            ignore_reason = None
        cases.append(
            WgCase(
                id=f"owl_wg_tests.{test_id}",
                test_type=test_type,
                status=status,
                engine="dl",
                premise_ofn=premise_ofn,
                conclusion_ofn=conclusion_ofn,
                expected_entailment=expected_entailment,
                expected_consistent=expected_consistent,
                ignore_reason=ignore_reason,
            )
        )
    return cases


def write_wg_rust(cases: list[WgCase]) -> None:
    lines = [
        "// Auto-generated by tests/hermit/generate_catalog.py — do not edit.",
        "",
        "use ontologos_conformance::run_wg_case;",
        "",
    ]
    for case in cases:
        fn = rust_fn_name(case.id)
        if case.status != "wg":
            reason = (case.ignore_reason or "planned WG test").replace('"', '\\"')
            lines.append(f"#[test]")
            lines.append(f'#[ignore = "{reason}"]')
        else:
            lines.append(f"#[test]")
        lines.append(f"fn {fn}() {{")
        lines.append(f'    run_wg_case("{case.id}");')
        lines.append("}")
        lines.append("")
    OUT_WG_RUST.write_text("\n".join(lines), encoding="utf-8")


def promote_wg_from_disk() -> None:
    """Activate WG cases with vendored premise/conclusion RDF on disk."""
    global PROMOTED_WG_IDS
    PROMOTED_WG_IDS = load_promoted_wg_ids()
    wg_path = OUT_WG_CATALOG
    raw = json.loads(wg_path.read_text(encoding="utf-8"))
    updated: list[dict] = []
    active = 0
    for row in raw:
        test_id = row["id"].split(".", 1)[-1]
        prem_rdf = OUT_WG_DATA / test_id / "premise.rdf"
        prem_ofn = OUT_WG_DATA / test_id / "premise.ofn"
        conc_rdf = OUT_WG_DATA / test_id / "conclusion.rdf"
        conc_ofn = OUT_WG_DATA / test_id / "conclusion.ofn"
        prem = wg_premise_on_disk(OUT_WG_DATA / test_id, test_id)
        conc = wg_conclusion_on_disk(OUT_WG_DATA / test_id, test_id)
        if prem is not None and is_stub_rdf(prem.read_text(errors="replace")):
            prem = None
        if conc is not None and is_stub_rdf(conc.read_text(errors="replace")):
            conc = None
        if prem is not None:
            row["premise_ofn"] = f"wg/{test_id}/{prem.name}"
            override = WG_CONSISTENCY_OVERRIDES.get(test_id)
            if override is not None:
                test_type, expected = override
                row["test_type"] = test_type
                row["expected_consistent"] = expected
                row["expected_entailment"] = None
                row["conclusion_ofn"] = None
            elif conc is not None:
                row["conclusion_ofn"] = f"wg/{test_id}/{conc.name}"
            if wg_should_be_active(
                test_id,
                row.get("premise_ofn"),
                row.get("conclusion_ofn"),
                row.get("expected_consistent"),
            ):
                row["status"] = "wg"
                row["ignore_reason"] = None
                active += 1
            elif row.get("conclusion_ofn") or row.get("expected_consistent") is not None:
                row["status"] = "planned"
                row["ignore_reason"] = "WG test — pending ontologos-dl entailment promotion"
        updated.append(row)
    wg_path.write_text(json.dumps(updated, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    cases = [WgCase(**row) for row in updated]
    write_wg_rust(cases)
    print(f"WG promote-only: {active} active of {len(updated)}")


def activate_all_from_disk() -> None:
    """Activate all runnable catalog cases from disk (no HermiT checkout required)."""
    global ALL_WG_ACTIVE, ALL_JAVA_ACTIVE, PROMOTED_AXIOM_IDS, PROMOTED_WG_IDS
    ALL_WG_ACTIVE = True
    ALL_JAVA_ACTIVE = True
    PROMOTED_AXIOM_IDS = load_promoted_axiom_ids()
    PROMOTED_WG_IDS = load_promoted_wg_ids()
    promote_only_from_disk()
    promote_wg_from_disk()
    wg_cases = [WgCase(**row) for row in json.loads(OUT_WG_CATALOG.read_text(encoding="utf-8"))]
    java_cases = [HermitCase(**row) for row in json.loads((OUT_CATALOG / "cases.json").read_text(encoding="utf-8"))]
    wg_active = sum(1 for c in wg_cases if c.status == "wg")
    java_active = sum(
        1
        for c in java_cases
        if c.status in ("axiom", "clausify", "swrl", "fixture")
    )
    print(
        f"activate-all-from-disk: {wg_active}/{len(wg_cases)} WG active, "
        f"{java_active}/{len(java_cases)} Java runnable active"
    )


def promote_only_from_disk() -> None:
    """Re-apply promoted_axiom_ids.txt to existing cases.json without HermiT checkout."""
    global PROMOTED_AXIOM_IDS
    PROMOTED_AXIOM_IDS = load_promoted_axiom_ids()
    catalog_path = OUT_CATALOG / "cases.json"
    raw = json.loads(catalog_path.read_text(encoding="utf-8"))
    cases: list[HermitCase] = []
    for row in raw:
        case = HermitCase(**row)
        apply_hardcoded_assertions(case)
        if case.id in HARDCODED_DATALOG_QUERIES:
            case.datalog_queries = HARDCODED_DATALOG_QUERIES[case.id]
        if case.id in FORCE_DL_AXIOM_IDS or case.id in FORCE_DL_CONSISTENCY_IDS:
            case.engine = "dl"
        if case.id in INCREMENTAL_CONSISTENCY_IDS and not case.incremental_ofn:
            case.consistent = None
        infer_status(case)
        cases.append(case)
    catalog_path.write_text(
        json.dumps([asdict(c) for c in cases], indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    write_rust(cases)
    by_status: dict[str, int] = {}
    for c in cases:
        by_status[c.status] = by_status.get(c.status, 0) + 1
    print(f"promoted-only refresh: {len(cases)} cases")
    print(f"  by status: {by_status}")
    print(f"  wrote {catalog_path.relative_to(REPO)}")
    print(f"  wrote {OUT_RUST.relative_to(REPO)}")


def wg_catalog_only() -> None:
    """Vendor WG fixtures and refresh wg_cases.json without HermiT Java sources."""
    global PROMOTED_WG_IDS, WG_IN_SCOPE_IDS
    PROMOTED_WG_IDS = load_promoted_wg_ids()
    WG_IN_SCOPE_IDS = load_wg_in_scope_ids()
    wg_cases = collect_wg_cases()
    OUT_CATALOG.mkdir(parents=True, exist_ok=True)
    wg_path = OUT_WG_CATALOG
    wg_path.write_text(
        json.dumps([asdict(c) for c in wg_cases], indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    write_wg_rust(wg_cases)
    active = sum(1 for c in wg_cases if c.status == "wg")
    print(f"WG catalog refresh: {len(wg_cases)} cases, {active} active")
    print(f"  wrote {wg_path.relative_to(REPO)}")
    print(f"  wrote {OUT_WG_RUST.relative_to(REPO)}")


def main() -> None:
    argv = sys.argv[1:]
    configure_activation_flags(argv)
    if argv and argv[0] == "--promote-only":
        promote_only_from_disk()
        return
    if argv and argv[0] == "--promote-wg-only":
        promote_wg_from_disk()
        return
    if argv and argv[0] == "--activate-all-from-disk":
        activate_all_from_disk()
        return
    if argv and argv[0] == "--wg-catalog-only":
        wg_catalog_only()
        return
    cases = collect_cases()
    wg_cases = collect_wg_cases()
    OUT_CATALOG.mkdir(parents=True, exist_ok=True)
    n_axiom = write_axioms(cases)
    catalog_path = OUT_CATALOG / "cases.json"
    catalog_path.write_text(
        json.dumps([asdict(c) for c in cases], indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    wg_path = OUT_WG_CATALOG
    wg_path.write_text(
        json.dumps([asdict(c) for c in wg_cases], indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    write_rust(cases)
    write_wg_rust(wg_cases)
    by_status: dict[str, int] = {}
    for c in cases:
        by_status[c.status] = by_status.get(c.status, 0) + 1
    print(f"cataloged {len(cases)} HermiT test methods")
    print(f"  by status: {by_status}")
    print(f"  axiom .ofn files: {n_axiom}")
    print(f"  WG tests cataloged: {len(wg_cases)}")
    print(f"  wrote {catalog_path.relative_to(REPO)}")
    print(f"  wrote {wg_path.relative_to(REPO)}")
    print(f"  wrote {OUT_RUST.relative_to(REPO)}")
    print(f"  wrote {OUT_WG_RUST.relative_to(REPO)}")


if __name__ == "__main__":
    main()
