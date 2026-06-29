# HermiT conformance porting

> **New to HermiT parity work?** Read the **[HermiT burndown guide](../../docs/guides/hermit-burndown.md)** first — it explains what to do, why, and the daily loop. This file covers catalog mechanics.

This directory catalogs tests ported from the [HermiT reasoner](https://github.com/owlcs/hermit-reasoner) (Java) into Rust integration tests under `crates/ontologos-conformance/`.

## Quick start (contributors)

```bash
./benchmarks/scripts/download.sh
bash benchmarks/scripts/hermit-burndown.sh status    # parity dashboard
bash benchmarks/scripts/hermit-burndown.sh loop      # daily fix-verify loop
```

HermiT source is **not committed** (see root `.gitignore`). Optional for full catalog regen:

```bash
export ONTOLOGOS_HERMIT_ROOT=/path/to/hermit-reasoner
# or clone to HermiT/
```

## Catalog regeneration

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

Engine-internal HermiT suites (`structural.*`, `tableau.*`) map to `ontologos-alc` unit tests via [internal_ports.toml](internal_ports.toml) (Tier B3).

## Tests and CI

See [HermiT burndown guide](../../docs/guides/hermit-burndown.md) for the full workflow. Short version:

```bash
bash benchmarks/scripts/hermit-burndown.sh triage    # what to fix next
bash benchmarks/scripts/hermit-burndown.sh promote   # update CI promoted lists
bash benchmarks/scripts/hermit-burndown.sh test      # blocking CI subset
bash benchmarks/scripts/hermit-burndown.sh test-full # failure-first truth
```

**Coverage:** Blocking CI runs the **full** active HermiT + OWL WG catalog @ 30s (`ONTOLOGOS_DL_BUDGET_SECS=30`; no `ONTOLOGOS_CI_PROMOTED_ONLY`). `promoted_*_ids.txt` feeds `phase9_closure` hygiene — run `hermit-burndown.sh promote` after fixes so promoted lists stay aligned with passing cases.

## Add a hand-written port

1. Find the Java test under `HermiT/src/test/java/org/semanticweb/HermiT/`.
2. Add an entry to [manifest.toml](manifest.toml) and `IMPLEMENTED` in [generate_catalog.py](generate_catalog.py).
3. Implement the Rust test in `crates/ontologos-conformance/tests/hermit_*.rs`.
4. Re-run `generate_catalog.py` so the generated stub is marked `#[ignore]`.
5. Update [hermit-replacement.md](../../docs/internal/research/hermit-replacement.md) if the capability mapping changes.

## License note

HermiT is LGPL-3.0. Ported **test logic** and **small fixture excerpts** are rewritten for OntoLogos; large ontology files are vendored under `benchmarks/data/hermit/` or read from a local HermiT checkout at test time.
