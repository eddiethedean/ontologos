# Python API Reference

Python bindings for OntoLogos via PyPI [`ontologos`](https://pypi.org/project/ontologos/).

Narrative guide: [Python guide](../guides/python.md). Install channels: [Install and channels](../guides/install-channels.md).

## Install

```bash
pip install ontologos
pip install 'ontologos[pandas]'   # optional DataFrame export
pip install 'ontologos[polars]'    # optional polars export
```

Requires **Python 3.10+**.

## Profiles by channel

| Profile | PyPI 0.9.0 | `main` / 1.0.0 |
|---------|------------|----------------|
| `"auto"`, `"rdfs"`, `"rl"`, `"el"` | Supported | Supported |
| `"dl"`, `"swrl"` | Not available | Supported |
| `"alc"`, `"dl-preview"` | Preview (errors common) | Preview |

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
| `profile` | See table above |
| `incremental` | Reuse session across `classify()` calls after mutations |
| `budget_secs` | Wall-clock budget for DL consistency/classify |

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `classify()` | `dict` | Run profile engine; taxonomy or materialization report |
| `explain()` | `dict` | Proof graph (EL full; RL/RDFS asserted-only) |
| `add_subclass_of(sub, sup)` | `None` | Incremental mutation |
| `remove_subclass_of(sub, sup)` | `None` | Incremental mutation |
| `add_axiom_json(axiom)` | `None` | Add axiom from JSON dict |

### Properties

| Property | Description |
|----------|-------------|
| `taxonomy` | Last taxonomy dict after EL/DL `classify()` |
| `parse_meta` | Parser metadata dict after file load |

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

## `Ontology`

In-memory ontology wrapper. Construct via `OntologyBuilder` or `Ontology.from_dict`.

## Export functions

```python
from ontologos import Reasoner, subsumptions_to_pandas, subsumptions_to_polars

taxonomy = Reasoner(path="family.owl", profile="auto").classify()
df = subsumptions_to_pandas(taxonomy)   # requires ontologos[pandas]
```

There is no `taxonomy_dataframe()` method — use `subsumptions_to_pandas(taxonomy)` or `subsumptions_to_polars(taxonomy)`.

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
- Subproperty / property-value queries: CLI only (see [Python guide](../guides/python.md))
- DL/SWRL on PyPI 0.9.0: build from `main`

## Related

- [Python guide](../guides/python.md)
- [Query API](query.md)
- [Explain API](explain.md)
- [CLI reference](cli.md)
- [Examples gallery](../examples/index.md)
