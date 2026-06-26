# Conformance Coverage

Summary for evaluators comparing OntoLogos to HermiT, ELK, and other reasoners. OntoLogos v0.9.0 ships EL classification, RL/RDFS saturation via reasonable, explanations, incremental reasoning, and growing HermiT ports.

## HermiT porting strategy

**Contributors:** see the **[HermiT burndown guide](../guides/hermit-burndown.md)** for the daily workflow, parity scoreboard, and what to fix when.

Tests are cataloged in [tests/hermit/manifest.toml](https://github.com/eddiethedean/ontologos/blob/main/tests/hermit/manifest.toml) and implemented in `ontologos-conformance`.

| Tier | CI | HermiT checkout required | Purpose |
|------|-----|--------------------------|---------|
| **A** | Always (`cargo test -p ontologos-conformance`) | No | Inlined RL logic + small fixtures |
| **B** | Always | No (vendored under `benchmarks/data/hermit/`) | `ClassificationTest` taxonomy goldens via [`compare-classification-fixtures.sh`](https://github.com/eddiethedean/ontologos/blob/main/benchmarks/scripts/compare-classification-fixtures.sh) |

Run locally:

```bash
cargo test -p ontologos-conformance
```

Optional full HermiT tree: set `ONTOLOGOS_HERMIT_ROOT` or clone to `HermiT/` for additional fixtures.

See [tests/hermit/README.md](https://github.com/eddiethedean/ontologos/blob/main/tests/hermit/README.md) for catalog regeneration. **New contributors:** start with the [HermiT burndown guide](../guides/hermit-burndown.md).

## Tier A coverage (RL engine)

Representative ported cases (see manifest for full list):

| HermiT test | Engine | Capability |
|-------------|--------|------------|
| `testSubsumption2` | RL | Existential subsumption via property hierarchy |
| `testSubsumption3` | RL | Equivalent properties + existentials |
| `testSameAs` | RL | `sameAs` propagates class assertions |
| `testEquivalentClassInstances` | RL | Equivalent classes share instance types |
| `testReflexiveAndSameAs` | RL | Reflexive properties + `sameAs` |
| `testIndividualRetrievalBug` | RL | Property assertion indexing |
| `testIsFunctionalObject` | RL | Functional characteristic on sub-properties |

## Tier B coverage (EL engine)

| HermiT test | Status | Notes |
|-------------|--------|-------|
| `ClassificationTest.testPizza` | **CI** | Vendored `pizza.xml` + golden `.txt` |
| `ClassificationTest.testWine` | **CI** | Hand-written `hermit_el`; `wine.xml` loads after `rdf:ID` dedup |
| `ClassificationTest.testGalenIansFullUndoctored` | **CI** | Entity expansion + numeric `rdf:ID` normalization |
| `ClassificationTest.testPropreo` | **CI** | Entity expansion (single-quoted `DOCTYPE`) |
| `ClassificationIndividualReuseTest.testDolce` | Excluded | `dolce_all.xml` not vendored |

## EL golden conformance

Pizza EL taxonomy is checked in CI via [`benchmarks/scripts/compare-pizza-el-golden.sh`](https://github.com/eddiethedean/ontologos/blob/main/benchmarks/scripts/compare-pizza-el-golden.sh) against committed `benchmarks/data/pizza-el-golden.json`. This is a **regression gate** against the in-house EL engine baseline (84 direct subsumptions), not a diff against ELK or whelk. HermiT `ClassificationTest` XML fixtures (pizza, wine, galen, propreo) are checked via [`compare-classification-fixtures.sh`](https://github.com/eddiethedean/ontologos/blob/main/benchmarks/scripts/compare-classification-fixtures.sh).

The in-house EL engine may omit direct `C ⊑ owl:Thing` edges that other EL tools emit for orphan classes; HermiT vendored Pizza tests do not require those edges.

## Known gaps (v0.5)

| Area | Status |
|------|--------|
| Full OWL DL | Not shipped (2.0 target) |
| Complete OWL RL rule set | Partial — see [RL rules](rl-rules.md) |
| Explanations | Available (EL traces; RL/RDFS asserted-only until reasonable exposes diagnostics) |
| Large DL corpora (GALEN, SNOMED) | Optional stress tests only |
| Wine / galen / propreo `ClassificationTest` | Active via parser preprocess (entities, `rdf:ID`) |
| SWRL `RulesTest` (24 cases) | Active via `ontologos-swrl` forward chaining |
| Tier C DL taxonomy | `family.owl` golden — [taxonomy tolerance](taxonomy-tolerance.md) |

## Benchmark corpora

Integration tests use Pizza (downloaded), Family (vendored), go-subset (vendored EL perf), and HermiT pizza fixtures. Manifest expected counts are **mapper output**, not Protégé logical axiom totals.

See [benchmarks.md](../project/benchmarks.md).

## Related

- [Comparison](../comparison.md)
- [Supported constructs](supported-constructs.md)
- [RL rules](rl-rules.md)
