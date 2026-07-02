# ontologos

[![PyPI](https://img.shields.io/pypi/v/ontologos.svg)](https://pypi.org/project/ontologos/)
[![Python](https://img.shields.io/pypi/pyversions/ontologos.svg)](https://pypi.org/project/ontologos/)
[![Documentation](https://readthedocs.org/projects/ontologos/badge/?version=latest)](https://ontologos.readthedocs.io/en/latest/guides/python/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/eddiethedean/ontologos/blob/main/LICENSE-MIT)

Python bindings for [OntoLogos](https://github.com/eddiethedean/ontologos) — a Rust-native OWL reasoner for **RDFS**, **OWL RL**, and **OWL EL** classification on **PyPI 0.9.0**. Build from `main` for **DL** and **SWRL**.

> **Channel:** `pip install ontologos` installs **0.9.0** (EL, RL, RDFS). `profile="dl"` requires workspace **1.0.0** from `main`. See [Profile stability](https://ontologos.readthedocs.io/en/latest/guides/profile-stability.html).

Load `.owl` / `.ttl` files or build ontologies in memory, run the same profile engines as the CLI, and export taxonomies to pandas or Polars. Powered by PyO3 and the stable Python ABI (`abi3`).

**Full guide:** [ontologos.readthedocs.io — Python](https://ontologos.readthedocs.io/en/latest/guides/python/)

## Features

- **File or in-memory** — `Reasoner(path=...)` or `Ontology` / `OntologyBuilder`
- **Profiles** — `"rdfs"`, `"rl"`, `"el"`, `"auto"` on **PyPI 0.9.0**; `"dl"`, `"dl-preview"`, `"alc"`, `"swrl"` on workspace **`main` / 1.0.0**
- **Classify** — RDFS/RL materialization reports or EL taxonomy dicts
- **Explain** — proof graph dicts with IRI-resolved conclusions (EL full traces)
- **Incremental** — multi-pass `add_subclass_of` / `remove_subclass_of` with `incremental=True`
- **Export** — optional `subsumptions_to_pandas` / `subsumptions_to_polars`

## Install

Requires **Python 3.10+**.

```bash
pip install ontologos
```

Optional DataFrame helpers:

```bash
pip install 'ontologos[pandas]'
pip install 'ontologos[polars]'
```

Pre-built wheels are published for:

| OS | Architectures |
|----|---------------|
| Linux | `x86_64`, `aarch64` (manylinux) |
| macOS | `x86_64`, `aarch64` |
| Windows | `x64`, `aarch64` |

One `abi3` wheel per platform covers Python 3.10–3.13+. If no wheel matches, build from source (Rust + [maturin](https://github.com/PyO3/maturin)).

## Quick start

Download Pizza for EL examples (from a clone: `./benchmarks/scripts/download.sh` at repo root):

```bash
# From repository clone only:
./benchmarks/scripts/download.sh
```

### Classify an OWL file

```python
import ontologos

# OWL EL taxonomy
reasoner = ontologos.Reasoner(path="pizza.owl", profile="el")
taxonomy = reasoner.classify()
print(taxonomy["subsumption_count"], taxonomy["subsumptions"][:3])

# RDFS materialization
reasoner = ontologos.Reasoner(path="ontology.owl", profile="rdfs")
report = reasoner.classify()
print(report["inferred_axioms"])

# Auto-detect profile (EL or RL)
reasoner = ontologos.Reasoner(path="ontology.owl", profile="auto")
result = reasoner.classify()
```

### Build in memory

```python
from ontologos import OntologyBuilder, Reasoner

builder = OntologyBuilder()
builder.add_class("http://example.org/A")
builder.add_class("http://example.org/B")
builder.subclass_of("http://example.org/A", "http://example.org/B")
ontology = builder.build()

reasoner = Reasoner(ontology=ontology, profile="el")
taxonomy = reasoner.classify()
```

Load from JSON v2 dict instead of the builder:

```python
from ontologos import Ontology, Reasoner

ontology = Ontology.from_dict({
    "format_version": 2,
    "entities": [
        {"iri": "http://example.org/A", "kind": "Class"},
        {"iri": "http://example.org/B", "kind": "Class"},
    ],
    "axioms": [
        {"SubClassOf": {
            "subclass": "http://example.org/A",
            "superclass": "http://example.org/B",
        }}
    ],
})
reasoner = Reasoner(ontology=ontology, profile="el")
reasoner.classify()
```

### Incremental edits

```python
reasoner = Reasoner(ontology=ontology, profile="el", incremental=True)
reasoner.classify()

reasoner.add_subclass_of("http://example.org/B", "http://example.org/C")
reasoner.classify()  # re-classify with warm session

reasoner.remove_subclass_of("http://example.org/A", "http://example.org/B")
reasoner.classify()
```

### Explain inferences

```python
graph = reasoner.explain()
print(graph["node_count"])
for node in graph["nodes"][:5]:
    print(node["rule"], node.get("conclusion_sub"))
```

| Profile | Explain coverage |
|---------|------------------|
| **EL** | Full inference traces |
| **RL / RDFS** | Asserted axioms seeded; inferred steps lack per-rule premises until upstream exposes traces |
| **auto** | Routes like `classify` |

### Export to DataFrame

```python
from ontologos import subsumptions_to_pandas

df = subsumptions_to_pandas(taxonomy)
# columns: subclass, superclass
```

## API overview

| Type / method | Description |
|---------------|-------------|
| `Reasoner(path=..., profile=..., incremental=False)` | Load OWL file and classify |
| `Reasoner(ontology=..., ...)` | Classify in-memory ontology |
| `reasoner.classify()` | Run engine; returns taxonomy or materialization dict |
| `reasoner.explain()` | Proof graph dict (`node_count`, `nodes`, `parse_meta`) |
| `reasoner.taxonomy` | Last EL taxonomy (after `classify`) |
| `reasoner.parse_meta` | Parser warnings and axiom counts |
| `reasoner.add_subclass_of` / `remove_subclass_of` | Incremental axiom edits |
| `reasoner.add_axiom_json` | Add axiom via JSON v2 object |
| `Ontology.from_json` / `from_dict` | Load JSON v2 snapshot |
| `OntologyBuilder` | Fluent builder → `build()` |
| `subsumptions_to_pandas` / `subsumptions_to_polars` | Optional taxonomy export |

Invalid profiles and unsupported constructs raise `RuntimeError` with a message string.

`Reasoner` is **not thread-safe** — do not mutate from multiple threads.

## Development

From a repository clone:

```bash
cd crates/ontologos-py
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install 'maturin>=1.7,<2.0' pytest '.[pandas]'
maturin develop --release
pytest tests/ -q
```

Pizza EL golden test (requires corpus):

```bash
./benchmarks/scripts/download.sh   # from repo root
pytest tests/test_pizza_golden.py -q
```

## What OntoLogos is (and is not)

OntoLogos is an **orchestration layer**: profile detection, a unified ontology model, CLI, Python wheels, and security limits on top of in-house EL and **reasonable** for RL/RDFS. It maps a **subset** of OWL — not full OWL DL / HermiT replacement.

For engine-only workflows, consider [reasonable](https://crates.io/crates/reasonable), [whelk-rs](https://github.com/INCATools/whelk-rs), or [horned-owl](https://crates.io/crates/horned-owl) directly.

## Links

| Resource | URL |
|----------|-----|
| Python guide | [readthedocs — Python](https://ontologos.readthedocs.io/en/latest/guides/python/) |
| Getting started | [readthedocs](https://ontologos.readthedocs.io/en/latest/getting-started/) |
| Migration v0.8 → v0.9 | [migration guide](https://ontologos.readthedocs.io/en/latest/migration/v0.8.x-to-v0.9.0/) |
| Repository | [github.com/eddiethedean/ontologos](https://github.com/eddiethedean/ontologos) |
| Rust crates | [crates.io — ontologos-core](https://crates.io/crates/ontologos-core) |
| Changelog | [CHANGELOG.md](https://github.com/eddiethedean/ontologos/blob/main/CHANGELOG.md) |

## License

Licensed under either of [Apache-2.0](https://github.com/eddiethedean/ontologos/blob/main/LICENSE-APACHE) or [MIT](https://github.com/eddiethedean/ontologos/blob/main/LICENSE-MIT).
