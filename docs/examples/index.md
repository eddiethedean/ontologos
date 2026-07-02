# Examples gallery

Copy-paste workflows for Rust and Python. Install pins use **published v0.9.0** unless noted.

Workflow pages: [DL evaluation](dl-evaluation.md) · [Contract tests](contract-tests.md)

## Download a sample ontology

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
```

## Rust

### RDFS materialization

See [Getting started — crates.io only](../getting-started/index.md#cratesio-only-no-clone). Crates: `ontologos-parser`, `ontologos-rl` (`ontologos_rl::rdfs`).

### Classify (facade)

[Classify quick start](../getting-started/classify-quickstart.md) — `ontologos-facade::classify` with `Profile::Auto`.

### OWL EL taxonomy (in-memory — no Pizza download)

Family.owl is an **RL** corpus — do not use it for EL demos.

```rust
use ontologos_core::Ontology;
use ontologos_el::ElClassifier;

let ontology = Ontology::builder()
    .class("http://example.org/Food")?
    .class("http://example.org/Pizza")?
    .subclass_of("http://example.org/Pizza", "http://example.org/Food")?
    .build()?;
let taxonomy = ElClassifier::new().classify(&ontology)?;
println!("subsumptions: {}", taxonomy.subsumption_count());
```

For Pizza EL golden tests, clone the repo and run `./benchmarks/scripts/download.sh`. See [OWL EL classification](../getting-started/owl-el-classification.md).

### JSON snapshot round-trip

```rust
use ontologos_core::Ontology;

let ontology = Ontology::builder()
    .class("http://example.org/A")?
    .build()?;
let json = ontology.to_json()?;
let restored = Ontology::from_json(&json)?;
```

See [JSON snapshot v3](../json-snapshot-v3.md) (writers emit v3; v2 still readable).

### Repository examples (clone required)

| Example | Command |
|---------|---------|
| Builder API | `cargo run -p ontologos-core --example pizza_builder` |
| Load + profile | `cargo run -p ontologos-parser --example load_and_profile` |
| RL saturation | `cargo run -p ontologos-rl --example rl_saturation` |
| Facade auto classify | `cargo run -p ontologos-facade --example facade_auto -- benchmarks/data/family.owl` |

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
# EL explain works best on EL corpora (e.g. pizza.owl after download.sh)
graph = Reasoner(path="family.owl", profile="rl").explain()
print(graph["node_count"])
```

### Pandas export (optional)

```bash
pip install 'ontologos[pandas]'
```

```python
from ontologos import Reasoner, subsumptions_to_pandas

# EL taxonomy export (requires pizza.owl — see download.sh or curl from repo)
report = Reasoner(path="pizza.owl", profile="el").classify()
df = subsumptions_to_pandas(report)
```

## CLI (clone or `cargo install --git ...`)

```bash
ontologos classify --profile auto benchmarks/data/family.owl
ontologos materialize benchmarks/data/family.owl
ontologos explain --profile el benchmarks/data/pizza.owl   # after download.sh
```

See [CLI reference](../reference/cli.md) and [Evaluator playbook](../guides/evaluator-playbook.md).

## OWL DL evaluation (`main` / 1.0.0)

Requires build from `main` — not available on PyPI **0.9.0**. See [Install channels](../guides/install-channels.md).

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh
cargo build -p ontologos-cli --release
./target/release/ontologos classify --profile dl --budget-secs 30 benchmarks/data/pizza.owl
```

```python
from ontologos import Reasoner

reasoner = Reasoner(path="benchmarks/data/pizza.owl", profile="dl", budget_secs=30)
consistency = reasoner.check_consistency()
assert consistency["complete"] and consistency["consistent"]
report = reasoner.classify()
```

Full workflow: [Evaluator playbook](../guides/evaluator-playbook.md) · [DL API reference](../reference/dl.md).

## Contract tests (contributors / evaluators)

Public API contract (facade-routed, no HermiT JAR required):

```bash
cargo test -p ontologos-contract --release
```

Sample case IDs: `crates/ontologos-contract/data/case_ids.txt`. See [Conformance](../reference/conformance.md).

## Channel-aware installs

| Goal | Install |
|------|---------|
| EL / RL / RDFS production | `pip install ontologos==0.9.0` or `ontologos-* = "0.9.0"` |
| DL / SWRL | Clone `main`, pin `"1.0.0"` on all workspace crates |
| CLI | `cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli` |

See [Install channels](../guides/install-channels.md).

## Related

- [Start here](../guides/start-here.md)
- [Choosing an API](../guides/choosing-an-api.md)
