"""HermiT Java test assertion harvest for generate_catalog.py (Phase 2)."""

from __future__ import annotations

import re
from typing import Any

HERMIT_NS = "file:/c/test.owl#"
LB = '"+LB+'


def normalize_class_name(raw: str) -> str:
    raw = raw.strip()
    if raw.startswith(":"):
        return raw
    if raw.startswith("file:") and "#" in raw:
        return ":" + raw.rsplit("#", 1)[-1]
    if re.fullmatch(r"[\w.-]+", raw):
        return f":{raw}"
    return raw


def normalize_individual(raw: str) -> str:
    return normalize_class_name(raw)


def extract_assert_satisfiable(body: str) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    seen: set[tuple[str, bool]] = set()

    def add(cls: str, expected: bool) -> None:
        cls = normalize_class_name(cls)
        if not cls.startswith(":"):
            return
        key = (cls, expected)
        if key in seen:
            return
        seen.add(key)
        out.append({"class": cls, "expected": expected})

    for m in re.finditer(
        r'assertSatisfiable\s*\(\s*"([^"]+)"\s*,\s*(true|false)\s*\)',
        body,
    ):
        add(m.group(1), m.group(2) == "true")
    for m in re.finditer(
        r"assertSatisfiable\s*\(\s*NS_C\s*\(\s*\"([^\"]+)\"\s*\)\s*,\s*(true|false)\s*\)",
        body,
    ):
        add(f":{m.group(1)}", m.group(2) == "true")
    for m in re.finditer(
        r'assertSatisfiable\s*\(\s*"([^"]+)"\s*,\s*(true|false)\s*\)',
        body,
    ):
        add(m.group(1), m.group(2) == "true")
    for m in re.finditer(
        r"assertSatisfiable\s*\(\s*([A-Za-z_][\w]*)\s*,\s*(true|false)\s*\)",
        body,
    ):
        var = m.group(1)
        if var in {"desc", "desc1", "desc2", "and", "or", "clazz", "concept"}:
            continue
        add(var, m.group(2) == "true")
    for m in re.finditer(
        r"assert(False|True)\s*\(\s*m_reasoner\.isSatisfiable\s*\(\s*([A-Za-z_][\w]*)\s*\)\s*\)",
        body,
    ):
        expected = m.group(1) == "True"
        add(m.group(2), expected)
    return out


def extract_conclusion_axioms(body: str) -> str:
    """OFN conclusion fragment before assertEntails / getOntologyWithAxioms."""
    if "assertEntails" not in body and "getOntologyWithAxioms" not in body:
        return ""
    rest = body
    for m in re.finditer(
        r'\bString\s+axioms\s*=\s*((?:"[^"\\]*(?:\\.[^"\\]*)*"\s*)+)',
        body,
    ):
        candidate = _decode_java_string_concat(m.group(1))
        if candidate and _looks_like_ofn_axioms(candidate):
            rest_assign = body[m.start() :]
            if "getOntologyWithAxioms" in rest_assign or "assertEntails" in rest_assign:
                return candidate
    assign_m = re.search(
        r"getOntologyWithAxioms\s*\(\s*axioms\s*\)",
        body,
    )
    if assign_m:
        before = body[: assign_m.start()]
        m = re.search(
            r'\baxioms\s*=\s*((?:"[^"\\]*(?:\\.[^"\\]*)*"\s*)+)\s*;',
            before,
        )
        if m:
            return _decode_java_string_concat(m.group(1))
    load_m = re.search(r"load(?:Reasoner|Ontology)WithAxioms\s*\(\s*axioms\s*\)", body)
    if load_m:
        rest = body[load_m.end() :]
        m = re.search(
            r'\baxioms\s*=\s*((?:"[^"\\]*(?:\\.[^"\\]*)*"\s*)+)',
            rest,
        )
        if m:
            return _decode_java_string_concat(m.group(1))
    return ""


