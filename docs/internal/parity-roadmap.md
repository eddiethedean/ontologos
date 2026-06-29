# HermiT parity roadmap (implementation tiers)

Tracks progress beyond the in-scope catalog gate (`parity_pct = 100%` on **915** cases). See [hermit-parity-honest-assessment.md](hermit-parity-honest-assessment.md) and [ROADMAP.md](../../ROADMAP.md#postphase-9--literal-parity-burndown-tiers-bd).

## Metrics (`parity_status`)

| Field | Meaning |
|-------|---------|
| `parity_pct` | In-scope catalog harness (**915** cases; `java_planned + wg_planned = 0`) |
| `literal_catalog_pct` | Active harness tests / full catalog (**1019** entries) — on `hermit-burndown.sh status` |
| `java_out_of_scope` | `excluded` + `internal` + `migrated` (**104** as of 2026-06-29) |

```bash
bash benchmarks/scripts/hermit-burndown.sh status
```

## Tier status

| Tier | Goal | Status |
|------|------|--------|
| **A** | v1.0.0 publish + tag | Checklist: [release-1.0-checklist.md](../project/release-1.0-checklist.md) |
| **B** | Literal catalog 1019/1019 | In progress — **104** Java out-of-scope; **122** `#[ignore]`; Bob A/B hand test green |
| **C** | Strict taxonomy (`--max-extra 0`) | In progress — `Taxonomy::reduce_transitive_redundancy` landed; CI gate not blocking |
| **D** | Perf + OWL API | In progress — Criterion bench scaffold; `is_subsumption_entailed`; `ParseLimits::merge_imports` |

## Remaining workstreams (5)

| ID | Workstream | Verify |
|----|------------|--------|
| **B3** | Port `ClausificationTest` / `NormalizationTest` / remaining `tableau/*` to `ontologos-alc` unit tests; document in manifest | `cargo test -p ontologos-alc` · [manifest.toml](../../tests/hermit/manifest.toml) |
| **B4** | Burn down **122** `#[ignore]` tests; promote OWLLink cases | Bob A/B **ported**; `literal_catalog_pct` live |
| **C** | Strict taxonomy CI (`ONTOLOGOS_STRICT_TAXONOMY=1`, `--max-extra 0`) | `compare-tier-c-gate.sh` |
| **D1** | Criterion saturation/tableau benches; Pizza DL **< 30 s** PR gate | `cargo bench -p ontologos-dl` |
| **D2–D4** | Default `owl:imports`; SWRL / `RulesTest` or waiver; `isConsistent` / `isEntailed` / `query` facade | [hermit-replacement.md](research/hermit-replacement.md) |

## Recent progress (2026-06-29)

- **Ian / ComplexConcept CE** — instance-check cluster promoted; `IanBackjumping3` only exclusion
- **Object-property classification** — HermiT-style surrogate taxonomy; `getSubObjectProperties`, equivalent/inverse queries; `RolePropertyQueryContext::prepare()`
- **OWLLink Bob A/B** — catalog-promoted (`testBobTestAandB` → `owllink_bob_knows_subproperties`)
- **Bob C** — still blocked (`getObjectPropertyValues` on `agent-inst.owl`)

## Internal test ports (Tier B3)

HermiT `internal` cases map to crate unit tests (not conformance axiom ports):

| HermiT suite | OntoLogos tests |
|--------------|-----------------|
| `structural/ClausificationTest` | [crates/ontologos-alc/tests/clausification.rs](../../crates/ontologos-alc/tests/clausification.rs) |
| `structural/NormalizationTest` | [crates/ontologos-alc/tests/normalization.rs](../../crates/ontologos-alc/tests/normalization.rs) |
| Ian/ComplexConcept CE | [crates/ontologos-alc/tests/ian_ce_sat.rs](../../crates/ontologos-alc/tests/ian_ce_sat.rs) · [ian_ce_excluded_triage.rs](../../crates/ontologos-alc/tests/ian_ce_excluded_triage.rs) |
| `tableau/*` | `ontologos-alc` engine unit tests (partial) |

## Excluded case triage (Tier B1)

Documented in `tests/hermit/generate_catalog.py` `EXCLUDED_IDS`. Re-include only after engine fix + promotion:

- **Ian/ComplexConcept CE** — `IanBackjumping3` + `iant6_unsat_regression` (inverse-universal CE gap)
- **4** RIA regularity — full OWL 2 algorithm
- **OWLLink** — `testBobTestC` (ABox), `testBobTests`, buffered API cases
- **20** datatype manager — JVM literal validation vs OWL entailment
