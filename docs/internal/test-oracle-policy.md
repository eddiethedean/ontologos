# Test oracle policy

Maintainer guide for writing tests that **independently verify** OntoLogos behavior. Applies after the 2026 test-suite verification pass.

## Ground truth (use these, not implementation mirrors)

| Oracle | Use for |
|--------|---------|
| HermiT / OWL WG catalogs + fixtures under `benchmarks/data/hermit/` | DL entailment, consistency, classification |
| HermiT ClassificationTest `.xml.txt` hierarchies | EL/DL taxonomy shape |
| W3C OWL 2 Profiles / RL / RDFS semantics | Profile detection, RL/RDFS (minus [reasonable-limits](../reference/reasonable-limits.md)) |
| [Rust integration contract](../guides/rust-integration-contract.md) | Public API MUST/MUST-NOT |
| `benchmarks/data/semantic-fixtures.json` | Cross-language binding parity |

## Do not treat as independent truth

- Root `SPEC.md` (historical; prefer architecture + guides)
- OntoLogos-authored JSON goldens (`pizza-el-golden.json`, `dl-taxonomy-golden.json`) — regression only
- Optimistic RL rule tables where ADR documents upstream gaps

## Anti-patterns (reject in review)

1. **Tautologies** — e.g. `count > 0 || is_empty()`
2. **Silent skips** — `if !path.exists() { return; }` in default CI tests
3. **Floor-only asserts** — `>= 0`, `>= 1`, `!is_empty()` without expected IRIs/pairs
4. **Positive entailment shortcuts** without semantic proof — guards must not be looser than full reasoner (see `entailment_guards` meta-test)
5. **Dump-only tests** — `eprintln!` without asserts belong behind `#[ignore]`
6. **Copy-paste shells** — identical test bodies with only IRI changes and weak asserts

## Conformance entailment guards

- Positive guards in `ontologos-conformance/src/catalog/mod.rs` must either call reasoner-backed checks or encode exact semantic conditions.
- `positive_entailment_guards_agree_with_full_reasoner_on_wg_samples` must stay green when adding guards.

## Binding tests

- Share expectations via `benchmarks/data/semantic-fixtures.json`
- Smokes may check build/load; semantic tests must assert concrete subsumption pairs or profile pins

## Catalog honesty

- `catalog_honesty` test tracks `covered` / `excluded` inventory
- Non-generated cases must remain visible in catalog JSON burn-down

## Mutation testing

Run periodically on facade/profile/EL:

```bash
cargo install cargo-mutants
cargo mutants -p ontologos-facade -p ontologos-profile -p ontologos-el
```

Surviving mutants require new behavioral tests, not guard loosening.