def _decode_java_string_concat(chunk: str) -> str:
    parts = re.findall(r'"([^"\\]*(?:\\.[^"\\]*)*)"', chunk)
    raw = "".join(parts)
    return bytes(raw, "utf-8").decode("unicode_escape")


def _looks_like_ofn_axioms(s: str) -> bool:
    s = s.strip()
    if not s:
        return False
    return any(
        kw in s
        for kw in (
            "SubClassOf",
            "ClassAssertion",
            "ObjectPropertyAssertion",
            "DataPropertyAssertion",
            "HasKey",
            "EquivalentClasses",
        )
    )


def extract_entailment_metadata(body: str) -> tuple[str | None, bool | None]:
    ent_m = re.search(r"assertEntails\s*\([^,]+,\s*(true|false)\s*\)", body)
    if not ent_m:
        return None, None
    expected = ent_m.group(1) == "true"
    conclusion = extract_conclusion_axioms(body)
    if not conclusion:
        return None, None
    return conclusion, expected


def extract_property_hierarchy(body: str) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    """Object and data property subsumptions from HermiT helper patterns."""
    obj_out: list[dict[str, Any]] = []
    data_out: list[dict[str, Any]] = []
    seen: set[tuple[str, str, str, bool]] = set()

    def add_obj(sub: str, sup: str, expected: bool) -> None:
        sub = normalize_class_name(sub).lstrip(":")
        sup = normalize_class_name(sup).lstrip(":")
        if not sub or not sup:
            return
        key = ("obj", sub, sup, expected)
        if key in seen:
            return
        seen.add(key)
        obj_out.append({"sub": f":{sub}", "sup": f":{sup}", "expected": expected})

    def add_data(sub: str, sup: str, expected: bool) -> None:
        sub = normalize_class_name(sub).lstrip(":")
        sup = normalize_class_name(sup).lstrip(":")
        key = ("data", sub, sup, expected)
        if key in seen:
            return
        seen.add(key)
        data_out.append({"sub": f":{sub}", "sup": f":{sup}", "expected": expected})

    for m in re.finditer(
        r'assertDirectSuperObjectProperties\s*\(\s*"([^"]+)"\s*,\s*EQ\s*\(\s*"([^"]+)"\s*\)\s*\)',
        body,
    ):
        add_obj(m.group(1), m.group(2), True)
    for m in re.finditer(
        r'assertDirectSuperDataProperties\s*\(\s*"([^"]+)"\s*,\s*EQ\s*\(\s*"([^"]+)"\s*\)\s*\)',
        body,
    ):
        add_data(m.group(1), m.group(2), True)
    for m in re.finditer(
        r"assert(True|False)\s*\(\s*m_reasoner\.getSuperObjectProperties\s*\(\s*NS_OP\s*\(\s*\"([^\"]+)\"\s*\)\s*,\s*true\s*\)\.getFlattened\(\)\.contains\s*\(\s*NS_OP\s*\(\s*\"([^\"]+)\"\s*\)\s*\)\s*\)",
        body,
    ):
        add_obj(m.group(3), m.group(2), m.group(1) == "True")
    for m in re.finditer(
        r"assert(True|False)\s*\(\s*m_reasoner\.getSubDataProperties\s*\(\s*(\w+)\s*,\s*false\s*\)\.containsEntity\s*\(\s*(\w+)\s*\)\s*\)",
        body,
    ):
        add_data(m.group(3), m.group(2), m.group(1) == "True")
    return obj_out, data_out


