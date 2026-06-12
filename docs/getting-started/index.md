# Getting Started

Five-minute success paths for common goals.

## I want to try it now

1. Clone the repo and download benchmarks:

   ```bash
   git clone https://github.com/eddiethedean/ontologos.git
   cd ontologos
   ./benchmarks/scripts/download.sh
   ```

2. Run the builder example:

   ```bash
   cargo run -p ontologos-core --example pizza_builder
   ```

3. Build the CLI and inspect an ontology:

   ```bash
   cargo build -p ontologos-cli --release
   ./target/release/ontologos profile benchmarks/data/family.owl
   ```

## I want OWL RL saturation (v0.4 headline feature)

Follow [OWL RL saturation](owl-rl-saturation.md) or run:

```bash
cargo run -p ontologos-rl --example rl_saturation
```

## I'm integrating in Rust

Read [Choosing an API](../guides/choosing-an-api.md) then the guide for your workflow:

| Goal | Guide |
|------|-------|
| Build ontologies in code | [First ontology](first-ontology.md) |
| Load OWL files | [Load an OWL file](load-owl-file.md) |
| RDFS materialization | [Load an OWL file](load-owl-file.md) + `ontologos-rdfs` |
| OWL RL saturation | [OWL RL saturation](owl-rl-saturation.md) |
| JSON snapshots | [JSON snapshot v2](../json-snapshot-v2.md) |

## I'm evaluating vs ELK / reasonable

See [Comparison with existing tools](../comparison.md) and [Conformance coverage](../reference/conformance.md).

## I'm using Python

```bash
pip install ontologos
```

See [Python guide](../guides/python.md).

## Full learning path

See the [documentation index](../index.md#learning-path).
