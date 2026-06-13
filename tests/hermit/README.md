# HermiT conformance porting

This directory catalogs tests ported from the [HermiT reasoner](https://github.com/owlcs/hermit-reasoner) (Java) into Rust integration tests under `crates/ontologos-conformance/`.

HermiT source is **not committed** (see root `.gitignore`). Place a checkout at `HermiT/` or set:

```bash
export ONTOLOGOS_HERMIT_ROOT=/path/to/hermit-reasoner
```

## Catalog (599 tests)

Every HermiT `test*` method is cataloged as one Rust test in `hermit_generated.rs`:

```bash
python3 tests/hermit/generate_catalog.py
```

This writes:

- `benchmarks/data/hermit/catalog/cases.json` — metadata per Java test
- `benchmarks/data/hermit/axioms/*.ofn` — extracted functional-syntax fixtures
- `crates/ontologos-conformance/tests/hermit_generated.rs` — one `#[test]` per case

**Runnable today** (no `#[ignore]`):

| Status | Meaning |
|--------|---------|
| `axiom` | OFN fixture + subsumption assertions (RDFS/RL) |
| `fixture` | EL classification golden (`pizza.xml` only in default CI) |
| `ported` | Hand-written in `hermit_rl.rs`, `hermit_rdfs.rs`, or `hermit_el.rs` |

**Ignored until 1.0** (`planned`, `internal`, `deferred`, `excluded`): DL tableau, OWL WG, SWRL, parser-blocked RDF/XML goldens (galen, propreo, wine, dolce).

Hand-written ports are listed in [manifest.toml](manifest.toml) and implemented in dedicated test modules.

## Run tests

```bash
# Tier A + catalog smoke (no HermiT tree required)
cargo test -p ontologos-conformance

# Include ignored ports (DL stubs, optional HermiT tree fixtures)
cargo test -p ontologos-conformance -- --ignored
```

## Add a hand-written port

1. Find the Java test under `HermiT/src/test/java/org/semanticweb/HermiT/`.
2. Add an entry to [manifest.toml](manifest.toml) and `IMPLEMENTED` in [generate_catalog.py](generate_catalog.py).
3. Implement the Rust test in `crates/ontologos-conformance/tests/hermit_*.rs`.
4. Re-run `generate_catalog.py` so the generated stub is marked `#[ignore]`.
5. Update [hermit-replacement.md](../../docs/internal/research/hermit-replacement.md) if the capability mapping changes.

## License note

HermiT is LGPL-3.0. Ported **test logic** and **small fixture excerpts** are rewritten for OntoLogos; large ontology files are vendored under `benchmarks/data/hermit/` or read from a local HermiT checkout at test time.
