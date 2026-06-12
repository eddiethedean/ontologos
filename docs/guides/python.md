# Python Guide

Alpha Python bindings for OntoLogos v0.4 via PyO3 (`pip install ontologos`).

## Install

Requires **Python 3.10+**. Pre-built wheels are published to PyPI for common Linux targets on release; other platforms may build from source (requires Rust toolchain + maturin).

```bash
pip install ontologos
```

Development install from a clone:

```bash
cd crates/ontologos-py
python -m venv .venv
source .venv/bin/activate
pip install 'maturin>=1.7,<2.0'
maturin develop --release
```

## Quick start

```python
import ontologos

print(ontologos.__version__)

# RDFS TBox materialization
reasoner = ontologos.Reasoner("ontology.owl", profile="rdfs")
reasoner.classify()

# OWL RL saturation (RDFS + RL rules)
reasoner = ontologos.Reasoner("family.owl", profile="rl")
reasoner.classify()
```

## API reference

### `Reasoner(path, profile=None)`

Constructs a reasoner by loading an OWL file via the Rust parser.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `path` | `str` | required | Path to `.owl`, `.rdf`, `.ttl`, or `.ofn` file |
| `profile` | `str` or `None` | `"auto"` | `"auto"`, `"rdfs"`, `"rl"`, or `"el"` |

**Profiles:**

| Profile | `classify()` behavior |
|---------|----------------------|
| `"rdfs"` | RDFS TBox materialization |
| `"rl"` | OWL RL saturation (includes RDFS pass) |
| `"auto"` | **Not implemented** until v0.5 — raises runtime error |
| `"el"` | **Not implemented** until v0.5 — raises runtime error |

Invalid profile strings raise `RuntimeError`.

### `classify() -> None`

Runs the selected profile engine **in place** on the loaded ontology held by the reasoner.

- **Return value:** `None` (no report object exposed in v0.4).
- **Side effect:** Inferred axioms are added to the internal ontology (not exported to Python yet).
- **Errors:** `RuntimeError` with message string on parse failure, unsupported profile, or engine errors.

To inspect results today, use `parse_meta` before/after or the Rust library for full report access.

### `parse_meta` (property)

Read-only dict after load (and still valid after `classify()`):

| Key | Type | Description |
|-----|------|-------------|
| `warnings` | `list[str]` | Parser mapping warnings |
| `mapped_axiom_count` | `int` | Axioms stored in core |
| `skipped_axiom_count` | `int` | Logical components not mapped |
| `logical_axiom_count` | `int` | Mapped + skipped |

Profile detection, JSON export, and query APIs are **not** exposed in Python until later milestones (v0.9 full API).

## Limitations (v0.4 alpha)

| Capability | Rust v0.4 | Python v0.4 |
|------------|-----------|-------------|
| Load OWL files | Yes | Yes (`Reasoner(path)`) |
| Profile detection | Yes | No |
| RDFS materialization | Yes | Yes (`profile="rdfs"`) |
| OWL RL saturation | Yes | Yes (`profile="rl"`) |
| Materialization report / rule counts | Yes | No |
| Export saturated ontology | Yes (in-process) | No |
| OWL EL classification | No (v0.5) | No |
| CLI-equivalent commands | Yes | No |

Use Rust crates for production integrations requiring reports, profile detection, or ontology export.

## Errors

All failures surface as `RuntimeError` with a string message. Common messages:

| Message pattern | Cause |
|-----------------|-------|
| `reasoning not yet implemented` | Default `"auto"` or `"el"` profile |
| `unsupported profile` | Invalid profile string |
| Parse / I/O errors | Bad path, unsupported format, mapping failure |

See [Error reference](../reference/errors.md) for Rust-side details.

## Related

- [Choosing an API](choosing-an-api.md)
- [OWL RL saturation](../getting-started/owl-rl-saturation.md)
- [Rust crate README](../../crates/ontologos-py/README.md)
- [ROADMAP](../../ROADMAP.md) — Python maturity in v0.9 / 1.4
