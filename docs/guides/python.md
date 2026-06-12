# Python Guide

Alpha Python bindings for OntoLogos v0.5 via PyO3 (`pip install ontologos`).

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

Development install from a clone:

```bash
cd crates/ontologos-py
python -m venv .venv
source .venv/bin/activate
pip install 'maturin>=1.7,<2.0' pytest
maturin develop --release
pytest tests/ -q
```

## Quick start

```python
import ontologos

print(ontologos.__version__)

# RDFS TBox materialization
reasoner = ontologos.Reasoner("ontology.owl", profile="rdfs")
report = reasoner.classify()
print(report["inferred_axioms"])

# OWL RL saturation (RDFS + RL rules)
reasoner = ontologos.Reasoner("family.owl", profile="rl")
report = reasoner.classify()

# OWL EL taxonomy
reasoner = ontologos.Reasoner("pizza.owl", profile="el")
taxonomy = reasoner.classify()
print(taxonomy["subsumption_count"])

# Auto-detect profile (EL or RL)
reasoner = ontologos.Reasoner("ontology.owl", profile="auto")
result = reasoner.classify()
```

## API reference

### `Reasoner(path, profile=None)`

Constructs a reasoner by loading an OWL file via the Rust parser.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `path` | `str` | required | Path to `.owl`, `.rdf`, `.ttl`, or `.ofn` file |
| `profile` | `str` or `None` | `"auto"` | `"auto"`, `"rdfs"`, `"rl"`, or `"el"` |

**Profiles:**

| Profile | `classify()` return value |
|---------|--------------------------|
| `"rdfs"` | `dict` with `initial_axiom_count`, `final_axiom_count`, `inferred_axioms` |
| `"rl"` | Same report shape as RDFS (includes RL inferences) |
| `"el"` | `dict` with `subsumption_count`, `subsumptions`, `equivalences`, `unsatisfiable` |
| `"auto"` | EL taxonomy or RL report based on profile detection |

Invalid profile strings raise `RuntimeError`.

### `classify() -> dict`

Runs the selected profile engine on the loaded ontology.

- **EL / auto (EL):** Returns taxonomy dict; also available via `reasoner.taxonomy` after classify.
- **RDFS / RL:** Returns materialization report dict with axiom counts.

### `parse_meta` (property)

Read-only dict after load:

| Key | Type | Description |
|-----|------|-------------|
| `warnings` | `list[str]` | Parser mapping warnings |
| `mapped_axiom_count` | `int` | Axioms stored in core |
| `skipped_axiom_count` | `int` | Logical components not mapped |
| `logical_axiom_count` | `int` | Mapped + skipped |

## Limitations (v0.5 alpha)

| Capability | Rust v0.5 | Python v0.5 |
|------------|-----------|-------------|
| Load OWL files | Yes | Yes |
| Profile detection | Yes | Via `"auto"` only |
| RDFS / RL / EL classify | Yes | Yes |
| Per-rule RL report | Yes | Counts only |
| Export saturated ontology | Yes (in-process) | No |
| Query API | Yes (`ontologos-query`) | No |

## Errors

All failures surface as `RuntimeError` with a string message. Common messages:

| Message pattern | Cause |
|-----------------|-------|
| `unsupported profile` | Invalid profile string |
| `classification not supported for profile` | Auto-routing hit DL-only ontology |
| Parse / I/O errors | Bad path, unsupported format, mapping failure |

## Related

- [Getting started](../getting-started/index.md)
- [OWL EL classification](../getting-started/owl-el-classification.md)
- [Migration v0.4 → v0.5](../migration/v0.4.x-to-v0.5.0.md)
