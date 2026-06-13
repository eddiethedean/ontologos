#!/usr/bin/env python3
"""Generate HermiT port catalog (JSON + Rust tests) from a local hermit-reasoner checkout.

Run from repo root:
  python3 tests/hermit/generate_catalog.py

Requires HermiT/ (owlcs/hermit-reasoner) or ONTOLOGOS_HERMIT_ROOT.
"""

from __future__ import annotations

import json
import os
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERMIT = Path(os.environ.get("ONTOLOGOS_HERMIT_ROOT", REPO / "HermiT"))
JAVA_ROOT = HERMIT / "src/test/java/org/semanticweb/HermiT"
RES_ROOT = HERMIT / "src/test/resources/org/semanticweb/HermiT"
OUT_CATALOG = REPO / "benchmarks/data/hermit/catalog"
OUT_AXIOMS = REPO / "benchmarks/data/hermit/axioms"
OUT_RUST = REPO / "crates/ontologos-conformance/tests/hermit_generated.rs"
OUT_WG_RUST = REPO / "crates/ontologos-conformance/tests/hermit_wg_generated.rs"
OUT_WG_CATALOG = REPO / "benchmarks/data/hermit/catalog/wg_cases.json"

SKIP_FILE = re.compile(
    r"(Abstract|AllTests|AllQuick|Descriptor|Registry|Invalid|Failing|TstDescriptor|AllWG|AllApproved|AllExtracredit|AllNonRejected|AllProposed)"
)

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

# DL axiom ports gated on tableau maturity (Phase 2+).
DEFERRED_DL_AXIOM_IDS = {
    "reasoner.ReasonerTest.testClassificationSubClassBug",
    "reasoner.ReasonerTest.testSatisfiabilityWithRIAs14",
}

# RL/RDFS axiom ports extracted but not yet passing in ontologos.
DEFERRED_RL_AXIOM_IDS = {
    "reasoner.OWLReasonerTest.testIncrementalAddition2",
    "reasoner.RIATest.testInverseAndChain",
    "reasoner.ReasonerTest.testBottomObjectPropertyAssertion",
    "reasoner.ReasonerTest.testIsInverseFunctionalObject",
    "reasoner.ReasonerTest.testIsIrreflexiveObject",
}

DEFERRED_PREFIXES = ("reasoner.RulesTest",)

INTERNAL_PREFIXES = (
    "tableau.",
    "structural.",
    "graph.",
    "rationals.",
)

# RDF/XML fixtures that horned-owl cannot parse yet (DOCTYPE entities, duplicate rdf:ID).
PARSER_IGNORE_FIXTURES = {
    "res/wine.xml",
    "res/galen-ians-full-undoctored.xml",
    "res/propreo.xml",
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
    for m in re.finditer(
        r'assertSubsumedBy\s*\(\s*(?:"([^"]+)"|NS_C\s*\(\s*"([^"]+)"\s*\))\s*,\s*(?:"([^"]+)"|NS_C\s*\(\s*"([^"]+)"\s*\))\s*,\s*(true|false)\s*\)',
        body,
    ):
        sub = m.group(1) or m.group(2) or ""
        sup = m.group(3) or m.group(4) or ""
        expected = m.group(5) == "true"
        key = (sub, sup, expected)
        if key in seen:
            continue
        seen.add(key)
        subs.append({"sub": sub, "sup": sup, "expected": expected})
    return subs


def extract_consistency(body: str) -> bool | None:
    if re.search(r"assertConsistent\s*\(\s*\)", body):
        return True
    if re.search(r"assertNotConsistent\s*\(\s*\)", body):
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
    """Concatenate `String axioms = ...` fragments up to the load call."""
    start_m = re.search(r"\bString\s+axioms\s*=", body)
    if not start_m:
        return ""
    rest = body[start_m.start() :]
    end_m = re.search(r"load(?:Reasoner|Ontology)WithAxioms\s*\(\s*axioms\s*\)", rest)
    if not end_m:
        return ""
    chunk = rest[: end_m.start()]
    # Java constant concatenation (e.g. + NS +) cannot be resolved to OFN.
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
    if case_id.startswith(INTERNAL_PREFIXES):
        return "internal"
    if case_id.startswith("owl_wg_tests."):
        return "dl"
    if case_id.startswith(DEFERRED_PREFIXES):
        return "swrl"
    if "ClassificationTest" in case_id:
        return "el"
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
    if case.id in IMPLEMENTED:
        case.status = "ported"
        case.rust_test = IMPLEMENTED[case.id]
        case.hand_written = True
        return
    if case.id.startswith(DEFERRED_PREFIXES):
        case.status = "planned"
        case.ignore_reason = "SWRL — requires ontologos-swrl (1.0)"
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
            case.status = "axiom"
            case.tier = "A"
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
        case.status = "axiom"
        case.tier = "A"
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
                if axioms and valid_ofn_axioms(axioms):
                    safe = rust_fn_name(case_id)
                    case.axiom_ofn = f"axioms/{safe}.ofn"

            case.subsumptions = extract_subsumptions(body)
            case.property_subsumptions = extract_property_subsumptions(body)
            case.property_characteristics = extract_property_characteristics(body)
            case.consistent = extract_consistency(body)
            infer_status(case)
            cases.append(case)
    return cases


def write_axioms(cases: list[HermitCase]) -> int:
    OUT_AXIOMS.mkdir(parents=True, exist_ok=True)
    expected: set[Path] = set()
    for case in cases:
        if case.axiom_ofn:
            expected.add(OUT_AXIOMS.parent / case.axiom_ofn)
    for stale in OUT_AXIOMS.glob("*.ofn"):
        if stale not in expected:
            stale.unlink()
    written = 0
    for case in cases:
        if not case.axiom_ofn:
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
        elif case.status in ("excluded", "deferred", "internal", "planned", "swrl") and case.ignore_reason:
            reason = case.ignore_reason.replace('"', '\\"')
            lines.append(f"#[test]")
            lines.append(f'#[ignore = "{reason}"]')
        elif case.status == "swrl":
            lines.append(f"#[test]")
            lines.append(f'#[ignore = "SWRL — requires ontologos-swrl engine"]')
        elif case.status == "fixture":
            lines.append(f"#[test]")
            if case.fixture in PARSER_IGNORE_FIXTURES:
                lines.append(
                    '#[ignore = "RDF/XML fixture not supported by parser yet (entities or duplicate rdf:ID)"]'
                )
            elif case.id == "reasoner.ClassificationTest.testWine":
                lines.append(
                    '#[ignore = "wine.xml fails to parse (duplicate rdf:ID)"]'
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
        cases.append(
            WgCase(
                id=f"owl_wg_tests.{test_id}",
                test_type=test_type,
                status="planned",
                engine="dl",
                expected_entailment=expected_entailment,
                expected_consistent=expected_consistent,
                ignore_reason="WG test — requires ontologos-dl + vendored WG OFN fixtures",
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


def main() -> None:
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
