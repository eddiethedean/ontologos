# Load an OWL File

Load OWL and RDF serializations into the core ontology model via [`ontologos-parser`](https://docs.rs/ontologos-parser/0.9.0).

> **Important:** OntoLogos maps a **subset** of OWL axioms into its core model. `axiom_count()` reflects mapped axioms, not Protégé's total. See [Known limitations](../guides/known-limitations.md) and [Supported constructs](../reference/supported-constructs.md).

!!! note "`owl:imports` behavior"
    Import handling is **format-dependent**. `load_ontology()` merges **local** RDF/XML imports (`merge_imports: true`). `ParseLimits::default()` has `merge_imports: false`. Turtle and OWL Functional do not merge imports. Remote URLs are never fetched.

    See [OWL imports reference](../reference/owl-imports.md).

## Prerequisites

- Rust 1.88+
- A sample file (download below) or a clone for benchmark corpora

## Supported formats

| Extension | Format |
|-----------|--------|
| `.owl` | OWL/XML or RDF/XML (content sniffed) |
| `.rdf`, `.xml` | RDF/XML |
| `.ttl`, `.turtle` | Turtle |
| `.ofn`, `.func` | OWL Functional Syntax |

## CLI

```bash
./benchmarks/scripts/download.sh
cargo build -p ontologos-cli --release
./target/release/ontologos profile benchmarks/data/pizza.owl
```

Expected output for Pizza (abbreviated):

```text
detected profile: Dl
diagnostics:
  - InverseObjectProperties: construct is outside OWL 2 EL (mapped axiom rules out OWL 2 EL)
  - SubClassOfExistential: construct is outside OWL 2 RL (mapped axiom rules out OWL 2 RL)
  ...
```

Family ontology typically reports `detected profile: Rl`.

## Library

!!! note "Path convention"
    **Crates.io quick start:** download `family.owl` to the current directory (see [Getting started](index.md#cratesio-only-no-clone)) and use `family.owl`.

    **Repository clone:** use `benchmarks/data/family.owl` after `./benchmarks/scripts/download.sh`.

Add dependencies:

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
```

Load and inspect:

```rust
use ontologos_parser::load_ontology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("family.owl");
    let ontology = load_ontology(path)?;

    println!("entities: {}", ontology.entity_count());
    println!("axioms (mapped): {}", ontology.axiom_count());

    if let Some(meta) = ontology.parse_meta() {
        println!("mapped: {}", meta.mapped_axiom_count);
        println!("skipped: {}", meta.skipped_axiom_count);
        for warning in &meta.warnings {
            println!("warning: {warning}");
        }
    }

    Ok(())
}
```

Run the repository example:

```bash
./benchmarks/scripts/download.sh
cargo run -p ontologos-parser --example load_and_profile
```

## Parse metadata

After loading, `ontology.parse_meta()` contains:

| Field | Meaning |
|-------|---------|
| `constructs` | All OWL constructs observed in the source |
| `profile_constructs` | Constructs from successfully mapped axioms |
| `mapped_axiom_count` | Axioms stored in core (equals `axiom_count()`) |
| `skipped_axiom_count` | Logical components not mapped |
| `logical_axiom_count` | `mapped + skipped` |
| `warnings` | Skipped shapes and mapping messages |

Not every construct in the file becomes a core axiom. See [Supported constructs](../reference/supported-constructs.md).

## OWL imports

See [OWL imports reference](../reference/owl-imports.md) for the full format matrix.

**RDF/XML:** local `owl:imports` are merged by default. Disable with `ParseLimits { merge_imports: false, .. }`.

**Turtle / OWL Functional:** imports are not merged — use a single bundle file or merge upstream (e.g. ROBOT).

**Remote import IRIs:** never fetched over the network.

## Untrusted files

Use custom limits for uploads:

```rust
use ontologos_parser::{load_ontology_with_limits, ParseLimits};

let limits = ParseLimits {
    max_file_bytes: 1_048_576, // 1 MiB
    ..ParseLimits::default()
};
let ontology = load_ontology_with_limits(path, limits)?;
```

See [Security](../security.md).

## Why not `Ontology::from_file`?

`ontologos-core` deliberately does not depend on the parser. `Ontology::from_file` returns `Error::ParseNotAvailable`. Always use `ontologos_parser::load_ontology`.

## Next steps

- [RDFS materialization](rdfs-materialization.md)
- [OWL RL saturation](owl-rl-saturation.md)
- [Profile detection](../guides/profile-detection.md)
- [Troubleshooting](../guides/troubleshooting.md)
- [Error reference](../reference/errors.md)
