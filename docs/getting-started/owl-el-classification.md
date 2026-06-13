# OWL EL classification

v0.5 adds completion-based **OWL EL taxonomy classification** via `ontologos-el`.

## Library

```rust
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;

let ontology = load_ontology(std::path::Path::new("ontology.owl"))?;
let taxonomy = ElClassifier::new().classify(&ontology)?;
println!("direct subsumptions: {}", taxonomy.subsumption_count());
```

Query the taxonomy with `ontologos-query`:

```rust
use ontologos_query::QueryEngine;

let engine = QueryEngine::new(&ontology, &taxonomy);
let subs = engine.direct_subclasses(class_id)?;
```

## CLI

```bash
ontologos classify --profile el benchmarks/data/pizza.owl
ontologos classify --profile auto family.owl   # detect EL vs RL
ontologos materialize ontology.owl             # explicit RDFS (unchanged)
```

`classify` with default `--profile auto` routes to EL or RL based on profile detection. Use `--profile rdfs` for RDFS materialization via the classify entry point, or `materialize` for explicit RDFS.

## Python

```python
from ontologos import Reasoner

r = Reasoner("pizza.owl", profile="el")
taxonomy = r.classify()
print(taxonomy["subsumption_count"])
```

## Limitations

- Classifies mapped EL TBox axioms only; complex DL constructs remain skipped by the parser.
- Hybrid ontologies (EL + RL) should use an explicit `--profile` flag.
- Explanations for EL inferences are available via `ontologos-explain` and CLI `explain` (v0.7.0).

See [Supported constructs](../reference/supported-constructs.md) and [migration guide](../migration/v0.4.x-to-v0.5.0.md).
