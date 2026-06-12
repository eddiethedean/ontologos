# Conformance Coverage

Summary for evaluators comparing OntoLogos to HermiT, ELK, and other reasoners. OntoLogos v0.4 is **pre-release** — conformance is growing, not complete.

## HermiT porting strategy

Tests are cataloged in [tests/hermit/manifest.toml](../../tests/hermit/manifest.toml) and implemented in `ontologos-conformance`.

| Tier | CI | HermiT checkout required | Purpose |
|------|-----|--------------------------|---------|
| **A** | Always (`cargo test -p ontologos-conformance`) | No | Inlined logic + small fixtures |
| **B** | `#[ignore]` unless `ONTOLOGOS_HERMIT_ROOT` set | Yes | Parser + classification goldens from HermiT tree |

Run locally:

```bash
cargo test -p ontologos-conformance              # Tier A
cargo test -p ontologos-conformance -- --ignored # Tier B (needs HermiT/)
```

See [tests/hermit/README.md](../../tests/hermit/README.md) for maintainer setup.

## Tier A coverage (v0.4, RL engine)

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

Additional Tier A tests cover RDFS materialization and parser mapping (RDFS rules, profile detection on corpora).

## Known gaps (v0.4)

| Area | Status |
|------|--------|
| OWL EL taxonomy classification | Not shipped (v0.5 target; compare to ELK / whelk-rs) |
| Full OWL DL | Not shipped (2.0 target) |
| Complete OWL RL rule set | Partial — see [RL rules](rl-rules.md) and clash detection limits |
| Explanations | Stub (v0.6) |
| Large DL corpora (GALEN, SNOMED) | Optional stress tests only |
| CLI RL routing | Library/Python only until v0.5 |

## Benchmark corpora

Integration tests use Pizza (downloaded) and Family (vendored) ontologies. Manifest expected counts are **mapper output**, not Protégé logical axiom totals.

See [benchmarks/README.md](../../benchmarks/README.md).

## External comparison harness

Optional script compares RL output against [reasonable](https://github.com/UKEmbassy/reasonable) when installed:

```bash
./benchmarks/scripts/compare-reasonable.sh benchmarks/data/family.owl
```

## v1.0 exit criteria (from ROADMAP)

Conformance targets at stable release:

- EL: parity measured against **ELK + whelk-rs**
- RL: parity measured against **reasonable**
- Documented known divergences from partial OWL mapping

## Related

- [Comparison with existing tools](../comparison.md)
- [Supported constructs](supported-constructs.md)
- [ROADMAP](../../ROADMAP.md)
