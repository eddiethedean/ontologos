# Conformance Coverage

Summary for evaluators comparing OntoLogos to HermiT, ELK, and other reasoners.

## User API contract (Tier 0 — PR gate)

[`ontologos-contract`](https://github.com/eddiethedean/ontologos/blob/main/crates/ontologos-contract) checks semantics through **`ontologos_facade`** (classify, consistency, entailment). This is what CLI and Python consumers depend on.

```bash
cargo test -p ontologos-contract --release
```

Sample catalog cases: [`crates/ontologos-contract/data/case_ids.txt`](https://github.com/eddiethedean/ontologos/blob/main/crates/ontologos-contract/data/case_ids.txt).

## HermiT parity harness (nightly / release)

**Contributors:** see the **[HermiT burndown guide](../guides/hermit-burndown.md)** for the daily workflow, parity scoreboard, and what to fix when.

Tests are cataloged in [tests/hermit/manifest.toml](https://github.com/eddiethedean/ontologos/blob/main/tests/hermit/manifest.toml) and implemented in `ontologos-conformance` (internal engine paths — not the public API contract).

| Tier | CI | HermiT checkout required | Purpose |
|------|-----|--------------------------|---------|
| **0** | PR | No | Facade-routed contract (`ontologos-contract`) + pizza/family scripts |
| **A** | Nightly / release | No | Full HermiT + OWL WG catalog (`ontologos-conformance`) |
| **B** | PR | No (vendored under `benchmarks/data/hermit/`) | `ClassificationTest` taxonomy goldens via [`compare-classification-fixtures.sh`](https://github.com/eddiethedean/ontologos/blob/main/benchmarks/scripts/compare-classification-fixtures.sh) |
| **C** | PR (`compare-tier-c-gate.sh`) + nightly HermiT JAR | JVM nightly only | DL taxonomy goldens + HermiT ⊆ OntoLogos cross-check — [taxonomy tolerance](taxonomy-tolerance.md) |

Run parity locally:

```bash
cargo test -p ontologos-conformance --release
```

Optional full HermiT tree: set `ONTOLOGOS_HERMIT_ROOT` or clone to `HermiT/` for additional fixtures.

See [tests/hermit/README.md](https://github.com/eddiethedean/ontologos/blob/main/tests/hermit/README.md) for catalog regeneration.

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

## Known gaps

| Area | Status |
|------|--------|
| Full OWL DL | Shipped on **crates.io/PyPI 1.1.1** (`ontologos-dl`) — see [Install channels](../guides/install-channels.md) |
| Complete OWL RL rule set | Partial — see [RL rules](rl-rules.md) |
| Explanations | Available (EL traces; RL/RDFS asserted-only until reasonable exposes diagnostics) |
| Large DL corpora (GALEN, SNOMED) | Optional stress tests only |
| Wine / galen / propreo `ClassificationTest` | Active via parser preprocess (entities, `rdf:ID`) |
| SWRL `RulesTest` (24 cases) | Active via `ontologos-swrl` forward chaining |
| Tier C DL taxonomy | `family.owl` golden — [taxonomy tolerance](taxonomy-tolerance.md); nightly HermiT JAR cross-check (`tier-c-hermit-crosscheck` in `conformance-nightly.yml`) |

## Benchmark corpora

Integration tests use Pizza (downloaded), Family (vendored), go-subset (vendored EL perf), and HermiT pizza fixtures. Manifest expected counts are **mapper output**, not Protégé logical axiom totals.

See [benchmarks.md](../project/benchmarks.md).

## Related

- [Comparison](../comparison.md)
- [Supported constructs](supported-constructs.md)
- [RL rules](rl-rules.md)
