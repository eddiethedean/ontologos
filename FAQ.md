# Frequently Asked Questions

## Why does `Ontology::from_file` fail?

`Ontology::from_file` on `ontologos-core` intentionally returns `Error::ParseNotAvailable` to keep the core crate free of parser dependencies.

**Load OWL/RDF files with `ontologos-parser`:**

```rust
use ontologos_parser::load_ontology;

let ontology = load_ontology(path::Path::new("ontology.owl"))?;
```

Or use the CLI: `ontologos profile ontology.owl`

See [Load an OWL file](https://ontologos.readthedocs.io/en/latest/getting-started/load-owl-file/).

## Which crate should I depend on?

For **v0.7.0**, typical workflows use:

```toml
[dependencies]
ontologos-core = "0.7.0"
ontologos-parser = "0.7.0"   # OWL/RDF file loading
ontologos-profile = "0.7.0"  # EL / RL / QL / DL detection
ontologos-rdfs = "0.7.0"     # RDFS materialization
ontologos-rl = "0.7.0"       # OWL RL saturation
ontologos-el = "0.7.0"       # OWL EL classification
ontologos-explain = "0.7.0"  # Proof graphs
```

Depend on **`ontologos-core` only** if you build ontologies programmatically or from JSON snapshots.

There is no umbrella `ontologos` crate on crates.io. The CLI binary is built from `ontologos-cli` in this repository.

## Can I use OntoLogos instead of Protégé + HermiT today?

**Not for full OWL DL classification.** v0.7.0 loads OWL files, detects profiles, materializes RDFS/RL via reasonable, classifies OWL EL taxonomies via in-house completion, and builds proof graphs via `ontologos-explain`. CLI `classify --profile auto|el|rl|rdfs` routes to the correct engine. Use Protégé with HermiT or Konclude for production OWL DL workflows.

OntoLogos is for early adopters who want to embed the Rust data model, load ontologies natively, run RL saturation, or follow the [roadmap](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md).

## Why was my JSON rejected?

Common causes:

- **`format_version: 1`** — v1 is rejected for untrusted input; use [JSON v2](https://ontologos.readthedocs.io/en/latest/json-snapshot-v2/)
- **Invalid IRI** — only `http`, `https`, and `urn` schemes; no control characters
- **Unknown entity IRI in axioms** — declare all entities before referencing them in axioms
- **Size limits** — default max JSON size is 16 MiB; see [Security](https://ontologos.readthedocs.io/en/latest/security/)

## Why does Pizza detect as DL?

Profile **classification** uses mapped TBox shapes (`parse_meta.profile_constructs`). The Pizza corpus mixes EL shapes (existentials) with constructs that rule out EL and RL (e.g. inverse and functional object properties), so detection reports **DL**. **Diagnostics** explain which mapped constructs violate EL/RL profile rules and may also list constructs seen in the source but not stored in core (e.g. `ObjectAllValuesFrom`).

See [Profile detection](https://ontologos.readthedocs.io/en/latest/guides/profile-detection/).

## Why doesn't `ontology.axiom_count()` match Protégé's axiom count?

The parser maps a subset of OWL constructs into the core model. Complex class expressions, many data-property axioms, and some property axioms are scanned for profile detection but **skipped** during mapping. Named ABox axioms (`ClassAssertion`, `ObjectPropertyAssertion`, `SameIndividual`, `DifferentIndividuals`) are mapped in v0.4. `axiom_count()` is **mapper output**, not raw OWL logical axiom count.

See [Protégé vs OntoLogos counts](https://ontologos.readthedocs.io/en/latest/guides/protege-axiom-counts/), [Troubleshooting](https://ontologos.readthedocs.io/en/latest/guides/troubleshooting/), and [Supported constructs](https://ontologos.readthedocs.io/en/latest/reference/supported-constructs/).

## What is the difference between ROADMAP.md and PLAN.md?

**[ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md)** is the canonical semver release plan. **[PLAN.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/PLAN.md)** is historical background and ecosystem vision; prefer ROADMAP for current status.

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

- Hosted: [docs.rs/ontologos-core](https://docs.rs/ontologos-core/0.7.0), [docs.rs/ontologos-parser](https://docs.rs/ontologos-parser/0.7.0), [docs.rs/ontologos-profile](https://docs.rs/ontologos-profile/0.7.0), [docs.rs/ontologos-rdfs](https://docs.rs/ontologos-rdfs/0.7.0), [docs.rs/ontologos-rl](https://docs.rs/ontologos-rl/0.7.0)
- Guides: [Choosing an API](https://ontologos.readthedocs.io/en/latest/guides/choosing-an-api/) · [Architecture](https://ontologos.readthedocs.io/en/latest/architecture/) · [OWL RL](https://ontologos.readthedocs.io/en/latest/getting-started/owl-rl-saturation/)
- Local: `cargo doc -p ontologos-core --open`
- Error catalog: [Error reference](https://ontologos.readthedocs.io/en/latest/reference/errors/)
- CLI: [CLI reference](https://ontologos.readthedocs.io/en/latest/reference/cli/)

## Does `pip install ontologos` work?

The PyPI package is an **alpha** release (v0.7.0). It supports `profile="auto"`, `"el"`, `"rl"`, and `"rdfs"`:

```python
Reasoner("ontology.owl", profile="rdfs").classify()  # RDFS materialization
Reasoner("ontology.owl", profile="rl").classify()    # OWL RL saturation
```

See [Python guide](https://ontologos.readthedocs.io/en/latest/guides/python/) for the full API and limitations.

## Where do I ask questions?

Open a [GitHub issue](https://github.com/eddiethedean/ontologos/issues) for bugs, feature requests, or design questions. Check this FAQ and [Troubleshooting](https://ontologos.readthedocs.io/en/latest/guides/troubleshooting/) first.
