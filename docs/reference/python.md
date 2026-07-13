# Python API Reference

Python bindings for OntoLogos via PyPI [`ontologos`](https://pypi.org/project/ontologos/).

Tutorial: [Python guide](../guides/python.md). Install channels: [Install and channels](../guides/install-channels.md).

## Install

```bash
pip install ontologos
pip install 'ontologos[pandas]'   # optional DataFrame export
pip install 'ontologos[polars]'    # optional polars export
```

Requires **Python 3.10+**.

## Profiles

Published on PyPI **1.1.3**: `"auto"`, `"rdfs"`, `"rl"`, `"el"`, `"dl"`, and `"swrl"` are production-supported. `"alc"` and `"dl-preview"` are preview only.

Full matrix: [Profile stability](../guides/profile-stability.md).

## `Reasoner`

```python
from ontologos import Reasoner
```

### Constructor

`Reasoner(path=None, ontology=None, profile="auto", incremental=False, budget_secs=None)`

Exactly one of `path` or `ontology` is required.

| Parameter | Description |
|-----------|-------------|
| `path` | Path to `.owl`, `.rdf`, `.ttl`, or `.ofn` |
| `ontology` | In-memory ontology from `OntologyBuilder` or JSON |
| `profile` | `"auto"`, `"rdfs"`, `"rl"`, `"el"`, `"dl"`, `"swrl"`, `"alc"`, `"dl-preview"` |
| `incremental` | Reuse session across `classify()` calls after mutations |
| `budget_secs` | Wall-clock budget for DL consistency/classify |

Invalid profile strings raise `RuntimeError`.

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `classify()` | `dict` | Run profile engine; taxonomy or materialization report |
| `check_consistency()` | `dict` | `{"consistent": bool, "complete": bool}` — prefer over `is_consistent` for DL |
| `is_consistent()` | `bool` | Convenience; raises `IncompleteReasoningError` when incomplete |
| `is_entailed(...)` | `bool` | Entailment for `SubClassOf`, `ClassAssertion`, or `ObjectPropertyAssertion` |
| `query(query: str)` | `list[dict]` | OWL QL conjunctive query (auto-classifies if needed) |
| `explain()` | `dict` | Proof graph (EL full; RL/RDFS asserted-only) |
| `add_subclass_of(sub, sup)` | `None` | Incremental mutation |
| `remove_subclass_of(sub, sup)` | `None` | Incremental mutation |
| `add_axiom_json(axiom)` | `None` | Add axiom from JSON dict |

#### `classify()` return shapes

| Profile family | Keys (typical) |
|----------------|----------------|
| EL / DL / ALC | `status: "classified"`, `subsumption_count`, `subsumptions` |
| RDFS / RL | `status: "classified"`, `initial_axiom_count`, `final_axiom_count`, `inferred_axioms` |

#### `is_entailed` forms

- `is_entailed(sub, sup)` — `SubClassOf`
- `is_entailed(individual=..., class_=...)` — `ClassAssertion`
- `is_entailed(subject=..., property=..., object=...)` — `ObjectPropertyAssertion`

### Properties

| Property | Description |
|----------|-------------|
| `taxonomy` | Last taxonomy dict after EL/DL `classify()` |
| `parse_meta` | Parser metadata dict after file load (`warnings`, `mapped_axiom_count`, …) |

## `OntologyBuilder`

```python
from ontologos import OntologyBuilder

b = OntologyBuilder()
b.add_class("http://example.org/Food")
b.add_class("http://example.org/Pizza")
b.subclass_of("http://example.org/Pizza", "http://example.org/Food")
ontology = b.build()
reasoner = Reasoner(ontology=ontology, profile="el")
```

Builder methods: `add_class`, `individual`, `object_property`, `subclass_of`, `subproperty_of`, `property_domain`, `property_range`, `class_assertion`, `object_property_assertion`, `build()`.

## `Ontology`

| Method | Description |
|--------|-------------|
| `from_json(str)` | Load JSON snapshot (v2 or v3) |
| `from_json_with_limits(str, *, max_json_bytes=..., ...)` | Load with resource caps |
| `from_dict(dict)` | Load from Python dict |
| `to_json()` / `to_dict()` | Serialize |
| `axiom_count` / `entity_count` | Size getters |

## Export functions

```python
from ontologos import Reasoner, subsumptions_to_pandas, subsumptions_to_polars

# EL taxonomy export (requires pizza.owl)
report = Reasoner(path="pizza.owl", profile="el").classify()
df = subsumptions_to_pandas(report)   # requires ontologos[pandas]
```

There is no `taxonomy_dataframe()` method — use `subsumptions_to_pandas(report)` or `subsumptions_to_polars(report)`.

## Exceptions

```python
from ontologos import ParseError, ResourceLimitError, IncompleteReasoningError
```

| Exception | When |
|-----------|------|
| `ParseError` | Parser / JSON serialization failures |
| `ResourceLimitError` | DL/ALC preview or resource caps exceeded |
| `IncompleteReasoningError` | `is_consistent()` when check incomplete; DL budget limits |
| `RuntimeError` | Unsupported profile, invalid constructor args, other failures |

## Quick start

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
```

```python
from ontologos import Reasoner

report = Reasoner(path="family.owl", profile="rl").classify()
print(report)
```

## Limitations

- Not thread-safe — one `Reasoner` per thread
- Subproperty / property-value lookups: CLI and Rust facade only
- Validate DL results on your corpus before production — see [DL API](dl.md) and [Evaluator scope](../guides/evaluator-scope.md)

## Related

- [Python guide](../guides/python.md)
- [DL API](dl.md)
- [Query API](query.md)
- [Explain API](explain.md)
- [CLI reference](cli.md)
- [Errors](errors.md)
- [Examples gallery](../examples/index.md)
