# HermiT parity roadmap (implementation tiers)

Tracks progress beyond the in-scope catalog gate (`parity_pct = 100%` on **915** cases). See [hermit-parity-honest-assessment.md](hermit-parity-honest-assessment.md) and [ROADMAP.md](../../ROADMAP.md#postphase-9--literal-parity-burndown-tiers-bd).

## Metrics (`parity_status`)

| Field | Meaning |
|-------|---------|
| `parity_pct` | In-scope catalog harness (**915** cases; `java_planned + wg_planned = 0`) |
| `literal_catalog_pct` | Active harness tests / full catalog (**1019** entries) — on `hermit-burndown.sh status` |
| `taxonomy_strict_pct` | Tier C corpora passing strict HermiT cross-check (`--max-extra 0`) — from `tier-c-strict-status.json` |
| `perf_gate_pct` | Tier D corpora meeting ROADMAP perf targets — from `dl-perf-snapshot.json` |
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
| **B3** | Port `ClausificationTest` / `NormalizationTest` / `tableau/*` | **Complete (portable scope)** — 34 `migrated`; 7/7 hyper goldens; 23+3 inventory tests ([internal_ports.toml](../../tests/hermit/internal_ports.toml)) |
| **B4** | Burn down **122** `#[ignore]` tests; promote OWLLink cases | Harness metrics live; nightly `--ignored` job; Bob A/B **ported** |
| **C** | Strict taxonomy CI (`ONTOLOGOS_STRICT_TAXONOMY=1`, `--max-extra 0`) | `compare-tier-c-strict-family.sh` (nightly, informational); engine extras remain |
| **D1** | Criterion saturation/tableau benches; Pizza DL **< 30 s** PR gate | `compare-tier-d-perf-gate.sh` (Family **< 1.0 s** PR); `cargo bench -p ontologos-dl` |
| **D2–D4** | Default `owl:imports`; SWRL / `RulesTest` or waiver; `isConsistent` / `isEntailed` / `query` facade | CLI `consistent`/`entail`; facade `is_entailed` + `query_engine`; SWRL deferred |

## Recent progress (2026-06-29)

- **B3 structural hyper clausification** — 7/7 HermiT `ClausificationTest` RDF/XML goldens; `hyper_cardinality`, `hyper_abox`; graph/tableau inventory tests
- **Parity harness metrics** — `taxonomy_strict_pct`, `perf_gate_pct` on `parity_status`
- **Tier D gates** — Family DL perf PR gate; CLI `consistent`/`entail`; facade `is_entailed`/`query_engine`
- **Ian / ComplexConcept CE** — instance-check cluster promoted; `IanBackjumping3` only exclusion
- **Object-property classification** — HermiT-style surrogate taxonomy; `getSubObjectProperties`, equivalent/inverse queries; `RolePropertyQueryContext::prepare()`
- **OWLLink Bob A/B** — catalog-promoted (`testBobTestAandB` → `owllink_bob_knows_subproperties`)
- **B3 internal ports** — 24/24 `NormalizationTest` + 8 internal `ClausificationTest` → `migrated`; structural XML fixtures vendored; `tableau.*` inventory test
- **Bob C** — still blocked (`getObjectPropertyValues` on `agent-inst.owl`)

## Internal test ports (Tier B3)

Manifest: [tests/hermit/internal_ports.toml](../../tests/hermit/internal_ports.toml)

| HermiT suite | OntoLogos tests | Status |
|--------------|-----------------|--------|
| `structural/ClausificationTest` | [clausification.rs](../../crates/ontologos-alc/tests/clausification.rs) | 33 OFN clausify + 7 XML load; hyper goldens `#[ignore]` |
| `structural/ClausificationDatatypesTest` | [clausification.rs](../../crates/ontologos-alc/tests/clausification.rs) | Via `hermit_clausify_catalog` |
| `structural/NormalizationTest` | [normalization.rs](../../crates/ontologos-alc/tests/normalization.rs) | 24/24 smoke clausify (`migrated`) |
| `tableau/*` (23 cases) | [tableau_internals.rs](../../crates/ontologos-alc/tests/tableau_internals.rs) | Inventory only — extension-manager port deferred |
| Ian/ComplexConcept CE | [ian_ce_sat.rs](../../crates/ontologos-alc/tests/ian_ce_sat.rs) · [ian_ce_excluded_triage.rs](../../crates/ontologos-alc/tests/ian_ce_excluded_triage.rs) | Conformance ports |

## Excluded case triage (Tier B1)

Documented in `tests/hermit/generate_catalog.py` `EXCLUDED_IDS`. Re-include only after engine fix + promotion:

- **Ian/ComplexConcept CE** — `IanBackjumping3` + `iant6_unsat_regression` (inverse-universal CE gap)
- **4** RIA regularity — full OWL 2 algorithm
- **OWLLink** — `testBobTestC` (ABox), `testBobTests`, buffered API cases
- **20** datatype manager — JVM literal validation vs OWL entailment
