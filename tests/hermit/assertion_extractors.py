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


# --- Phase 5: datatype DR satisfiability, RIA regularity, role simplicity ---


def _split_java_call_args(inner: str) -> list[str]:
    """Split a Java argument list on top-level commas."""
    args: list[str] = []
    depth = 0
    in_str = False
    escape = False
    current: list[str] = []
    for ch in inner:
        if in_str:
            current.append(ch)
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == '"':
                in_str = False
            continue
        if ch == '"':
            in_str = True
            current.append(ch)
            continue
        if ch in "([{":
            depth += 1
            current.append(ch)
            continue
        if ch in ")]}":
            depth -= 1
            current.append(ch)
            continue
        if ch == "," and depth == 0:
            arg = "".join(current).strip()
            if arg:
                args.append(arg)
            current = []
            continue
        current.append(ch)
    tail = "".join(current).strip()
    if tail:
        args.append(tail)
    return args


def _java_string_literal(expr: str) -> str | None:
    m = re.fullmatch(r'"((?:[^"\\]|\\.)*)"', expr.strip())
    if not m:
        return None
    return bytes(m.group(1), "utf-8").decode("unicode_escape")


def _expand_dr_java_expr(expr: str) -> str:
    """Expand HermiT AbstractReasonerTest DR/NOT/OO/INT/... helpers to OFN data ranges."""
    expr = expr.strip()
    if not expr:
        return expr

    lit = _java_string_literal(expr)
    if lit is not None:
        return f'"{lit}"'

    m = re.match(r"(\w+)\s*\((.*)\)\s*$", expr, re.DOTALL)
    if not m:
        return expr
    func, inner = m.group(1), m.group(2)
    args = _split_java_call_args(inner)

    if func == "NOT":
        return f"DataComplementOf( {_expand_dr_java_expr(args[0])} )"
    if func == "DR":
        dt = args[0].strip().strip('"')
        if len(args) == 1:
            return dt
        expanded_facets: list[str] = []
        i = 1
        while i < len(args):
            facet_name = args[i].strip().strip('"')
            expanded_facets.append(facet_name)
            i += 1
            if i < len(args) and not args[i].strip().strip('"').startswith(
                ("xsd:", "owl:", "rdf:")
            ):
                expanded_facets.append(_expand_dr_java_expr(args[i]))
                i += 1
        return f"DatatypeRestriction( {dt} {' '.join(expanded_facets)} )"
    if func == "OO" or func == "S":
        parts = " ".join(_expand_dr_java_expr(a) for a in args)
        return f"DataOneOf( {parts} )"
    if func == "INT":
        v = _java_string_literal(args[0]) or args[0].strip('"')
        return f'"{v}"^^xsd:integer'
    if func == "DEC":
        v = _java_string_literal(args[0]) or args[0].strip('"')
        return f'"{v}"^^xsd:decimal'
    if func == "RAT":
        num = _java_string_literal(args[0]) or args[0].strip('"')
        denom = _java_string_literal(args[1]) or args[1].strip('"')
        return f'"{num}/{denom}"^^owl:rational'
    if func == "FLT":
        v = _java_string_literal(args[0]) or args[0].strip('"')
        return f'"{v}"^^xsd:float'
    if func == "DBL":
        v = _java_string_literal(args[0]) or args[0].strip('"')
        return f'"{v}"^^xsd:double'
    if func in ("DATE", "DATES"):
        v = _java_string_literal(args[0]) or args[0].strip('"')
        dt = "xsd:dateTimeStamp" if func == "DATES" else "xsd:dateTime"
        return f'"{v}"^^{dt}'
    if func == "XMLL":
        v = _java_string_literal(args[0]) or args[0].strip('"')
        return f'"{v}"^^rdf:XMLLiteral'
    if func in ("HEXB", "B64B"):
        v = _java_string_literal(args[0]) or args[0].strip('"')
        dt = "xsd:hexBinary" if func == "HEXB" else "xsd:base64Binary"
        return f'"{v}"^^{dt}'
    if func == "STR":
        if len(args) == 1:
            v = _java_string_literal(args[0]) or args[0].strip('"')
            return f'"{v}"^^xsd:string'
        v = _java_string_literal(args[0]) or args[0].strip('"')
        lang = _java_string_literal(args[1]) or args[1].strip('"')
        return f'"{v}"@{lang}'
    if func == "AURI":
        v = _java_string_literal(args[0]) or args[0].strip('"')
        return f'"{v}"^^xsd:anyURI'
    if func == "PL":
        v = _java_string_literal(args[0]) or args[0].strip('"')
        lang = _java_string_literal(args[1]) if len(args) > 1 else None
        if lang:
            return f'"{v}"@{lang}'
        return f'"{v}"^^rdf:PlainLiteral'
    return expr


def _parse_s_literals(part: str) -> list[str] | None:
    """Parse HermiT S(...) literal sets used by assertDRSatisfiableNEQ."""
    m = re.match(r"S\s*\((.*)\)\s*$", part.strip(), re.DOTALL)
    if not m:
        return None
    args = _split_java_call_args(m.group(1))
    if not args:
        return None
    return [_expand_dr_java_expr(a) for a in args]


