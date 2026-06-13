# Python Guide

Python bindings for OntoLogos v0.9.0 via PyO3 (`pip install ontologos`).

OntoLogos is an **orchestration layer**: the Python API routes to the same Rust facades as the CLI
(`ontologos-el` in-house EL, `ontologos-rl` / `ontologos-rdfs` → reasonable). Power users who need
direct engine access can use upstream crates ([reasonable](https://crates.io/crates/reasonable),
[whelk-rs](https://github.com/INCATools/whelk-rs) as an EL peer) or horned-owl for parsing-only workflows.
OntoLogos adds profile detection, the unified `Ontology` model, security limits, CLI, and wheels.

## Install

Requires **Python 3.10+**. Release wheels are published for:

| OS | Architectures |
|----|---------------|
| Linux | `x86_64`, `aarch64` (manylinux) |
| macOS | `x86_64`, `aarch64` (Apple Silicon + Intel) |
| Windows | `x64`, `aarch64` |

Wheels use the stable Python ABI (`abi3`) — one wheel per platform covers Python 3.10–3.13+. If no wheel matches your platform, build from source (requires Rust + maturin).

```bash
pip install ontologos
```

Optional DataFrame export:

```bash
pip install 'ontologos[pandas]'
pip install 'ontologos[polars]'
```

Development install from a clone:

```bash
cd crates/ontologos-py
python -m venv .venv
source .venv/bin/activate
pip install 'maturin>=1.7,<2.0' pytest '.[pandas]'
maturin develop --release
pytest tests/ -q
```

## Quick start

```python
import ontologos

print(ontologos.__version__)

# RDFS TBox materialization
reasoner = ontologos.Reasoner(path="ontology.owl", profile="rdfs")
report = reasoner.classify()
print(report["inferred_axioms"])

# OWL EL taxonomy
reasoner = ontologos.Reasoner(path="pizza.owl", profile="el")
taxonomy = reasoner.classify()
print(taxonomy["subsumption_count"])

# Build in memory
builder = ontologos.OntologyBuilder()
builder.add_class("http://example.org/A")
builder.add_class("http://example.org/B")
builder.subclass_of("http://example.org/A", "http://example.org/B")
reasoner = ontologos.Reasoner(ontology=builder.build(), profile="el")
reasoner.classify()

# Explain
graph = reasoner.explain()
print(graph["node_count"])
```

## API reference

### `Reasoner(path=None, ontology=None, profile=None, incremental=False)`

Constructs a reasoner from a file path **or** an in-memory `Ontology`. Exactly one of `path` or `ontology` is required.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `path` | `str` or `None` | `None` | Path to `.owl`, `.rdf`, `.ttl`, or `.ofn` file |
| `ontology` | `Ontology` or `None` | `None` | In-memory ontology from builder or JSON |
| `profile` | `str` or `None` | `"auto"` | `"auto"`, `"rdfs"`, `"rl"`, `"el"`, `"alc"`, `"dl"`, `"dl-preview"`, `"swrl"` |
| `incremental` | `bool` | `False` | Enable incremental session for multi-pass workflows |

**Profiles:**

| Profile | `classify()` return value |
|---------|--------------------------|
| `"rdfs"` | `dict` with `initial_axiom_count`, `final_axiom_count`, `inferred_axioms` |
| `"rl"` | Same report shape as RDFS (includes RL inferences) |
| `"el"` | `dict` with `subsumption_count`, `subsumptions`, `equivalences`, `unsatisfiable` |
| `"auto"` | EL taxonomy, RL report, or DL taxonomy based on profile detection |
| `"dl"`, `"dl-preview"`, `"alc"` | Taxonomy dict (preview — see [Preview profiles](preview-profiles.md)) |
| `"swrl"` | Preview — usually errors (`NotImplemented` / `PreviewLimit`) |

Invalid profile strings raise `RuntimeError`.

### `classify() -> dict`

Runs the selected profile engine on the loaded ontology.

- **EL / auto (EL):** Returns taxonomy dict; also available via `reasoner.taxonomy` after classify.
- **RDFS / RL:** Returns materialization report dict with axiom counts.

### `explain() -> dict`

Returns a proof graph dict with `node_count`, `nodes` (list of step dicts with `rule`, `premises`, and optional conclusion fields), and `parse_meta` when warnings exist.

#### Explain trace limits

| Profile | Coverage |
|---------|----------|
| **EL** | Full inference traces → proof graph with IRI-resolved conclusions |
| **RL / RDFS** | Proof graph seeds **asserted** axioms; inferred steps lack per-rule premises until reasonable exposes a trace API |
| **auto** | Routes like `classify`; DL-detected ontologies use DL preview classifier |

See [Conformance](../reference/conformance.md) for evaluator notes.

### Incremental mutations

With `incremental=True`, edit the ontology between `classify()` calls:

| Method | Description |
|--------|-------------|
| `add_subclass_of(sub_iri, sup_iri)` | Add `SubClassOf` axiom |
| `remove_subclass_of(sub_iri, sup_iri)` | Remove matching asserted axiom |
| `add_axiom_json(axiom_dict)` | Add axiom using JSON v2 axiom object (e.g. `{"SubClassOf": {...}}`) |

```python
reasoner = ontologos.Reasoner(ontology=ont, profile="el", incremental=True)
reasoner.classify()
reasoner.add_subclass_of("http://example.org/B", "http://example.org/C")
reasoner.classify()
```

See [Incremental reasoning](incremental-reasoning.md).

### `Ontology`

| Method | Description |
|--------|-------------|
| `Ontology.from_json(str)` | Load from JSON v2 snapshot |
| `Ontology.from_dict(dict)` | Load from Python dict (same schema as JSON v2) |
| `to_json()` / `to_dict()` | Serialize |
| `axiom_count` / `entity_count` | Size getters |

### `OntologyBuilder`

Fluent builder for common TBox/ABox axioms:

- `add_class`, `individual`, `object_property`
- `subclass_of`, `subproperty_of`, `property_domain`, `property_range`
- `class_assertion`, `object_property_assertion`
- `build() -> Ontology`

Exotic axioms (nominals, property characteristics, etc.) use `Ontology.from_json` / `from_dict`.

### DataFrame export

```python
from ontologos import subsumptions_to_pandas, subsumptions_to_polars

taxonomy = reasoner.classify()
df = subsumptions_to_pandas(taxonomy)
```

Requires optional `pandas` or `polars` install (`pip install 'ontologos[pandas]'`).

### `parse_meta` (property)

Read-only dict after load:

| Key | Type | Description |
|-----|------|-------------|
| `warnings` | `list[str]` | Parser mapping warnings |
| `mapped_axiom_count` | `int` | Axioms stored in core |
| `skipped_axiom_count` | `int` | Logical components not mapped |
| `logical_axiom_count` | `int` | Mapped + skipped |

## Limitations (v0.9.0)

| Capability | Rust v0.9.0 | Python v0.9.0 |
|------------|-------------|---------------|
| Load OWL files | Yes (horned-owl) | Yes |
| In-memory ontology | Yes (`OntologyBuilder`) | Yes |
| Profile detection | Yes | Via `"auto"` |
| RDFS / RL / EL classify | Yes | Yes |
| DL / ALC preview | Yes (preview) | Yes (`dl`, `dl-preview`, `alc`) |
| Incremental multi-pass | Yes (library API) | Yes |
| EL explain | Yes | Yes |
| RL/RDFS explain (full traces) | Partial (asserted-only) | Partial (asserted-only) |
| Export saturated ontology | Yes (in-process) | No |
| Query API | Yes (`ontologos-query`) | No |

`Reasoner` is not thread-safe; do not mutate from multiple threads concurrently.

## When to use upstream crates directly

| Goal | Prefer |
|------|--------|
| OWL RL saturation on oxrdf triples | `reasonable` |
| OWL EL on horned-owl ontologies | `whelk-rs` or ELK |
| Parse OWL/RDF only | `horned-owl` |
| One CLI, Python package, profile routing, core model | **OntoLogos** |

## Errors

All failures surface as `RuntimeError` with a string message. Common messages:

| Message pattern | Cause |
|-----------------|-------|
| `unsupported profile` | Invalid profile string |
| `classification not supported for profile` | Invalid profile or SWRL not implemented |
| `PreviewLimit` / `ResourceLimit` | DL preview limits — see [Preview profiles](preview-profiles.md) |
| `requires exactly one of path or ontology` | Both or neither constructor args |
| Parse / I/O errors | Bad path, unsupported format, mapping failure |

## Related

- [Preview profiles](preview-profiles.md)
- [Facade API](facade-api.md)
- [Getting started](../getting-started/index.md)
- [Incremental reasoning](incremental-reasoning.md)
- [OWL EL classification](../getting-started/owl-el-classification.md)
- [Migration v0.8 → v0.9](../migration/v0.8.x-to-v0.9.0.md)
