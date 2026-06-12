# Frequently Asked Questions

## Why does `Ontology::from_file` fail?

`Ontology::from_file` on `ontologos-core` intentionally returns `Error::ParseNotAvailable` to keep the core crate free of parser dependencies.

**Load OWL/RDF files with `ontologos-parser`:**

```rust
use ontologos_parser::load_ontology;

let ontology = load_ontology(path::Path::new("ontology.owl"))?;
```

Or use the CLI: `ontologos profile ontology.owl`

See [Load an OWL file](docs/getting-started/load-owl-file.md).

## Which crate should I depend on?

For **v0.3**, typical workflows use:

```toml
[dependencies]
ontologos-core = "0.3"
ontologos-parser = "0.3"   # OWL/RDF file loading
ontologos-profile = "0.3"  # EL / RL / QL / DL detection
ontologos-rdfs = "0.3"     # RDFS materialization
```

Depend on **`ontologos-core` only** if you build ontologies programmatically or from JSON snapshots.

There is no umbrella `ontologos` crate on crates.io. The CLI binary is built from `ontologos-cli` in this repository.

## Can I use OntoLogos instead of Protégé + HermiT today?

**Not for full OWL classification.** v0.3 loads OWL files, detects profiles, and materializes RDFS TBox inferences (`ontologos materialize` or `ontologos_rdfs::RdfsEngine`). OWL EL/RL classification is not yet available — use Protégé with HermiT or ELK for production OWL reasoning workflows.

OntoLogos is for early adopters who want to embed the Rust data model, load ontologies natively, or follow the [roadmap](ROADMAP.md).

## Why was my JSON rejected?

Common causes:

- **`format_version: 1`** — v1 is rejected for untrusted input; use [JSON v2](docs/json-snapshot-v2.md)
- **Invalid IRI** — only `http`, `https`, and `urn` schemes; no control characters
- **Unknown entity IRI in axioms** — declare all entities before referencing them in axioms
- **Size limits** — default max JSON size is 16 MiB; see [security.md](docs/security.md)

## Why does Pizza detect as EL but diagnostics mention DL constructs?

Profile **classification** uses mapped TBox shapes (`parse_meta.profile_constructs`). **Diagnostics** also flag constructs observed in the full parse that fall outside the detected profile. Pizza is detected as **EL** based on mapped axioms, while diagnostics may list constructs such as `ObjectAllValuesFrom` seen in the source but not stored in core.

See [Profile detection](docs/guides/profile-detection.md).

## Why doesn't `ontology.axiom_count()` match Protégé's axiom count?

The parser maps a subset of OWL constructs into the core model. Complex class expressions, ABox axioms, and many property axioms are scanned for profile detection but **skipped** during mapping. `axiom_count()` is **mapper output**, not raw OWL logical axiom count.

See [Troubleshooting](docs/guides/troubleshooting.md) and [Supported constructs](docs/reference/supported-constructs.md).

## What is the difference between ROADMAP.md and PLAN.md?

**[ROADMAP.md](ROADMAP.md)** is the canonical semver release plan. **[PLAN.md](PLAN.md)** is historical background and ecosystem vision; prefer ROADMAP for current status.

## Is OntoLogos the same as Ontologos?

**Display name:** OntoLogos. **Crate and command names:** `ontologos-*` and `ontologos` (lowercase).

## How do I load the pizza test fixture?

**From JSON (no download):**

```rust
let json = include_str!("../tests/fixtures/pizza_minimal.json");
let ontology = Ontology::from_json(json)?;
```

**From OWL (benchmark corpus):**

```bash
./benchmarks/scripts/download.sh
```

```rust
let ontology = ontologos_parser::load_ontology(path::Path::new("benchmarks/data/pizza.owl"))?;
```

Or run `cargo run -p ontologos-core --example pizza_builder`.

## Where is the API reference?

- Hosted: [docs.rs/ontologos-core](https://docs.rs/ontologos-core), [docs.rs/ontologos-parser](https://docs.rs/ontologos-parser), [docs.rs/ontologos-profile](https://docs.rs/ontologos-profile)
- Local: `cargo doc -p ontologos-core --open`
- Error catalog: [docs/reference/errors.md](docs/reference/errors.md)
- CLI: [docs/reference/cli.md](docs/reference/cli.md)

## Does `pip install ontologos` work?

The PyPI package is an **alpha placeholder** (v0.3). It installs, reports its version, and `Reasoner(path)` loads an OWL file via the Rust parser — but `classify()` returns not-implemented until v0.5. Profile detection, materialize, and full Python APIs ship in later milestones (see [Python README](crates/ontologos-py/README.md)). Use the Rust crates for v0.3 workflows.
