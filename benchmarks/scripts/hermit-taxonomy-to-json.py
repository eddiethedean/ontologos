#!/usr/bin/env python3
"""Parse HermiT functional-syntax SubClassOf lines into taxonomy JSON."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

SUBCLASS_LINE = re.compile(r"^\s*SubClassOf\(")


def parse_hermit_taxonomy(text: str) -> list[list[str]]:
    pairs: list[list[str]] = []
    seen: set[tuple[str, str]] = set()
    for line in text.splitlines():
        if not SUBCLASS_LINE.match(line):
            continue
        iris = re.findall(r"<([^>]+)>", line)
        if len(iris) < 2:
            continue
        key = (iris[0], iris[1])
        if key not in seen:
            seen.add(key)
            pairs.append([iris[0], iris[1]])
    return pairs


def main() -> int:
    parser = argparse.ArgumentParser(description="HermiT taxonomy OFN → JSON subsumptions")
    parser.add_argument("input", type=Path, help="HermiT -o classify output file")
    parser.add_argument("-o", "--output", type=Path, help="Write JSON document")
    args = parser.parse_args()

    subs = parse_hermit_taxonomy(args.input.read_text())
    doc = {
        "status": "classified",
        "subsumption_count": len(subs),
        "subsumptions": subs,
    }
    payload = json.dumps(doc, indent=2) + "\n"
    if args.output:
        args.output.write_text(payload)
    else:
        sys.stdout.write(payload)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
