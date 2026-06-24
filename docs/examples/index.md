# Examples gallery

Copy-paste workflows for Rust and Python. Install pins use **published v0.9.0** unless noted.

## Download a sample ontology

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
```

## Rust

### RDFS materialization

See [Getting started — crates.io only](../getting-started/index.md#cratesio-only-no-clone). Crates: `ontologos-parser`, `ontologos-rdfs`.

### Classify (facade)

[Classify quick start](../getting-started/classify-quickstart.md) — `ontologos-facade::classify` with `Profile::Auto`.

### OWL EL taxonomy

```rust
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;

let ontology = load_ontology("family.owl".as_ref())?;
let taxonomy = ElClassifier::new().classify(&ontology)?;
```

For Pizza EL golden tests, clone the repo and run `./benchmarks/scripts/download.sh`.

### JSON snapshot round-trip

```rust
let json = ontology.to_json()?;
let restored = ontologos_core::Ontology::from_json(&json)?;
```

See [JSON snapshot v2](../json-snapshot-v2.md).

### Repository examples (clone required)

| Example | Command |
|---------|---------|
| Builder API | `cargo run -p ontologos-core --example pizza_builder` |
| Load + profile | `cargo run -p ontologos-parser --example load_and_profile` |
| RL saturation | `cargo run -p ontologos-rl --example rl_saturation` |

## Python

### Classify from file

```bash
pip install ontologos
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
```

```python
from ontologos import Reasoner

report = Reasoner(path="family.owl", profile="rl").classify()
print(report)
```

### In-memory builder + incremental

```python
from ontologos import OntologyBuilder, Reasoner

b = OntologyBuilder()
b.add_class("http://example.org/Food")
b.add_class("http://example.org/Pizza")
b.subclass_of("http://example.org/Pizza", "http://example.org/Food")
r = Reasoner(ontology=b.build(), profile="el", incremental=True)
r.classify()
r.add_subclass_of("http://example.org/VeggiePizza", "http://example.org/Pizza")
r.classify()
```

### Explain

```python
graph = Reasoner(path="family.owl", profile="el").explain()
print(graph["node_count"])
```

### Pandas export (optional)

```bash
pip install 'ontologos[pandas]'
```

```python
df = Reasoner(path="family.owl", profile="el").taxonomy_dataframe()
```

## CLI (clone or `cargo install --git ...`)

```bash
ontologos classify --profile auto benchmarks/data/family.owl
ontologos materialize benchmarks/data/family.owl
ontologos explain --profile el benchmarks/data/pizza.owl   # after download.sh
```

See [CLI reference](../reference/cli.md) and [Evaluator playbook](../guides/evaluator-playbook.md).

## Related

- [Start here](../guides/start-here.md)
- [Choosing an API](../guides/choosing-an-api.md)
