# Python Guide

Python bindings for OntoLogos (PyPI **1.0.0** on `main`; **0.9.0** latest tag) via PyO3 (`pip install ontologos`).

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

Download a sample ontology (works with `pip install` only — no clone required):

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
```

```python
import ontologos

print(ontologos.__version__)

# OWL RL saturation (Family ontology)
reasoner = ontologos.Reasoner(path="family.owl", profile="rl")
report = reasoner.classify()
print(report.get("inferred_axioms", report))

# RDFS TBox materialization
reasoner = ontologos.Reasoner(path="family.owl", profile="rdfs")
report = reasoner.classify()
print(report)

# OWL EL taxonomy — Pizza corpus requires clone + ./benchmarks/scripts/download.sh
# reasoner = ontologos.Reasoner(path="benchmarks/data/pizza.owl", profile="el")
# taxonomy = reasoner.classify()
# print(taxonomy["subsumption_count"])

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

### `Reasoner(path=None, ontology=None, profile=None, incremental=False, budget_secs=None)`

Constructs a reasoner from a file path **or** an in-memory `Ontology`. Exactly one of `path` or `ontology` is required.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `path` | `str` or `None` | `None` | Path to `.owl`, `.rdf`, `.ttl`, or `.ofn` file |
| `ontology` | `Ontology` or `None` | `None` | In-memory ontology from builder or JSON |
| `profile` | `str` or `None` | `"auto"` | `"auto"`, `"rdfs"`, `"rl"`, `"el"`, `"alc"`, `"dl"`, `"dl-preview"`, `"swrl"` |
| `incremental` | `bool` | `False` | Enable incremental session for multi-pass workflows |
| `budget_secs` | `int` or `None` | `None` | Wall-clock budget for DL consistency/classify (mirrors `ReasonerConfig`) |

**Profiles:**

| Profile | `classify()` return value |
|---------|--------------------------|
| `"rdfs"` | `dict` with `status`, `initial_axiom_count`, `final_axiom_count`, `inferred_axioms`, `inferred_by_rule`, `clash_count`, optional `clashes` |
| `"rl"` | Same report shape as RDFS (includes RL inferences) |
| `"el"` | `dict` with `status`, `subsumption_count`, `subsumptions`, `equivalences`, `unsatisfiable` |
| `"auto"` | EL taxonomy, RL report, or DL taxonomy based on profile detection |
| `"dl"`, `"dl-preview"`, `"alc"` | Taxonomy dict (preview — see [Preview profiles](preview-profiles.md)) |
| `"swrl"` | Preview — usually errors (`NotImplemented` / `PreviewLimit`) |

Invalid profile strings raise `RuntimeError`.

### `classify() -> dict`

Runs the selected profile engine on the loaded ontology.

- **EL / auto (EL) / DL / ALC:** Returns taxonomy dict with `status: "classified"`; also available via `reasoner.taxonomy` after classify.
- **RDFS / RL:** Returns materialization report dict aligned with CLI JSON.

### `check_consistency() -> dict`

Returns `{"consistent": bool, "complete": bool}`. When `complete` is `False`, do not treat `consistent` as proof — the DL engine hit a budget or tableau limit.

### `is_consistent() -> bool`

OWLReasoner-style convenience: returns `True`/`False` only when complete. Raises `IncompleteReasoningError` when the check did not finish.

### `is_entailed(...) -> bool`

Check entailment for `SubClassOf` (positional `sub`, `sup`), `ClassAssertion` (`individual=`, `class_=`), or `ObjectPropertyAssertion` (`subject=`, `property=`, `object=`). Exactly one form required.

### `query(query: str) -> list[dict]`

Answer an OWL QL conjunctive query after taxonomy classification (auto-classifies if needed). Each answer is a dict mapping variable names to IRI strings.

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
| `Ontology.from_json_with_limits(str, *, max_json_bytes=..., max_entities=..., max_axioms=..., max_iri_len=...)` | Load with resource caps (preferred for untrusted input) |
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

## Limitations (v1.0.0)

| Capability | Rust | Python |
|------------|------|--------|
| Load OWL files | Yes (horned-owl) | Yes |
| In-memory ontology | Yes (`OntologyBuilder`) | Yes |
| Profile detection | Yes | Via `"auto"` |
| RDFS / RL / EL classify | Yes | Yes |
| DL / ALC preview | Yes (preview) | Yes (`dl`, `dl-preview`, `alc`) |
| Consistency (`check_consistency`) | Yes (facade) | Yes |
| Incremental multi-pass | Yes (library API) | Yes |
| EL explain | Yes | Yes |
| RL/RDFS explain (full traces) | Partial (asserted-only) | Partial (asserted-only) |
| Export saturated ontology | Yes (in-process) | No |
| OWL QL query | Yes (`ontologos-ql` / CLI) | Yes (`Reasoner.query`) |
| Subproperties / property values | Yes (facade) | CLI only |
| Typed exceptions | Yes (`Error` enum) | Yes (`ParseError`, `ResourceLimitError`, `IncompleteReasoningError`) |
| `py.typed` / stubs | N/A | Yes |

`Reasoner` is not thread-safe; do not mutate from multiple threads concurrently.

## When to use upstream crates directly

| Goal | Prefer |
|------|--------|
| OWL RL saturation on oxrdf triples | `reasonable` |
| OWL EL on horned-owl ontologies | `whelk-rs` or ELK |
| Parse OWL/RDF only | `horned-owl` |
| One CLI, Python package, profile routing, core model | **OntoLogos** |

## Errors

Import typed exceptions from the package root:

```python
from ontologos import ParseError, ResourceLimitError, IncompleteReasoningError
```

| Exception | When |
|-----------|------|
| `ParseError` | Parser / JSON serialization failures |
| `ResourceLimitError` | DL/ALC preview or resource caps exceeded |
| `IncompleteReasoningError` | `is_consistent()` when check incomplete; DL budget limits |
| `RuntimeError` | Other failures (unsupported profile, invalid constructor args, etc.) |

Common `RuntimeError` messages:

| Message pattern | Cause |
|-----------------|-------|
| `unsupported profile` | Invalid profile string |
| `classification not supported for profile` | Invalid profile or SWRL not implemented |
| `requires exactly one of path or ontology` | Both or neither constructor args |
| Parse / I/O errors | Bad path, unsupported format, mapping failure |

## Related

- [Preview profiles](preview-profiles.md)
- [Facade API](facade-api.md)
- [Getting started](../getting-started/index.md)
- [Incremental reasoning](incremental-reasoning.md)
- [OWL EL classification](../getting-started/owl-el-classification.md)
- [Migration v0.8 → v0.9](../migration/v0.8.x-to-v0.9.0.md)
