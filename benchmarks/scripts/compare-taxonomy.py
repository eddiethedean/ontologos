#!/usr/bin/env python3
"""Compare classification taxonomies with documented tolerance (Tier C harness).

Golden baselines are vendored JSON from OntoLogos or external reasoners (HermiT/Konclude).
See docs/reference/taxonomy-tolerance.md for allowed diffs.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

OWL_THING = "http://www.w3.org/2002/07/owl#Thing"


def pair_set(subsumptions: list[list[str]]) -> set[tuple[str, str]]:
    return {tuple(p) for p in subsumptions}


def filter_thing(edges: set[tuple[str, str]]) -> set[tuple[str, str]]:
    return {(s, t) for s, t in edges if t != OWL_THING}


def filter_namespace(
    edges: set[tuple[str, str]], prefix: str
) -> set[tuple[str, str]]:
    if not prefix:
        return edges
    return {(s, t) for s, t in edges if s.startswith(prefix)}


def compare_taxonomies(
    golden: list[list[str]],
    actual: list[list[str]],
    *,
    max_missing: int = 0,
    max_extra: int = 0,
    ignore_thing: bool = True,
    namespace_prefix: str = "",
) -> tuple[bool, str]:
    g = pair_set(golden)
    a = pair_set(actual)
    if namespace_prefix:
        g = filter_namespace(g, namespace_prefix)
        a = filter_namespace(a, namespace_prefix)
    if ignore_thing:
        g = filter_thing(g)
        a = filter_thing(a)
    missing = g - a
    extra = a - g
    if len(missing) <= max_missing and len(extra) <= max_extra:
        return True, f"ok: {len(a)} edges (missing={len(missing)} extra={len(extra)})"
    msg = f"mismatch: missing={len(missing)} extra={len(extra)}"
    if missing:
        sample = sorted(missing)[:5]
        msg += f"; missing sample: {sample}"
    if extra:
        sample = sorted(extra)[:5]
        msg += f"; extra sample: {sample}"
    return False, msg


def main() -> int:
    parser = argparse.ArgumentParser(description="Compare taxonomy JSON files")
    parser.add_argument("golden", type=Path, help="Golden JSON file or corpora entry path")
    parser.add_argument("actual", type=Path, help="Actual classify JSON output")
    parser.add_argument("--max-missing", type=int, default=0)
    parser.add_argument("--max-extra", type=int, default=0)
    parser.add_argument(
        "--allow-thing",
        action="store_true",
        help="Do not ignore direct subsumptions to owl:Thing",
    )
    parser.add_argument(
        "--corpus-key",
        help="Key under corpora in dl-taxonomy-golden.json (default: read flat subsumptions)",
    )
    parser.add_argument(
        "--namespace-prefix",
        default="",
        help="Keep only subsumptions whose subclass IRI starts with this prefix",
    )
    args = parser.parse_args()

    golden_doc = json.loads(args.golden.read_text())
    actual_doc = json.loads(args.actual.read_text())

    if args.corpus_key:
        golden_doc = golden_doc["corpora"][args.corpus_key]

    golden_subs = golden_doc["subsumptions"]
    actual_subs = actual_doc.get("subsumptions", [])

    ok, message = compare_taxonomies(
        golden_subs,
        actual_subs,
        max_missing=args.max_missing,
        max_extra=args.max_extra,
        ignore_thing=not args.allow_thing,
        namespace_prefix=args.namespace_prefix,
    )
    print(message)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
