# Incremental reasoning (v0.8)

OntoLogos **0.8** adds incremental re-classification and materialization when ontologies change between runs.

## Enable incremental mode

Set `ReasonerConfig::incremental = true` (default `false` preserves v0.7 batch behavior).

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

**CLI:** `ontologos classify --incremental ontology.owl`  
**Python:** `Reasoner(path, profile="el", incremental=True)`

## Engines

| Profile | Strategy |
|---------|----------|
| **EL** | Persisted `CompletionGraph` + partition overdelete-rederive (Kazakov ISWC 2013 style) |
| **RL / RDFS** | Persistent `reasonable::Reasoner`; delta triples on add-only edits |

## Limitations

- **Axiom removal:** EL and RL/RDFS fall back to full re-classify / `set_base_triples` rematerialization. Stale inferred axioms may remain in core after removals (asserted vs inferred is not tracked yet).
- **Large edits:** EL falls back to full classify when >50% of partitions are affected.
- **File watch:** `ontologos-watch` reloads OWL files for Ontocode; CLI `--watch` is v1.2.

## Conformance

- Correctness: `cargo test -p ontologos-el --test incremental_correctness`
- Performance gate (local): `./benchmarks/scripts/bench-el-incremental.sh` (≥5× on 10-axiom delta)

See [performance.md](performance.md) and [migration/v0.7.x-to-v0.8.0.md](../migration/v0.7.x-to-v0.8.0.md).
