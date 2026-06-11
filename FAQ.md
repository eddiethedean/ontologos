# Frequently Asked Questions

## Why does `Ontology::from_file` fail?

v0.1 ships the in-memory data model only. File parsing lands in **v0.2** via `ontologos-parser`. Until then, use:

- `Ontology::builder()` for programmatic construction
- `Ontology::from_json()` for JSON v2 snapshots

The error is `Error::ParseNotAvailable`, not a bug.

## Which crate should I depend on?

For v0.1, depend on **`ontologos-core`** only:

```toml
[dependencies]
ontologos-core = "0.1"
```

Other workspace crates (`ontologos-parser`, `ontologos-el`, …) are stubs or internal until their roadmap milestones complete. There is no umbrella `ontologos` crate on crates.io.

## Can I use OntoLogos instead of Protégé + HermiT today?

**Not for reasoning.** OntoLogos v0.1 does not classify ontologies or load OWL files. Use Protégé with HermiT or ELK for production OWL workflows. OntoLogos is for early adopters embedding the Rust data model or tracking the roadmap.

## Why was my JSON rejected?

Common causes:

- **`format_version: 1`** — v1 is rejected for untrusted input; use [JSON v2](docs/json-snapshot-v2.md)
- **Invalid IRI** — only `http`, `https`, and `urn` schemes; no control characters
- **Unknown entity IRI in axioms** — declare all entities before referencing them in axioms
- **Size limits** — default max JSON size is 16 MiB; see [security.md](docs/security.md)

## Why does the CLI say "parsing is not available"?

The CLI binary is wired but ontology loading requires the v0.2 parser. All subcommands fail at load until then.

## What is the difference between ROADMAP.md and PLAN.md?

**[ROADMAP.md](ROADMAP.md)** is the canonical semver release plan. **[PLAN.md](PLAN.md)** is historical background and ecosystem vision; prefer ROADMAP for current status.

## Is OntoLogos the same as Ontologos?

**Display name:** OntoLogos. **Crate and command names:** `ontologos-*` and `ontologos` (lowercase, no camel case).

## How do I load the pizza test fixture?

```rust
let json = include_str!("../tests/fixtures/pizza_minimal.json");
let ontology = Ontology::from_json(json)?;
```

Or run `cargo run -p ontologos-core --example pizza_builder`.

## Where is the API reference?

- In-source rustdoc: `cargo doc -p ontologos-core --open`
- Hosted: [docs.rs/ontologos-core](https://docs.rs/ontologos-core) after crates.io publish
- Error catalog: [docs/reference/errors.md](docs/reference/errors.md)
