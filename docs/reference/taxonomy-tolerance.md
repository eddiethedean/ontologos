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

**HermiT JAR cross-check:** set `HERMIT_JAR` and run `benchmarks/scripts/compare-dl-hermit-crosscheck.sh` (also invoked from `compare-hermit-tier-c.sh` when the JAR is present). Uses external tolerance (≤5 extra edges or 1%).

**Provenance:** committed goldens are generated with `UPDATE_GOLDEN=1` on the named script after a reviewed OntoLogos release build. External baselines (Konclude/HermiT) are optional cross-checks when `KONCLUDE_BIN` or `HERMIT_JAR` are set locally or in nightly jobs.

## Exact match (default CI)

For vendored OntoLogos goldens, `benchmarks/scripts/compare-taxonomy.py` requires:

- **Zero** missing subsumption edges (after filtering)
- **Zero** extra subsumption edges (after filtering)
- Direct `A ⊑ owl:Thing` edges are **ignored** on both sides (common DL artifact)

## External reference tolerance (optional)

When comparing against Konclude or HermiT output (manual/nightly, not default PR CI):

| Rule | Allowed |
|------|---------|
| Missing direct `⊑ owl:Thing` | Ignored |
| Missing redundant transitive edges | Up to **0** (report only) |
| Extra inferred edges | Up to **5** per corpus or **1%** of golden size, whichever is larger |
| Equivalence vs mutual subsumption | Normalize to mutual `⊑` before compare |
| Timeout / OOM on reference tool | Skip with logged warning; does not fail PR CI |

Use `--max-missing` and `--max-extra` on `compare-taxonomy.py` for external runs.

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
- `benchmarks/scripts/compare-hermit-tier-c.sh` — Tier C orchestration
