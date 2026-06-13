# Conformance Coverage

Summary for evaluators comparing OntoLogos to HermiT, ELK, and other reasoners. OntoLogos v0.8.0 ships EL classification, RL/RDFS saturation via reasonable, explanations, and growing HermiT ports.

## HermiT porting strategy

Tests are cataloged in [tests/hermit/manifest.toml](https://github.com/eddiethedean/ontologos/blob/main/tests/hermit/manifest.toml) and implemented in `ontologos-conformance`.

| Tier | CI | HermiT checkout required | Purpose |
|------|-----|--------------------------|---------|
| **A** | Always (`cargo test -p ontologos-conformance`) | No | Inlined RL logic + small fixtures |
| **B** | Always for pizza; wine optional | No (vendored under `benchmarks/data/hermit/`) | `ClassificationTest` taxonomy goldens |

Run locally:

```bash
cargo test -p ontologos-conformance
```

Optional full HermiT tree: set `ONTOLOGOS_HERMIT_ROOT` or clone to `HermiT/` for additional fixtures.

See [tests/hermit/README.md](https://github.com/eddiethedean/ontologos/blob/main/tests/hermit/README.md) for maintainer setup.

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
| `ClassificationTest.testWine` | Optional | Ignored: `wine.xml` fails to parse (duplicate `rdf:ID`) |

## EL golden conformance

Pizza EL taxonomy is checked in CI via [`benchmarks/scripts/compare-pizza-el-golden.sh`](https://github.com/eddiethedean/ontologos/blob/main/benchmarks/scripts/compare-pizza-el-golden.sh) against committed `benchmarks/data/pizza-el-golden.json`. This is a **regression gate** against the in-house EL engine baseline (84 direct subsumptions), not a diff against ELK or whelk. Regenerate baselines when updating completion rules.

The in-house EL engine may omit direct `C ⊑ owl:Thing` edges that other EL tools emit for orphan classes; HermiT vendored Pizza tests do not require those edges.

## Known gaps (v0.5)

| Area | Status |
|------|--------|
| Full OWL DL | Not shipped (2.0 target) |
| Complete OWL RL rule set | Partial — see [RL rules](rl-rules.md) |
| Explanations | Available (EL traces; RL/RDFS asserted-only until reasonable exposes diagnostics) |
| Large DL corpora (GALEN, SNOMED) | Optional stress tests only |
| Wine `ClassificationTest` | Parser limitation on legacy RDF/XML |

## Benchmark corpora

Integration tests use Pizza (downloaded), Family (vendored), go-subset (vendored EL perf), and HermiT pizza fixtures. Manifest expected counts are **mapper output**, not Protégé logical axiom totals.

See [benchmarks.md](../project/benchmarks.md).

## Related

- [Comparison](../comparison.md)
- [Supported constructs](supported-constructs.md)
- [RL rules](rl-rules.md)
