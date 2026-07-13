# Explain API Reference

Proof graphs and explanation export via [`ontologos-explain`](https://docs.rs/ontologos-explain/1.1.4).

## Overview

| Surface | Entry point |
|---------|-------------|
| Rust | `explain_with_profile`, `build_proof_graph`, `render_text` |
| CLI | `ontologos explain --profile el\|rl\|rdfs\|auto ontology.owl` |
| Python | `Reasoner.explain() -> dict` |

## Trace coverage by profile

| Profile | Coverage |
|---------|----------|
| **EL** | Full inference traces from in-house completion → proof graph with IRI-resolved conclusions |
| **RL / RDFS** | Proof graph seeds **asserted** axioms; inferred steps lack per-rule premises until reasonable exposes a trace API |
| **auto** | Routes like `classify`; on `main` / 1.0.0, DL-detected ontologies use the DL engine (preview traces on `dl-preview` / `alc`) |

## Rust API

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_explain::{explain_with_profile, render_text};
use ontologos_parser::load_ontology;

let ontology = load_ontology(path)?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::El)
    .build(ontology)?;

let graph = explain_with_profile(&mut reasoner)?;
let text = render_text(&graph, &reasoner.ontology())?;
println!("{text}");
```

### `ProofGraph` fields

| Field | Type | Description |
|-------|------|-------------|
| `nodes` | `Vec<ProofNode>` | Steps in topological construction order |

### `ProofNode` fields

| Field | Description |
|-------|-------------|
| `rule` | Rule name that produced the step |
| `premises` | Premise node IDs in the graph |
| `conclusion_axiom` | Concluded axiom ID (when applicable) |
| `conclusion_sub` | EL subsumption `(sub, sup)` entity IDs |
| `conclusion_existential` | EL existential `(class, property, filler)` |
| `conclusion_subproperty` | EL subproperty `(sub, sup)` |

Export JSON:

```rust
let json = graph.to_json()?;
```

Query helpers: `explain_subsumption`, `explain_unsatisfiable`, `find_subsumption_step`.

## CLI

```bash
./target/release/ontologos explain --profile el benchmarks/data/pizza.owl
./target/release/ontologos --format json explain --profile el benchmarks/data/pizza.owl
```

Text output lists proof steps with rule names and premise references. JSON output is a serialized `ProofGraph`.

## Python

```python
from ontologos import Reasoner

reasoner = Reasoner(path="pizza.owl", profile="el")
reasoner.classify()
graph = reasoner.explain()
print(graph["node_count"])
for step in graph["nodes"]:
    print(step["rule"], step.get("premises"))
```

Dict shape:

| Key | Type | Description |
|-----|------|-------------|
| `node_count` | `int` | Number of proof steps |
| `nodes` | `list[dict]` | Steps with `rule`, `premises`, optional conclusion fields |
| `parse_meta` | `dict` | Present when parser warnings exist |

## Errors

| Error | Cause |
|-------|-------|
| `UnsupportedProfile` | DL/QL ontology with no explain engine |
| `NotImplemented` | Reserved for future profiles |
| `Profile` | Profile detection failed |

See [Error reference](errors.md) and [Conformance](conformance.md).

## Related

- [OWL EL classification](../getting-started/owl-el-classification.md)
- [CLI reference](cli.md)
- [Python guide](../guides/python.md)
- [Reasonable adapter limits](reasonable-limits.md) — RL/RDFS trace gaps
