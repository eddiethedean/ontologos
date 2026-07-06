# Incremental reasoning

Incremental re-classification and materialization when ontologies change between runs. **Stable on v1.1.0** (introduced in v0.8).

## Enable incremental mode

Set `ReasonerConfig::incremental = true` (default `false` preserves batch behavior).

```rust
use ontologos_core::{Profile, Reasoner, ReasonerConfig};
use ontologos_el::classify_reasoner;

let mut reasoner = Reasoner::builder()
    .profile(Profile::El)
    .config(ReasonerConfig {
        incremental: true,
        ..ReasonerConfig::default()
    })
    .build(ontology)?;

classify_reasoner(&mut reasoner)?;

// Edit ontology (add_axiom marks dirty)
reasoner.ontology_mut().add_axiom(new_axiom)?;
classify_reasoner(&mut reasoner)?; // incremental path when session is warm
```

**CLI / Python:** The `--incremental` flag and `incremental=True` enable session mode, but **single-shot** file classify/materialize still runs one pass only. Incremental benefit requires a **library workflow**: load once, classify/materialize, edit axioms, classify/materialize again.

**Python:**

```python
from ontologos import OntologyBuilder, Reasoner

builder = OntologyBuilder()
builder.add_class("http://example.org/A")
builder.add_class("http://example.org/B")
builder.add_class("http://example.org/C")
builder.subclass_of("http://example.org/A", "http://example.org/B")
reasoner = Reasoner(ontology=builder.build(), profile="el", incremental=True)
reasoner.classify()
reasoner.add_subclass_of("http://example.org/B", "http://example.org/C")
reasoner.classify()
```

See [Python guide](python.md#incremental-mutations).

## Engines

| Profile | Strategy |
|---------|----------|
| **EL** | Persisted `CompletionGraph` + partition overdelete-rederive (Kazakov ISWC 2013 style) |
| **RL / RDFS** | Persistent `reasonable::Reasoner`; delta triples on add-only edits; full rematerialize on removal |

## Limitations

- **Axiom removal:** EL falls back to full classify; RL/RDFS strip inferred axioms then cold rematerialize (asserted-only base triples).
- **Large edits:** EL falls back to full classify when >50% of partitions are affected.
- **CLI single-shot:** `--incremental` on one file load does not speed up batch runs; use the Rust session API or wait for CLI `--watch` (v1.2).
- **File watch:** `ontologos-watch` reloads OWL files for Ontocode; CLI `--watch` is v1.2.

## Conformance

- Correctness: `cargo test -p ontologos-el --test incremental_correctness`
- Performance gate (local): `./benchmarks/scripts/bench-el-incremental.sh` (≥5× on 10-axiom delta)

See [performance.md](performance.md) and [migration/v0.7.x-to-v0.8.0.md](../migration/v0.7.x-to-v0.8.0.md).