def extract_incremental_ofn(body: str) -> str:
    """OFN from addAxioms string literal or factory-built set."""
    m = re.search(
        r'addAxioms\s*\(\s*m_ontology\s*,\s*((?:"[^"\\]*(?:\\.[^"\\]*)*"\s*)+)\s*\)',
        body,
    )
    if m:
        return _decode_java_string_concat(m.group(1))
    m = re.search(
        r'addAxioms\s*\(\s*m_ontology\s*,\s*Collections\.singleton\s*\(\s*m_dataFactory\.getOWLSubClassOfAxiom\s*\([^)]+\)\s*\)\s*\)',
        body,
    )
    if m:
        return ""
    chunks: list[str] = []
    for m in re.finditer(
        r"m_dataFactory\.getOWLClassAssertionAxiom\s*\(\s*NS_C\s*\(\s*\"([^\"]+)\"\s*\)\s*,\s*NS_NI\s*\(\s*\"([^\"]+)\"\s*\)\s*\)",
        body,
    ):
        chunks.append(f"ClassAssertion(:{m.group(1)} :{m.group(2)})")
    for m in re.finditer(
        r"assertions\.add\s*\(\s*m_dataFactory\.getOWLClassAssertionAxiom\s*\(\s*NS_C\s*\(\s*\"([^\"]+)\"\s*\)\s*,\s*NS_NI\s*\(\s*\"([^\"]+)\"\s*\)\s*\)\s*\)",
        body,
    ):
        chunks.append(f"ClassAssertion(:{m.group(1)} :{m.group(2)})")
    return "".join(chunks)


def extract_individual_types(body: str) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    seen: set[tuple[str, str, bool, bool]] = set()

    def add(ind: str, cls: str, expected: bool, direct: bool) -> None:
        ind = normalize_individual(ind)
        cls = normalize_class_name(cls)
        if not ind.startswith(":") or not cls.startswith(":"):
            return
        key = (ind, cls, expected, direct)
        if key in seen:
            return
        seen.add(key)
        out.append(
            {
                "individual": ind,
                "class": cls,
                "expected": expected,
                "direct": direct,
            }
        )

    for m in re.finditer(
        r"assert(True|False)\s*\(\s*m_reasoner\.hasType\s*\(\s*NS_NI\s*\(\s*\"([^\"]+)\"\s*\)\s*,\s*NS_C\s*\(\s*\"([^\"]+)\"\s*\)\s*,\s*(true|false)\s*\)\s*\)",
        body,
    ):
        add(f":{m.group(2)}", f":{m.group(3)}", m.group(1) == "True", m.group(4) == "true")
    get_types = re.search(r"getTypes\s*\(\s*(\w+)\s*,\s*true\s*\)", body)
    if get_types:
        ind_var = get_types.group(1)
        ind_m = re.search(
            rf"{re.escape(ind_var)}\s*=\s*NS_NI\s*\(\s*\"([^\"]+)\"\s*\)",
            body,
        )
        for m in re.finditer(
            r"assertTrue\s*\(\s*contains\s*\(\s*result\.entities\s*\(\s*\)\s*,\s*(\w+)\s*\)\s*\)",
            body,
        ):
            cls_var = m.group(1)
            cls_m = re.search(
                rf"{re.escape(cls_var)}\s*=\s*NS_C\s*\(\s*\"([^\"]+)\"\s*\)",
                body,
            )
            if ind_m and cls_m:
                add(f":{ind_m.group(1)}", f":{cls_m.group(1)}", True, True)
    for m in re.finditer(
        r"assertInstanceOf\s*\(\s*([A-Za-z_][\w]*)\s*,\s*([A-Za-z_][\w]*)\s*,\s*(true|false)\s*\)",
        body,
    ):
        add(m.group(2), m.group(1), m.group(3) == "true", False)
    return out


def extract_individual_instances(body: str) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for m in re.finditer(
        r"assertInstancesOf\s*\(\s*([A-Za-z_][\w]*)\s*,\s*(true|false)\s*,\s*IRIs\s*\(\s*([^)]*)\s*\)\s*\)",
        body,
    ):
        cls_var = m.group(1)
        direct = m.group(2) == "true"
        inds = [
            normalize_individual(x.strip().strip('"'))
            for x in m.group(3).split(",")
            if x.strip()
        ]
        out.append(
            {
                "class": f":{cls_var}" if not cls_var.startswith(":") else cls_var,
                "expected_individuals": inds,
                "direct": direct,
            }
        )
    return out