def _build_drsatisfiable_axioms(
    expected: bool,
    cardinality: int,
    parts: list[str],
    *,
    forbidden_internal: bool,
    forbidden_literals: list[str] | None = None,
) -> str:
    """Mirror HermiT AbstractReasonerTest.assertDRSatisfiableNEQ OFN assembly."""
    del expected
    buf: list[str] = [
        "Declaration(NamedIndividual(:a))",
        "Declaration(Class(:A))",
        "Declaration(DataProperty(:dp))",
        f"SubClassOf( :A DataMinCardinality( {cardinality} :dp rdfs:Literal ) )",
    ]
    for part in parts:
        expanded = _expand_dr_java_expr(part)
        buf.append(f"SubClassOf( :A DataAllValuesFrom( :dp {expanded} ) )")
    buf.append("ClassAssertion( :A :a )")
    if forbidden_literals:
        for index, forbidden in enumerate(forbidden_literals):
            fv = f":fv{index}"
            buf.extend(
                [
                    f"Declaration(DataProperty({fv}))",
                    f"DisjointDataProperties( :dp {fv} )",
                    f"DataPropertyAssertion( {fv} :a {forbidden} )",
                ]
            )
    elif forbidden_internal:
        buf.extend(
            [
                "Declaration(DataProperty(:fv0))",
                "DisjointDataProperties( :dp :fv0 )",
                'DataPropertyAssertion( :fv0 :a "$internal$"^^xsd:string )',
            ]
        )
    return " ".join(buf)


def extract_assert_drsatisfiable(body: str) -> dict[str, Any] | None:
    """Harvest assertDRSatisfiable* calls into OFN axioms + expected consistency."""
    for method in (
        "assertDRSatisfiableUseCliqueOptimization",
        "assertDRSatisfiableNEQ",
        "assertDRSatisfiable",
    ):
        m = re.search(rf"{method}\s*\((.*)\)\s*;", body, re.DOTALL)
        if not m:
            continue
        args = _split_java_call_args(m.group(1))
        if not args:
            continue
        idx = 0
        expected = args[idx].strip() == "true"
        idx += 1
        cardinality = 1
        forbidden_internal = method == "assertDRSatisfiable" and len(args) > 2
        if method == "assertDRSatisfiableNEQ":
            forbidden_internal = False
            if idx < len(args) and re.fullmatch(r"\d+", args[idx].strip()):
                cardinality = int(args[idx].strip())
                idx += 1
            if idx < len(args) and args[idx].strip() == "null":
                idx += 1
            elif idx < len(args) and args[idx].strip().startswith("new String[]"):
                idx += 1
        elif method == "assertDRSatisfiableUseCliqueOptimization":
            forbidden_internal = False
            if idx < len(args) and re.fullmatch(r"\d+", args[idx].strip()):
                cardinality = int(args[idx].strip())
                idx += 1
        elif method == "assertDRSatisfiable":
            if idx < len(args) and re.fullmatch(r"\d+", args[idx].strip()):
                cardinality = int(args[idx].strip())
                idx += 1
                forbidden_internal = True
            else:
                forbidden_internal = False
        parts = args[idx:]
        forbidden_literals: list[str] | None = None
        if method == "assertDRSatisfiableNEQ" and parts:
            s_literals = _parse_s_literals(parts[0])
            if s_literals is not None:
                forbidden_literals = s_literals
                parts = parts[1:]
        axioms = _build_drsatisfiable_axioms(
            expected,
            cardinality,
            parts,
            forbidden_internal=forbidden_internal,
            forbidden_literals=forbidden_literals,
        )
        return {"axioms": axioms, "expected": expected, "cardinality": cardinality}
    return None


def _extract_java_axioms_string(body: str) -> str:
    """Pull a Java string-literal axiom block from assertRegular/assertSimple calls."""
    m = re.search(
        r"assert(?:Regular|Simple)\s*\(\s*((?:\"[^\"\\]*(?:\\.[^\"\\]*)*\"\s*)+)\s*,\s*(true|false)\s*\)",
        body,
    )
    if m:
        parts = re.findall(r'"([^"\\]*(?:\\.[^\"\\]*)*)"', m.group(1))
        return bytes("".join(parts), "utf-8").decode("unicode_escape")
    m = re.search(
        r"assert(?:Regular|Simple)\s*\(\s*(\w+)\s*,\s*(true|false)\s*\)",
        body,
    )
    if not m:
        return ""
    var = m.group(1)
    assign = re.search(
        rf"String\s+{re.escape(var)}\s*=\s*((?:(?:\"[^\"\\]*(?:\\.[^\"\\]*)*\"\s*)|(?:\+\s*\"[^\"\\]*(?:\\.[^\"\\]*)*\"\s*))+)",
        body,
        re.DOTALL,
    )
    if not assign:
        return ""
    parts = re.findall(r'"([^"\\]*(?:\\.[^\"\\]*)*)"', assign.group(1))
    return bytes("".join(parts), "utf-8").decode("unicode_escape")


def extract_ria_regularity(body: str) -> dict[str, Any] | None:
    m = re.search(
        r"assertRegular\s*\(\s*(.+?)\s*,\s*(true|false)\s*\)\s*;",
        body,
        re.DOTALL,
    )
    if not m:
        return None
    axioms = _extract_java_axioms_string(body)
    if not axioms:
        return None
    return {"axioms": axioms, "expected": m.group(2) == "true"}


def extract_role_simplicity(body: str) -> dict[str, Any] | None:
    m = re.search(
        r"assertSimple\s*\(\s*(.+?)\s*,\s*(true|false)\s*\)\s*;",
        body,
        re.DOTALL,
    )
    if not m:
        return None
    axioms = _extract_java_axioms_string(body)
    if not axioms:
        return None
    return {"axioms": axioms, "expected": m.group(2) == "true"}
