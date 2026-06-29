# Taxonomy comparison tolerance (Tier C)

OntoLogos **Tier C** gates compare classification taxonomies against vendored golden baselines and, optionally, external reference reasoners (HermiT JAR, Konclude CLI). This document defines what differences are acceptable.

## Baselines

| Corpus | Profile | Golden file | CI gate |
|--------|---------|-------------|---------|
| `pizza.owl` | `el` | `benchmarks/data/pizza-el-golden.json` | Tier B (`compare-pizza-el-golden.sh`) |
| `family.owl` | `dl` | `benchmarks/data/dl-taxonomy-golden.json` | Tier C (`compare-dl-taxonomy.sh`) — default CI |
| `pizza.owl` | `dl` | `benchmarks/data/dl-taxonomy-golden.json` | Tier C **optional** (`RUN_SLOW_DL_GATES=1`) |
| `go-subset.owl` | `dl` | `benchmarks/data/dl-taxonomy-golden.json` | Tier C **optional** — OBO biomedical subset |
| `galen-ians-full-undoctored.xml` | `el` | HermiT fixture golden | Tier B (conformance `fixture`) |
| `go-subset.owl` | `el` / hybrid | Load + hybrid smoke only | Tier C smoke (profile detect) |

**HermiT JAR cross-check:** download with `benchmarks/scripts/download-hermit-jar.sh` (writes `benchmarks/data/hermit.jar`, gitignored). Set `HERMIT_JAR` and run `benchmarks/scripts/compare-dl-hermit-crosscheck.sh` (also invoked from `compare-hermit-tier-c.sh` when the JAR is present). Nightly CI sets `ONTOLOGOS_REQUIRE_HERMIT_JAR=1` and fails if the JAR or `java` is missing. Uses external tolerance (≤5 extra edges or 1% of HermiT edge count, whichever is larger).

**Provenance:** committed goldens are generated with `UPDATE_GOLDEN=1` on the named script after a reviewed OntoLogos release build. External baselines (Konclude/HermiT) are optional cross-checks when `KONCLUDE_BIN` or `HERMIT_JAR` are set locally or in nightly jobs.

## Exact match (default CI)

For vendored OntoLogos goldens, `benchmarks/scripts/compare-taxonomy.py` requires:

- **Zero** missing subsumption edges (after filtering)
- **Zero** extra subsumption edges (after filtering)
- Direct `A ⊑ owl:Thing` edges are **ignored** on both sides (common DL artifact)

## External reference tolerance (optional)

When comparing against Konclude or HermiT output (manual/nightly, not default PR CI):

Set `ONTOLOGOS_STRICT_TAXONOMY=1` when running `compare-dl-hermit-crosscheck.sh` to require **zero** extra edges (Tier C strict mode). Default uses superset tolerance below.

| Rule | Allowed |
|------|---------|
| Missing direct `⊑ owl:Thing` | Ignored |
| Missing redundant transitive edges | Up to **0** (report only) |
| Extra inferred edges | Up to **5** per corpus or **1%** of HermiT edge count, whichever is larger; additionally up to the **namespace-filtered edge-count gap** when OntoLogos is a strict superset |
| Equivalence vs mutual subsumption | Normalize to mutual `⊑` before compare |
| Timeout / OOM on reference tool | Skip with logged warning; does not fail PR CI |

Use `--max-missing` and `--max-extra` on `compare-taxonomy.py` for external runs.

## Timeout policy (DL classification)

Wall times are measured with `benchmarks/scripts/benchmark-dl-perf.sh` (release CLI). Re-run locally after engine changes; nightly runs with `RUN_SLOW_DL_GATES=1` (informational).

| Corpus | PR CI | Nightly | ROADMAP target | Baseline (release, local) |
|--------|-------|---------|----------------|---------------------------|
| Family DL | Gate (`compare-tier-c-gate.sh`) | Gate + HermiT cross-check | &lt;100 ms | ~0.5 s (release) |
| Pizza DL | Skip | Gate + perf snapshot | &lt;30 s medium-DL | ~5 min (gap documented) |
| go-subset DL | Skip | Gate + perf snapshot | &lt;10 s (best-effort DL) | ~2 min |

Pizza DL remains far from the 30 s medium-DL target; Phase 7 documents the gap rather than blocking exit. Optional slow gates use `RUN_SLOW_DL_GATES=1`.

## Large corpora

| Corpus | PR CI | Notes |
|--------|-------|-------|
| Pizza EL | Yes | ~84 subsumptions |
| Family DL | Yes | ~59 subsumptions, ~20s release classify |
| Pizza DL | Optional (`RUN_SLOW_DL_GATES=1`) | ~4000 subsumptions, ~5min release classify |
| go-subset DL | Optional (`RUN_SLOW_DL_GATES=1`) | OBO biomedical subset, ~2min release classify |
| Full GALEN / SNOMED | Optional `#[ignore]` | Parser stress tests; manual download |

## Regenerating goldens

```bash
UPDATE_GOLDEN=1 ./benchmarks/scripts/compare-pizza-el-golden.sh
UPDATE_GOLDEN=1 ./benchmarks/scripts/compare-dl-taxonomy.sh
```

Review diffs in PR; goldens are regression locks, not silent updates.

## Related

- [Conformance](conformance.md)
- [Benchmarks](../project/benchmarks.md)
- `benchmarks/scripts/compare-tier-c-gate.sh` — PR-blocking Tier C vendored goldens
- `benchmarks/scripts/compare-hermit-tier-c.sh` — Tier C orchestration
- `benchmarks/scripts/download-hermit-jar.sh` — standalone HermiT CLI JAR for nightly cross-check
- `benchmarks/scripts/benchmark-dl-perf.sh` — DL wall-time snapshot