def extract_datalog_queries(body: str) -> list[dict[str, Any]]:
    """Parse ConjunctiveQuery + QueryChecker chains from DatalogEngineTest."""
    queries: list[dict[str, Any]] = []
    parts = re.split(r"new\s+ConjunctiveQuery\s*\(", body)
    for part in parts[1:]:
        atom_m = re.search(
            r"AS\s*\(\s*(.*?)\s*\)\s*,\s*TS\s*\(",
            part,
            re.DOTALL,
        )
        if not atom_m:
            continue
        atoms_block = atom_m.group(1)
        atoms: list[dict[str, str]] = []
        for am in re.finditer(
            r'A\s*\(\s*CN\s*\(\s*"([^"]+)"\s*\)\s*,\s*V\s*\(\s*"([^"]+)"\s*\)\s*\)',
            atoms_block,
        ):
            atoms.append(
                {
                    "kind": "class",
                    "class": f":{am.group(1)}",
                    "variable": am.group(2),
                }
            )
        for am in re.finditer(
            r'A\s*\(\s*R\s*\(\s*"([^"]+)"\s*\)\s*,\s*V\s*\(\s*"([^"]+)"\s*\)\s*,\s*V\s*\(\s*"([^"]+)"\s*\)\s*\)',
            atoms_block,
        ):
            atoms.append(
                {
                    "kind": "role",
                    "role": f":{am.group(1)}",
                    "variable": am.group(2),
                    "variable2": am.group(3),
                }
            )
        for am in re.finditer(
            r'A\s*\(\s*R\s*\(\s*"([^"]+)"\s*\)\s*,\s*V\s*\(\s*"([^"]+)"\s*\)\s*,\s*I\s*\(\s*"([^"]+)"\s*\)\s*\)',
            atoms_block,
        ):
            atoms.append(
                {
                    "kind": "role_individual",
                    "role": f":{am.group(1)}",
                    "variable": am.group(2),
                    "individual": f":{am.group(3)}",
                }
            )
        for am in re.finditer(
            r'A\s*\(\s*R\s*\(\s*"([^"]+)"\s*\)\s*,\s*V\s*\(\s*"([^"]+)"\s*\)\s*,\s*I\s*\(\s*"([^"]+)"\s*\)\s*\)\s*,\s*'
            r'A\s*\(\s*R\s*\(\s*"([^"]+)"\s*\)\s*,\s*V\s*\(\s*"([^"]+)"\s*\)\s*,\s*I\s*\(\s*"([^"]+)"\s*\)\s*\)',
            atoms_block,
        ):
            atoms.append(
                {
                    "kind": "role_individual",
                    "role": f":{am.group(1)}",
                    "variable": am.group(2),
                    "individual": f":{am.group(3)}",
                }
            )
            atoms.append(
                {
                    "kind": "role_individual",
                    "role": f":{am.group(4)}",
                    "variable": am.group(5),
                    "individual": f":{am.group(6)}",
                }
            )
        after = part[atom_m.end() :]
        end_m = re.search(r"assertEquals\s*\(\s*\)", after)
        checker_region = after[: end_m.start()] if end_m else after[:800]
        answers: list[str] = []
        for im in re.finditer(r'\.add\s*\(\s*I\s*\(\s*"([^"]+)"\s*\)', checker_region):
            answers.append(f":{im.group(1)}")
        if atoms:
            queries.append({"atoms": atoms, "answers": answers})
    return queries


def extract_equivalence_subsumptions(body: str) -> list[dict[str, Any]]:
    """Map assertEquivalentClasses to mutual subsumptions."""
    subs: list[dict[str, Any]] = []
    for m in re.finditer(
        r"assertEquivalentClasses\s*\(\s*NS_C\s*\(\s*\"([^\"]+)\"\s*\)\s*,\s*NS_C\s*\(\s*\"([^\"]+)\"\s*\)\s*\)",
        body,
    ):
        a, b = f":{m.group(1)}", f":{m.group(2)}"
        subs.append({"sub": a, "sup": b, "expected": True})
        subs.append({"sub": b, "sup": a, "expected": True})
    return subs


