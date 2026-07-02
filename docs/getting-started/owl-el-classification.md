# OWL EL Classification

Completion-based **OWL EL taxonomy classification** via [`ontologos-el`](https://docs.rs/ontologos-el/0.9.0). The engine computes direct and indirect subsumptions, equivalence clusters, and unsatisfiable classes from mapped EL TBox axioms.

--8<-- "snippets/channel-banner.md"

## Prerequisites

- Rust 1.88+
- An EL-shaped ontology (`.owl`, `.rdf`, `.ttl`, `.ofn`) or a repository clone for benchmark examples
- For Pizza corpus: `./benchmarks/scripts/download.sh` (Family is vendored)

Verify profile before classifying:

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos profile benchmarks/data/pizza.owl
```

Expected output (abbreviated):

```text
detected profile: Dl
```

Pizza often detects as **DL** because of inverse/functional properties in the source — use `--profile el` to force EL classification on mapped axioms, or use a corpus that is EL-only.

## Run the CLI

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos classify --profile el benchmarks/data/pizza.owl
./target/release/ontologos classify --profile auto benchmarks/data/family.owl
```

`classify --profile auto` routes to EL taxonomy when detection reports EL, otherwise RL saturation. Use `--profile rdfs` for RDFS materialization, or `materialize` for explicit RDFS.

Expected text output (abbreviated):

```text
status: classified
subsumption_count: 84
equivalence_clusters: 0
unsatisfiable_classes: 0
```

JSON output:

```bash
./target/release/ontologos --format json classify --profile el benchmarks/data/pizza.owl
```

Explain inferences:

```bash
./target/release/ontologos explain --profile el benchmarks/data/pizza.owl
```

## Library (crates.io)

Add dependencies:

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
ontologos-el = "0.9.0"
ontologos-ql = "0.9.0"
```

Load and classify:

```rust
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("ontology.owl");
    let ontology = load_ontology(path)?;

    let taxonomy = ElClassifier::new().classify(&ontology)?;

    println!("subsumptions: {}", taxonomy.subsumption_count());
    for (sub, sup) in taxonomy.subsumptions() {
        println!("  {} ⊑ {}", sub, sup);
    }

    Ok(())
}
```

For EL via the reasoner wrapper, use **`ontologos_facade::classify`** or **`ontologos_el::classify_reasoner`**. See [Facade API](../guides/facade-api.md).

### Via the reasoner facade

```rust
use ontologos_core::{Profile, Reasoner, ReasonerConfig};
use ontologos_facade::{classify, ClassifyOutcome};
use ontologos_parser::load_ontology;

let ontology = load_ontology(path)?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::El)
    .config(ReasonerConfig::default())
    .build(ontology)?;

match classify(&mut reasoner)? {
    ClassifyOutcome::Taxonomy(t) => {
        println!("subsumptions: {}", t.subsumption_count());
    }
    _ => unreachable!("EL profile yields taxonomy"),
}
```

### Query the taxonomy

```rust
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;
use ontologos_ql::TaxonomyHierarchy;

let ontology = load_ontology(path)?;
let taxonomy = ElClassifier::new().classify(&ontology)?;
let hierarchy = TaxonomyHierarchy::new(&ontology, &taxonomy);

let pizza = hierarchy
    .lookup("http://www.co-ode.org/ontologies/pizza/pizza.owl#Pizza")
    .expect("class registered");
let supers = hierarchy.direct_superclasses(pizza)?;
println!("direct superclasses of Pizza: {supers:?}");
```

See [Query API reference](../reference/query.md).

## Reading the taxonomy

| Field | Meaning |
|-------|---------|
| `subsumption_count` | Number of direct and indirect `subClassOf` relationships inferred |
| `subsumptions` | Pairs `(sub, sup)` of class entity IDs |
| `equivalences` | Clusters of mutually equivalent classes |
| `unsatisfiable` | Classes inferred to be equivalent to `owl:Nothing` |

IRIs are resolved via `ontology.iri(entity_id)` or Python `reasoner.taxonomy` after `classify()`.

## Python

Download Pizza first (from a clone: `./benchmarks/scripts/download.sh`; or obtain `pizza.owl` from the benchmark corpus):

```bash
# From a repository clone:
./benchmarks/scripts/download.sh
```

```python
from ontologos import Reasoner

reasoner = Reasoner(path="pizza.owl", profile="el")
taxonomy = reasoner.classify()
print(taxonomy["subsumption_count"])

# After classify, taxonomy property is also available
print(reasoner.taxonomy["subsumptions"][:3])

graph = reasoner.explain()
print(graph["node_count"])
```

Incremental edits:

```python
reasoner = Reasoner(path="pizza.owl", profile="el", incremental=True)
reasoner.classify()
reasoner.add_subclass_of("http://example.org/VeggiePizza", "http://example.org/Pizza")
reasoner.classify()
```

See [Python guide](../guides/python.md) and [Incremental reasoning](../guides/incremental-reasoning.md).

## Limitations

- Classifies **mapped EL TBox axioms** only; complex DL constructs remain skipped by the parser.
- Hybrid ontologies (EL + RL shapes) should use an explicit `--profile` or `profile=` flag.
- QL and DL profiles are **detect-only** — no reasoning engine; `auto` errors on pure DL ontologies.
- Explanations for EL inferences are available via `ontologos-explain`, CLI `explain`, and Python `explain()`. RL/RDFS explain coverage is partial — see [Explain API](../reference/explain.md).

## Next steps

- [Choosing an API](../guides/choosing-an-api.md) — RDFS vs RL vs EL
- [Profile detection](../guides/profile-detection.md) — EL/RL/QL/DL diagnostics
- [Explain API](../reference/explain.md) — proof graphs
- [Supported constructs](../reference/supported-constructs.md)
- [Migration v0.4→v0.5](../migration/v0.4.x-to-v0.5.0.md) — CLI classify semantics change
