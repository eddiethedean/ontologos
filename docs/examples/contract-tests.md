# Contract test example

Run the public API contract harness without a HermiT Java checkout.

## What it tests

[`ontologos-contract`](https://github.com/eddiethedean/ontologos/tree/main/crates/ontologos-contract) validates semantics through **`ontologos_facade`** — the same routing path as CLI and Python.

This is **Tier 0** in CI (PR gate). Full HermiT parity uses `ontologos-conformance` (nightly/release).

## Run

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # some cases need Pizza corpus
cargo test -p ontologos-contract --release
```

## Sample catalog

Case IDs: [`crates/ontologos-contract/data/case_ids.txt`](https://github.com/eddiethedean/ontologos/blob/main/crates/ontologos-contract/data/case_ids.txt)

Each case exercises classify, consistency, or entailment through the facade on a small fixture.

## Evaluator use

1. Run contract tests on your machine after installing from your target channel (0.9.0 or `main`).
2. For HermiT catalog parity, run `cargo test -p ontologos-conformance --release` (longer; see [Conformance](../reference/conformance.md)).
3. Compare taxonomy output using [Taxonomy tolerance](../reference/taxonomy-tolerance.md) when diffing against HermiT.

## Related

- [Conformance reference](../reference/conformance.md)
- [Evaluator scope](../guides/evaluator-scope.md)
- [Examples gallery](index.md)