def extract_load_error_expected(body: str) -> bool:
    if re.search(r"loadReasonerWithAxioms\s*\(\s*axioms\s*\)\s*;\s*\n\s*fail\s*\(\s*\)", body):
        return True
    if re.search(r"try\s*\{[^}]*loadReasonerWithAxioms[^}]*fail\s*\(\s*\)", body, re.DOTALL):
        return True
    return False


def extract_ce_satisfiability_fallback(body: str) -> list[dict[str, Any]]:
    """When assertSatisfiable uses a Java CE variable, bind to first named class in the test."""
    out: list[dict[str, Any]] = []
    m = re.search(
        r"assertSatisfiable\s*\(\s*(desc\d*|desc)\s*,\s*(true|false)\s*\)",
        body,
    )
    if not m:
        return out
    expected = m.group(2) == "true"
    names = re.findall(r"file:/c/test\.owl#([\w-]+)", body)
    if not names:
        names = re.findall(r":([\w-]+)", body)
        names = [n for n in names if n not in ("owl", "xsd", "rdf", "rdfs")]
    if not names:
        names = ["test"]
    cls = normalize_class_name(names[0])
    out.append({"class": cls, "expected": expected})
    return out


def extract_has_key_entailment(body: str) -> tuple[str | None, bool | None]:
    m = re.search(
        r"assertEntails\s*\(\s*m_dataFactory\.getOWLHasKeyAxiom\s*\(\s*[^,]+,\s*[^)]+\)\s*,\s*(true|false)\s*\)",
        body,
    )
    if not m:
        return None, None
    expected = m.group(1) == "true"
    cm = re.search(
        r"getOWLClass\s*\(\s*IRI\.create\([^+]*\+\s*\"(\w+)\"\s*\)\s*\)",
        body,
    )
    pm = re.search(
        r"getOWLObjectProperty\s*\(\s*IRI\.create\([^+]*\+\s*\"(\w+)\"\s*\)\s*\)",
        body,
    )
    if cm and pm:
        conclusion = f"HasKey( :{cm.group(1)} () ( :{pm.group(1)} ) )"
        return conclusion, expected
    return None, None


def extract_top_op_equivalence(body: str) -> list[dict[str, Any]]:
    if "getEquivalentObjectProperties" not in body:
        return []
    m = re.search(r'NS_OP\s*\(\s*"([^"]+)"\s*\)', body)
    if not m:
        return []
    prop = f":{m.group(1)}"
    return [
        {"sub": prop, "sup": "owl:topObjectProperty", "expected": True},
        {"sub": "owl:topObjectProperty", "sup": prop, "expected": True},
    ]


def extract_entailment_checker_fail(body: str) -> tuple[str | None, bool | None]:
    """EntailmentTest patterns where EntailmentChecker.entails must fail."""
    if "EntailmentChecker" not in body or "fail()" not in body:
        return None, None
    conclusion = extract_conclusion_axioms(body)
    if conclusion:
        return conclusion, False
    return None, None


def extract_instance_retrieval(body: str) -> list[dict[str, Any]]:
    """assertInstancesOf and getObjectPropertyValues patterns."""
    out = extract_individual_instances(body)
    for m in re.finditer(
        r"assertTrue\s*\(\s*m_reasoner\.getObjectPropertyValues\s*\(\s*NS_NI\s*\(\s*\"([^\"]+)\"\s*\)\s*,\s*NS_OP\s*\(\s*\"([^\"]+)\"\s*\)\s*\)\.containsEntity\s*\(\s*NS_NI\s*\(\s*\"([^\"]+)\"\s*\)\s*\)\s*\)",
        body,
    ):
        out.append(
            {
                "class": f":{m.group(2)}",
                "expected_individuals": [f":{m.group(3)}"],
                "direct": False,
            }
        )
    return out


