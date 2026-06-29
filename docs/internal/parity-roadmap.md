# HermiT parity roadmap (implementation tiers)

Tracks progress beyond the in-scope catalog gate (`parity_pct = 100%` on 889 cases). See [hermit-parity-honest-assessment.md](hermit-parity-honest-assessment.md).

## Metrics (`parity_status`)

| Field | Meaning |
|-------|---------|
| `parity_pct` | In-scope catalog harness (889 cases) |
| `literal_catalog_pct` | Active CI tests / full catalog (1019) |
| `java_out_of_scope` | excluded + internal + migrated |

```bash
bash benchmarks/scripts/hermit-burndown.sh status
```

## Tier status

| Tier | Goal | Status |
|------|------|--------|
| **A** | v1.0.0 publish + tag | Checklist: [release-1.0-checklist.md](../project/release-1.0-checklist.md) |
| **B** | Literal catalog 1019/1019 | In progress — 130 Java out-of-scope; 143 `#[ignore]` |
| **C** | Strict taxonomy (`--max-extra 0`) | In progress — `Taxonomy::reduce_transitive_redundancy`; `ONTOLOGOS_STRICT_TAXONOMY=1` |
| **D** | Perf + OWL API | In progress — Criterion bench; `is_subsumption_entailed`; `ParseLimits::merge_imports` |

## Internal test ports (Tier B3)

HermiT `internal` cases map to crate unit tests (not conformance axiom ports):

| HermiT suite | OntoLogos tests |
|--------------|-----------------|
| `structural/ClausificationTest` | [crates/ontologos-alc/tests/clausification.rs](../../crates/ontologos-alc/tests/clausification.rs) |
| `structural/NormalizationTest` | [crates/ontologos-alc/tests/normalization.rs](../../crates/ontologos-alc/tests/normalization.rs) |
| Ian/ComplexConcept CE | [crates/ontologos-alc/tests/ian_ce_sat.rs](../../crates/ontologos-alc/tests/ian_ce_sat.rs) |
| `tableau/*` | `ontologos-alc` engine unit tests (partial) |

## Excluded case triage (Tier B1)

Documented in `tests/hermit/generate_catalog.py` `EXCLUDED_IDS`. Re-include only after engine fix + promotion:

- **13** Ian/ComplexConcept CE — tableau soundness
- **4** RIA regularity — full OWL 2 algorithm
- **8** OWLLink — parser / buffered API
- **20** datatype manager — JVM literal validation vs OWL entailment
