"""Unit tests for Tier C taxonomy harness scripts."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = REPO_ROOT / "benchmarks" / "scripts"


def test_hermit_taxonomy_parser_sample() -> None:
    sample = "\n".join(
        [
            "SubClassOf( <http://a.com/ontology#Father> <http://a.com/ontology#Parent> )",
            "SubClassOf( <http://a.com/ontology#Mother> <http://a.com/ontology#Parent> )",
            "# comment",
            "SubClassOf( <http://a.com/ontology#Brother> <http://a.com/ontology#Sibling> )",
        ]
    )
    tmp = REPO_ROOT / "target" / "hermit-sample.ofn"
    tmp.parent.mkdir(parents=True, exist_ok=True)
    tmp.write_text(sample)

    out = subprocess.check_output(
        [sys.executable, str(SCRIPTS / "hermit-taxonomy-to-json.py"), str(tmp)],
        text=True,
    )
    doc = json.loads(out)
    assert doc["subsumption_count"] == 3
    assert ["http://a.com/ontology#Father", "http://a.com/ontology#Parent"] in doc[
        "subsumptions"
    ]


def test_compare_taxonomy_missing_extra_and_filters() -> None:
    golden = {
        "status": "classified",
        "subsumption_count": 3,
        "subsumptions": [
            ["http://ex.org/a", "http://ex.org/b"],
            ["http://ex.org/b", "http://ex.org/c"],
            ["http://ex.org/x", "http://www.w3.org/2002/07/owl#Thing"],
        ],
    }
    actual_match = {
        "status": "classified",
        "subsumption_count": 2,
        "subsumptions": [
            ["http://ex.org/a", "http://ex.org/b"],
            ["http://ex.org/b", "http://ex.org/c"],
        ],
    }
    actual_extra = {
        "status": "classified",
        "subsumption_count": 3,
        "subsumptions": [
            ["http://ex.org/a", "http://ex.org/b"],
            ["http://ex.org/b", "http://ex.org/c"],
            ["http://ex.org/z", "http://ex.org/c"],
        ],
    }
    data_dir = REPO_ROOT / "target" / "taxonomy-harness"
    data_dir.mkdir(parents=True, exist_ok=True)
    golden_path = data_dir / "golden.json"
    actual_path = data_dir / "actual.json"
    golden_path.write_text(json.dumps(golden))
    actual_path.write_text(json.dumps(actual_match))

    ok = subprocess.run(
        [
            sys.executable,
            str(SCRIPTS / "compare-taxonomy.py"),
            str(golden_path),
            str(actual_path),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert ok.returncode == 0, ok.stderr

    actual_path.write_text(json.dumps(actual_extra))
    bad = subprocess.run(
        [
            sys.executable,
            str(SCRIPTS / "compare-taxonomy.py"),
            str(golden_path),
            str(actual_path),
            "--max-extra",
            "0",
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert bad.returncode != 0

    hermit = {
        "subsumptions": [
            ["http://ex.org/a", "http://ex.org/b"],
            ["http://other.org/a", "http://other.org/b"],
        ]
    }
    onto = {
        "subsumptions": [
            ["http://ex.org/a", "http://ex.org/b"],
            ["http://ex.org/c", "http://ex.org/d"],
        ]
    }
    hermit_path = data_dir / "hermit.json"
    onto_path = data_dir / "onto.json"
    hermit_path.write_text(json.dumps(hermit))
    onto_path.write_text(json.dumps(onto))
    filtered = subprocess.run(
        [
            sys.executable,
            str(SCRIPTS / "compare-taxonomy.py"),
            str(hermit_path),
            str(onto_path),
            "--namespace-prefix",
            "http://ex.org/",
            "--max-missing",
            "0",
            "--max-extra",
            "5",
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert filtered.returncode == 0, filtered.stderr
