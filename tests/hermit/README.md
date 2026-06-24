# HermiT conformance porting

This directory catalogs tests ported from the [HermiT reasoner](https://github.com/owlcs/hermit-reasoner) (Java) into Rust integration tests under `crates/ontologos-conformance/`.

HermiT source is **not committed** (see root `.gitignore`). Place a checkout at `HermiT/` or set:

```bash
export ONTOLOGOS_HERMIT_ROOT=/path/to/hermit-reasoner
```

## Catalog (599 tests)

Regenerate from vendored fixtures (no HermiT checkout required):

```bash
python3 tests/hermit/generate_catalog.py --activate-all-from-disk
```

Full regen from a HermiT checkout:

```bash
python3 tests/hermit/generate_catalog.py
```

By default **all runnable cases are active** (failure-first). Use `--promoted-only` to gate on `promoted_*_ids.txt` (legacy promotion workflow).

This writes:

- `benchmarks/data/hermit/catalog/cases.json` — metadata per Java test
- `benchmarks/data/hermit/catalog/wg_cases.json` — OWL WG entailment/consistency cases
- `benchmarks/data/hermit/axioms/*.ofn` — extracted functional-syntax fixtures
- `crates/ontologos-conformance/tests/hermit_generated.rs` — one `#[test]` per Java case
- `crates/ontologos-conformance/tests/hermit_wg_generated.rs` — one `#[test]` per WG case

**Runnable today** (no `#[ignore]` on harvested cases):

| Status | Meaning |
|--------|---------|
| `axiom` | OFN fixture + semantic assertions |
| `wg` | OWL WG premise/conclusion RDF |
| `fixture` | EL classification golden (`pizza.xml` only in default CI) |
| `clausify` / `swrl` | Clausification / SWRL ports |
| `ported` | Hand-written in `hermit_rl.rs`, `hermit_rdfs.rs`, or `hermit_el.rs` |

**Still ignored:** `internal`, `excluded`, `migrated`, `ported` stubs, and Java `planned` cases without harvested assertions.

Hand-written ports are listed in [manifest.toml](manifest.toml) and implemented in dedicated test modules.

## Run tests

```bash
# Blocking CI subset (promoted passing cases only)
ONTOLOGOS_CI_PROMOTED_ONLY=1 ONTOLOGOS_DL_BUDGET_SECS=30 \
  cargo test -p ontologos-conformance --release

# Full failure-first suite (local or nightly)
bash benchmarks/scripts/run-hermit-full-suite.sh

# Active vs ignored inventory
./benchmarks/scripts/report-conformance-coverage.sh

# Legacy ignored tier (permanent exclusions, hand-written stubs)
cargo test -p ontologos-conformance -- --ignored
```

**Coverage honesty:** Blocking CI runs `ONTOLOGOS_CI_PROMOTED_ONLY=1` so only cases listed in `promoted_wg_ids.txt` / `promoted_axiom_ids.txt` execute semantic checks. The full suite (`run-hermit-full-suite.sh`) runs every active test and is the source of truth for parity progress.

## Add a hand-written port

1. Find the Java test under `HermiT/src/test/java/org/semanticweb/HermiT/`.
2. Add an entry to [manifest.toml](manifest.toml) and `IMPLEMENTED` in [generate_catalog.py](generate_catalog.py).
3. Implement the Rust test in `crates/ontologos-conformance/tests/hermit_*.rs`.
4. Re-run `generate_catalog.py` so the generated stub is marked `#[ignore]`.
5. Update [hermit-replacement.md](../../docs/internal/research/hermit-replacement.md) if the capability mapping changes.

## License note

HermiT is LGPL-3.0. Ported **test logic** and **small fixture excerpts** are rewritten for OntoLogos; large ontology files are vendored under `benchmarks/data/hermit/` or read from a local HermiT checkout at test time.
