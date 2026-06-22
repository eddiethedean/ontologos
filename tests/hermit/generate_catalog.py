#!/usr/bin/env python3
"""Generate HermiT port catalog (JSON + Rust tests) from a local hermit-reasoner checkout.

Run from repo root:
  python3 tests/hermit/generate_catalog.py

Requires HermiT/ (owlcs/hermit-reasoner) or ONTOLOGOS_HERMIT_ROOT.
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
HERMIT = Path(os.environ.get("ONTOLOGOS_HERMIT_ROOT", REPO / "HermiT"))


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

# All Approved DL WG tests with embedded RDF are vendored from all.rdf during --wg-catalog-only.
# Only ids in promoted_wg_ids.txt (from `promote_wg`) become status=wg in CI.

SKIP_FILE = re.compile(
    r"(Abstract|AllTests|AllQuick|Descriptor|Registry|Invalid|Failing|TstDescriptor|AllWG|AllApproved|AllExtracredit|AllNonRejected|AllProposed)"
)

# ReasonerTest incremental consistency checks — static OFN is initial load only.
INCREMENTAL_CONSISTENCY_IDS: set[str] = {
    "reasoner.ReasonerTest.testIncrementalWithNegatedClass",
    "reasoner.ReasonerTest.testIncrementalWithNegatedHasSelf",
    "reasoner.ReasonerTest.testIncrementalWithNegatedHasValue",
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
    "reasoner.ReasonerTest.testSubProperties": "excluded_subproperties",
    "reasoner.ReasonerTest.testObjectPropertyHierarchy": "excluded_object_property_hierarchy",
    "reasoner.ReasonerTest.testIsSymmetricObject": "excluded_symmetric_subproperty",
    "reasoner.ReasonerTest.testIsTransitiveObject": "excluded_transitive_subproperty",
    "reasoner.OWLLinkTest.testUpdatesBuffered": "owllink_update_hierarchy_buffered",
    "reasoner.OWLLinkTest.testUpdatesNonBuffered": "owllink_update_hierarchy_non_buffered",
    "reasoner.ClassificationTest.testPizza": "hermit_classification_pizza_taxonomy",
    "reasoner.ClassificationTest.testWine": "hermit_classification_wine_taxonomy",
}

EXCLUDED_IDS = {
    "reasoner.ReasonerTest.testSubProperties",
    "reasoner.ReasonerTest.testObjectPropertyHierarchy",
    "reasoner.ReasonerTest.testIsSymmetricObject",
    "reasoner.ReasonerTest.testIsTransitiveObject",
}

# OFN extracts that fail load_ontology (punning / inverse CE) — keep out of axioms/.
OFN_WRITE_SKIP_IDS = {
    "reasoner.ReasonerTest.testPunning",
    "reasoner.ReasonerTest.testPunning2",
    "reasoner.ReasonerTest.testPunning3",
    "reasoner.ReasonerTest.testInverses",
}

# DL axiom ports gated on tableau maturity (Phase 2+).
DEFERRED_DL_AXIOM_IDS: set[str] = set()

# RL/RDFS axiom ports extracted but not yet passing in ontologos.
DEFERRED_RL_AXIOM_IDS = {
    "reasoner.OWLReasonerTest.testIncrementalAddition2",
}

# ReasonerTest cases that pass via RL engine, not DL tableau.
FORCE_RL_ENGINE_IDS = {
    "reasoner.ReasonerTest.testSubsumption2",
    "reasoner.ReasonerTest.testSubsumption3",
}

# RL/RDFS consistency cases verified passing — promote to axiom.
APPROVED_RL_CONSISTENCY_IDS = {
    "reasoner.ReasonerTest.testBottomObjectPropertyAssertion",
}

DEFERRED_PREFIXES = ("reasoner.RulesTest",)

# HermiT engine-internal tests ported to engine unit tests (permanent conformance ignore).
MIGRATED_INTERNAL_IDS: set[str] = {
    "structural.NormalizationTest.testDataPropertiesAll1",
    "structural.NormalizationTest.testDataPropertiesAll2",
    "structural.NormalizationTest.testDataPropertiesHasValue1",
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
    consistent: bool | None = None
    rust_test: str | None = None
    hand_written: bool = False


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
    if case.id in MIGRATED_INTERNAL_IDS:
        case.status = "migrated"
        case.ignore_reason = "ported to ontologos-alc/dl unit tests"
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
    if case.id in EXCLUDED_IDS:
        case.status = "excluded"
        case.ignore_reason = "documented semantic or mapping gap (see manifest)"
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
            case.status = "axiom"
            case.tier = "A"
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
        case.status = "axiom"
        case.tier = "A"
        return
    if case.axiom_ofn and case.property_subsumptions and case.engine in ("rdfs", "rl"):
        case.status = "axiom"
        case.tier = "A"
        return
    if case.axiom_ofn and case.consistent is not None and case.engine in ("dl", "alc"):
        case.status = "planned"
        case.ignore_reason = "DL axiom fixture; consistency assertions pending engine (Phase 2+)"
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
    if case.axiom_ofn and case.engine in ("dl", "alc") and not case.subsumptions and case.consistent is None:
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
    return (
        "\n".join(prefixes)
        + "\n"
        + "Ontology(<file:/c/test.owl#>\n"
        + f"{axioms}\n"
        + ")\n"
    )


def rust_fn_name(case_id: str) -> str:
    return "hermit_" + re.sub(r"[^a-zA-Z0-9]+", "_", case_id).strip("_").lower()


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
                safe = rust_fn_name(case_id)
                ofn_rel = f"axioms/{safe}.ofn"
                if axioms and valid_ofn_axioms(axioms):
                    case.axiom_ofn = ofn_rel
                elif (OUT_AXIOMS.parent / ofn_rel).is_file():
                    case.axiom_ofn = ofn_rel

            case.subsumptions = filter_java_ce_subsumptions(extract_subsumptions(body))
            case.subsumptions.extend(
                filter_java_ce_subsumptions(extract_entailment_subsumptions(body))
            )
            if not case.subsumptions and case.id in HARDCODED_AXIOM_SUBSUMPTIONS:
                case.subsumptions = HARDCODED_AXIOM_SUBSUMPTIONS[case.id]
            case.property_subsumptions = extract_property_subsumptions(body)
            case.property_characteristics = extract_property_characteristics(body)
            case.consistent = extract_consistency(body)
            if case.id in INCREMENTAL_CONSISTENCY_IDS:
                case.consistent = None
            infer_status(case)
            cases.append(case)
    return cases


def write_axioms(cases: list[HermitCase]) -> int:
    OUT_AXIOMS.mkdir(parents=True, exist_ok=True)
    expected: set[Path] = set()
    for case in cases:
        if case.axiom_ofn:
            expected.add(OUT_AXIOMS.parent / case.axiom_ofn)
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
        if not axioms or not valid_ofn_axioms(axioms):
            continue
        out = OUT_AXIOMS.parent / case.axiom_ofn
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(wrap_ofn(axioms), encoding="utf-8")
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


def extract_wg_embedded_rdf(block: str, tag: str) -> str | None:
    m = re.search(
        rf"<test:{tag}[^>]*>(.*?)</test:{tag}>",
        block,
        re.DOTALL,
    )
    if not m:
        return None
    raw = html.unescape(m.group(1).strip())
    return raw if raw.startswith("<") else None


def write_wg_fixture(test_id: str, block: str) -> tuple[str | None, str | None]:
    premise = extract_wg_embedded_rdf(block, "rdfXmlPremiseOntology")
    conclusion = extract_wg_embedded_rdf(block, "rdfXmlConclusionOntology")
    if not premise:
        return None, None
    safe = re.sub(r"[^a-zA-Z0-9._-]+", "_", test_id)
    out_dir = OUT_WG_DATA / safe
    out_dir.mkdir(parents=True, exist_ok=True)
    prem_path = out_dir / "premise.rdf"
    prem_path.write_text(premise, encoding="utf-8")
    prem_rel = f"wg/{safe}/premise.rdf"
    conc_rel = None
    if conclusion:
        conc_path = out_dir / "conclusion.rdf"
        conc_path.write_text(conclusion, encoding="utf-8")
        conc_rel = f"wg/{safe}/conclusion.rdf"
    return prem_rel, conc_rel


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
    """Catalog OWL WG approved DL tests from all.rdf (stub entries until OFN extraction)."""
    all_rdf = RES_ROOT / "owl_wg_tests/ontologies/all.rdf"
    if not all_rdf.is_file():
        return []
    text = all_rdf.read_text(errors="replace")
    cases: list[WgCase] = []
    # Each TestCase individual id in the WG export.
    for m in re.finditer(
        r'rdf:about="(http://owl\.semanticweb\.org/id/[^"]+)"',
        text,
    ):
        test_id = m.group(1).rsplit("/", 1)[-1]
        block_start = m.start()
        block = text[block_start : block_start + 8000]
        if "Approved" not in block or "DL" not in block:
            continue
        test_type = "entailment"
        expected_entailment = None
        expected_consistent = None
        if "PositiveEntailmentTest" in block:
            test_type = "positive_entailment"
            expected_entailment = True
        elif "NegativeEntailmentTest" in block:
            test_type = "negative_entailment"
            expected_entailment = False
        elif "ConsistencyTest" in block:
            test_type = "consistency"
            expected_consistent = True
        elif "InconsistencyTest" in block:
            test_type = "inconsistency"
            expected_consistent = False
        if test_id in WG_CONSISTENCY_OVERRIDES:
            test_type, expected_consistent = WG_CONSISTENCY_OVERRIDES[test_id]
            expected_entailment = None
            conclusion_ofn = None
        premise_ofn = None
        conclusion_ofn = None
        status = "planned"
        ignore_reason = "WG test — requires ontologos-dl + vendored WG OFN fixtures"
        if "PositiveEntailmentTest" in block:
            prem_rel, conc_rel = write_wg_fixture(test_id, block)
            premise_ofn = prem_rel
            conclusion_ofn = conc_rel
        elif expected_consistent is not None:
            prem_rel, _ = write_wg_fixture(test_id, block)
            premise_ofn = prem_rel
        if (
            test_id in PROMOTED_WG_IDS
            and premise_ofn
            and (conclusion_ofn is not None or expected_consistent is not None)
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
        prem = OUT_WG_DATA / test_id / "premise.rdf"
        conc = OUT_WG_DATA / test_id / "conclusion.rdf"
        if prem.is_file():
            row["premise_ofn"] = f"wg/{test_id}/premise.rdf"
            override = WG_CONSISTENCY_OVERRIDES.get(test_id)
            if override is not None:
                test_type, expected = override
                row["test_type"] = test_type
                row["expected_consistent"] = expected
                row["expected_entailment"] = None
                row["conclusion_ofn"] = None
            elif conc.is_file():
                row["conclusion_ofn"] = f"wg/{test_id}/conclusion.rdf"
            if test_id in PROMOTED_WG_IDS:
                row["status"] = "wg"
                row["ignore_reason"] = None
                active += 1
            elif row.get("conclusion_ofn") or row.get("expected_consistent") is not None:
                row["status"] = "planned"
                row["ignore_reason"] = "WG test — pending ontologos-dl entailment promotion"
        updated.append(row)
    wg_path.write_text(json.dumps(updated, indent=2) + "\n", encoding="utf-8")
    cases = [WgCase(**row) for row in updated]
    write_wg_rust(cases)
    print(f"WG promote-only: {active} active of {len(updated)}")


def promote_only_from_disk() -> None:
    """Re-apply promoted_axiom_ids.txt to existing cases.json without HermiT checkout."""
    global PROMOTED_AXIOM_IDS
    PROMOTED_AXIOM_IDS = load_promoted_axiom_ids()
    catalog_path = OUT_CATALOG / "cases.json"
    raw = json.loads(catalog_path.read_text(encoding="utf-8"))
    cases: list[HermitCase] = []
    for row in raw:
        case = HermitCase(**row)
        if case.id in INCREMENTAL_CONSISTENCY_IDS:
            case.consistent = None
        infer_status(case)
        cases.append(case)
    catalog_path.write_text(
        json.dumps([asdict(c) for c in cases], indent=2) + "\n", encoding="utf-8"
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
    global PROMOTED_WG_IDS
    PROMOTED_WG_IDS = load_promoted_wg_ids()
    wg_cases = collect_wg_cases()
    OUT_CATALOG.mkdir(parents=True, exist_ok=True)
    wg_path = OUT_WG_CATALOG
    wg_path.write_text(
        json.dumps([asdict(c) for c in wg_cases], indent=2) + "\n", encoding="utf-8"
    )
    write_wg_rust(wg_cases)
    active = sum(1 for c in wg_cases if c.status == "wg")
    print(f"WG catalog refresh: {len(wg_cases)} cases, {active} active")
    print(f"  wrote {wg_path.relative_to(REPO)}")
    print(f"  wrote {OUT_WG_RUST.relative_to(REPO)}")


def main() -> None:
    if len(sys.argv) > 1 and sys.argv[1] == "--promote-only":
        promote_only_from_disk()
        return
    if len(sys.argv) > 1 and sys.argv[1] == "--promote-wg-only":
        promote_wg_from_disk()
        return
    if len(sys.argv) > 1 and sys.argv[1] == "--wg-catalog-only":
        wg_catalog_only()
        return
    cases = collect_cases()
    wg_cases = collect_wg_cases()
    OUT_CATALOG.mkdir(parents=True, exist_ok=True)
    n_axiom = write_axioms(cases)
    catalog_path = OUT_CATALOG / "cases.json"
    catalog_path.write_text(
        json.dumps([asdict(c) for c in cases], indent=2) + "\n", encoding="utf-8"
    )
    wg_path = OUT_WG_CATALOG
    wg_path.write_text(
        json.dumps([asdict(c) for c in wg_cases], indent=2) + "\n", encoding="utf-8"
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
