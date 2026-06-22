#!/usr/bin/env python3
"""Extract HermiT DL clause goldens from ClausificationDatatypesTest.java."""

from __future__ import annotations

import json
import re
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
JAVA_DT = (
    REPO.parent
    / "tmp"
    / "hermit-src"
    / "src"
    / "test"
    / "java"
    / "org"
    / "semanticweb"
    / "HermiT"
    / "structural"
    / "ClausificationDatatypesTest.java"
)
# Fallback: clone path used during development
if not JAVA_DT.is_file():
    JAVA_DT = Path("/tmp/hermit-src/src/test/java/org/semanticweb/HermiT/structural/ClausificationDatatypesTest.java")

ASYMMETRY = Path("/tmp/hermit-src/src/test/resources/org/semanticweb/HermiT/structural/res/asymmetry-control.txt")

CATALOG = REPO / "benchmarks/data/hermit/catalog/cases.json"
OUT_DIR = REPO / "benchmarks/data/hermit/clauses"


def parse_datatype_tests(java_text: str) -> dict[str, list[str]]:
    pat = re.compile(
        r"public void (test\w+)\(\) throws Exception \{.*?assertContainsAll\(this\.getName\(\),clauses,\s*S\((.*?)\)\s*\);",
        re.S,
    )
    out: dict[str, list[str]] = {}
    for m in pat.finditer(java_text):
        method = m.group(1)
        body = m.group(2)
        clauses = [bytes(s, "utf-8").decode("unicode_escape") for s in re.findall(r'"((?:[^"\\]|\\.)*)"', body)]
        out[method] = clauses
    return out


def case_id(java_class: str, java_method: str) -> str:
    return f"{java_class}.{java_method}"


def safe_filename(case_id_str: str) -> str:
    return case_id_str.replace(".", "_") + ".txt"


def main() -> None:
    if not JAVA_DT.is_file():
        raise SystemExit(f"missing HermiT test source at {JAVA_DT}")

    java_text = JAVA_DT.read_text(encoding="utf-8")
    by_method = parse_datatype_tests(java_text)

    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    clausify_cases = [c for c in catalog if c.get("status") == "clausify"]
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    written = 0
    for case in clausify_cases:
        method = case["java_method"]
        cid = case["id"]
        if method == "testAsymmetry":
            if not ASYMMETRY.is_file():
                raise SystemExit(f"missing asymmetry control at {ASYMMETRY}")
            lines = [ln.strip() for ln in ASYMMETRY.read_text(encoding="utf-8").splitlines() if ln.strip()]
        else:
            lines = by_method.get(method)
            if lines is None:
                raise SystemExit(f"no Java clauses for {cid} ({method})")
        path = OUT_DIR / safe_filename(cid)
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        written += 1
        print(f"wrote {path.name} ({len(lines)} clauses)")

    print(f"done: {written} golden files in {OUT_DIR}")


if __name__ == "__main__":
    main()
