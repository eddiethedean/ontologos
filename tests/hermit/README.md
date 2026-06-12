# HermiT conformance porting

This directory catalogs tests ported from the [HermiT](https://github.com/owlcs/HermiT) Java reasoner into Rust integration tests.

HermiT source is **not committed** (see root `.gitignore`). Place a checkout at `HermiT/` or set:

```bash
export ONTOLOGOS_HERMIT_ROOT=/path/to/HermiT
```

## Run tests

```bash
# Tier A (always — no HermiT tree required)
cargo test -p ontologos-conformance

# Tier B (parser loads, classification goldens) — needs HermiT/
cargo test -p ontologos-conformance -- --ignored
```

## Add a port

1. Find the Java test under `HermiT/project/test/org/semanticweb/HermiT/`.
2. Add an entry to [manifest.toml](manifest.toml).
3. Implement the Rust test in `crates/ontologos-conformance/tests/`.
4. Update [hermit-replacement.md](../../docs/internal/research/hermit-replacement.md) if the capability mapping changes.

## License note

HermiT is LGPL-3.0. Ported **test logic** and **small fixture excerpts** are rewritten for OntoLogos; large ontology files are read from the local HermiT checkout at test time, not redistributed.