def extract_equivalent_properties(body: str) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    obj: list[dict[str, Any]] = []
    data: list[dict[str, Any]] = []
    for m in re.finditer(
        r'assertEquivalentObjectProperties\s*\(\s*"([^"]+)"\s*,\s*IRIs\s*\(([^)]*)\)\s*\)',
        body,
    ):
        base = f":{m.group(1)}"
        others = [
            f":{x.strip().strip(chr(34))}" for x in m.group(2).split(",") if x.strip()
        ]
        for other in others:
            if other != base:
                obj.append({"sub": base, "sup": other, "expected": True})
                obj.append({"sub": other, "sup": base, "expected": True})
    for m in re.finditer(
        r'assertEquivalentDataProperties\s*\(\s*"([^"]+)"\s*,\s*IRIs\s*\(([^)]*)\)\s*\)',
        body,
    ):
        base = f":{m.group(1)}"
        others = [
            f":{x.strip().strip(chr(34))}" for x in m.group(2).split(",") if x.strip()
        ]
        for other in others:
            if other != base:
                data.append({"sub": base, "sup": other, "expected": True})
                data.append({"sub": other, "sup": base, "expected": True})
    return obj, data


def extract_datatype_def_entailment(body: str) -> tuple[str | None, bool | None]:
    if "assertEntails(ddef" not in body and "getOWLDatatypeDefinitionAxiom" not in body:
        return None, None
    m = re.search(r'DatatypeDefinition\s*\([^)]+\)', body)
    if m:
        return m.group(0), True
    return None, None


def extract_subproperty_chain_entailment(body: str) -> tuple[str | None, bool | None]:
    m = re.search(r"assertEntails\s*\(\s*ax\s*,\s*(true|false)\s*\)", body)
    if not m:
        return None, None
    expected = m.group(1) == "true"
    if "SubPropertyChainOfAxiom" in body or "ObjectPropertyChain" in body:
        cm = re.search(
            r"SubObjectPropertyOf\s*\(\s*ObjectPropertyChain\s*\(([^)]+)\)\s*(:[\w]+)\s*\)",
            body,
        )
        if cm:
            return f"SubObjectPropertyOf(ObjectPropertyChain({cm.group(1)}) {cm.group(2)})", expected
    return None, None


def extract_functional_data_property(body: str) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for m in re.finditer(
        r"assert(True|False)\s*\(\s*m_reasoner\.isEntailed\s*\(\s*m_dataFactory\.getOWLFunctionalDataPropertyAxiom\s*\(\s*(\w+)\s*\)\s*\)\s*\)",
        body,
    ):
        out.append(
            {
                "property": f":{m.group(2)}",
                "kind": "functional",
                "expected": m.group(1) == "True",
            }
        )
    return out


def extract_object_property_domains(body: str) -> list[dict[str, Any]]:
    """Timothy bug: domains of q include A, B, and owl:Thing."""
    if "getObjectPropertyDomains" not in body:
        return []
    subs: list[dict[str, Any]] = []
    for cls in (":A", ":B", "owl:Thing"):
        subs.append({"sub": cls, "sup": "owl:Thing", "expected": True})
    return subs


def extract_buffer_axioms(body: str) -> str:
    """Concatenate StringBuffer.append fragments for loadReasonerWithAxioms(buffer.toString())."""
    if "buffer.toString()" not in body and "StringBuffer" not in body:
        return ""
    chunks: list[str] = []
    for m in re.finditer(r'buffer\.append\s*\(\s*"([^"]*)"\s*\)', body):
        chunks.append(m.group(1))
    for m in re.finditer(
        r'buffer\.append\s*\(\s*"([^"]*)"\s*\+\s*"([^"]*)"\s*\)',
        body,
    ):
        chunks.append(m.group(1) + m.group(2))
    return "".join(chunks)
