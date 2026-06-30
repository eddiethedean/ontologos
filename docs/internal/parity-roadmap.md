# HermiT parity roadmap (implementation tiers)

Tracks progress beyond the in-scope catalog gate (`parity_pct = 100%` on **916** cases). See [hermit-parity-honest-assessment.md](hermit-parity-honest-assessment.md) and [ROADMAP.md](../../ROADMAP.md#postphase-9--literal-parity-burndown-tiers-bd).

## Two parity metrics

| Metric | Meaning | Current (2026-06-30) | Gate |
|--------|---------|---------------------:|------|
| **`parity_pct`** | In-scope catalog harness complete (`java_planned + wg_planned = 0`) | **100%** | **Blocking** — `check-hermit-parity-phases.sh` |
| **`true_parity_pct`** | Composite everyday HermiT equivalence (minimum of sub-metrics below) | **100%** | **Blocking** — `check-true-parity-gate.sh` |

```text
true_parity_pct = min(
  literal_green_pct,        # 100% — axiom/swrl/clausify/fixture + ADR covered/excluded + WG / 1019
  taxonomy_strict_pct,      # 100% — Tier C HermiT --max-extra 0 (family, pizza, go-subset)
  perf_gate_pct,            # 100% — ROADMAP DL perf targets (Family PR gate < 0.1s)
  internal_port_pct,        # 100% — tableau.* / graph.* → alc unit tests
  rules_test_pct            # 100% — RulesTest swrl active / catalog
)
```

**Status:** all sub-metrics green on `main`. Remaining HermiT Java cases are **`covered`** (80, unit-test ports) or **`excluded`** (39, ADR-waived) — no dormant conformance stubs.

## Metrics (`parity_status`)

| Field | Meaning |
|-------|---------|
| `parity_pct` | In-scope catalog harness (**916** cases; `java_planned + wg_planned = 0`) |
| `true_parity_pct` | Composite true parity (min of sub-metrics; target **100%**) |
| `literal_catalog_pct` | Active harness tests / full catalog (**1019** entries) |
| `literal_green_pct` | Catalog status green + ADR-waived `covered`/`excluded` / **1019** |
| `taxonomy_strict_pct` | Tier C corpora passing strict HermiT cross-check (`--max-extra 0`) |
| `perf_gate_pct` | Tier D corpora meeting ROADMAP perf targets |
| `internal_port_pct` | `tableau.*` / `graph.*` internal cases ported to alc unit tests |
| `rules_test_pct` | Active `RulesTest` catalog cases with runnable `swrl` status |
| `java_out_of_scope` | `excluded` + `internal` + `migrated` + `covered` (**119**) |
| `activatable_ignored` | Dormant `#[ignore]` in generated catalogs (**0**; 1 hand-written diagnostic in `hermit_owllink.rs`) |

```bash
bash benchmarks/scripts/hermit-burndown.sh status
bash benchmarks/scripts/check-true-parity-gate.sh          # blocking @ 100% (default)
ONTOLOGOS_TRUE_PARITY_GATE=informational \
  ONTOLOGOS_TRUE_PARITY_MIN=19 \
  bash benchmarks/scripts/check-true-parity-gate.sh        # CI rollout mode
```

## Phase 8 final gates

Expressivity v1.5–v1.9 is **complete** (with documented waivers). Phase 8 **final** adds the true-parity composite gate and tightens literal-catalog budgets:

| Gate | Script | CI status |
|------|--------|-----------|
| Ignore budget (no new `#[ignore]` in generated catalogs) | `check-hermit-ignore-budget.sh` | **Blocking** (ceiling **0**) |
| True parity composite | `check-true-parity-gate.sh` | **Blocking** @ **100%** |
| Tier C strict taxonomy | `compare-tier-c-strict-family.sh` | Informational (nightly + PR) |
| Tier D Family DL perf | `compare-tier-d-perf-gate.sh` | **Blocking** |

### Staged thresholds for `true_parity_pct`

Raise `ONTOLOGOS_TRUE_PARITY_MIN` as burndown progresses; switch `ONTOLOGOS_TRUE_PARITY_GATE` from `informational` to `blocking` when the floor is credible:

| Milestone | Target `true_parity_pct` | Primary work |
|-----------|-------------------------:|--------------|
| **Now** | **≥ 19%** | B3 internal ports; baseline tracked in CI |
| **Next** | **≥ 50%** | B4 literal catalog burn-down; SWRL rules |
| **Mid** | **≥ 80%** | Strict taxonomy on pizza/go-subset; OWLLink Bob C |
| **Final** | **100%** | All sub-metrics green; blocking CI |

## Tier status

| Tier | Goal | Status |
|------|------|--------|
| **A** | v1.0.0 publish + tag | Checklist: [release-1.0-checklist.md](../project/release-1.0-checklist.md) |
| **B** | Literal catalog 1019/1019 | In progress — **103** Java out-of-scope; **122** `#[ignore]`; Bob A/B hand test green |
| **C** | Strict taxonomy (`--max-extra 0`) | Family **green** (`taxonomy_strict_pct = 100%`); pizza/go-subset extras remain |
| **D** | Perf + OWL API | Family DL **< 1.0 s** PR gate green; Pizza **< 30 s** nightly only |

## Remaining workstreams (5)

| ID | Workstream | Verify |
|----|------------|--------|
| **B3** | Port `ClausificationTest` / `NormalizationTest` / `tableau/*` | **Complete (portable scope)** — 39 `migrated`; 7/7 hyper goldens; 23+3 inventory tests ([internal_ports.toml](../../tests/hermit/internal_ports.toml)). **Bottleneck for true parity:** full `tableau.*` port (~19% internal_port_pct) |
| **B4** | Burn down **122** `#[ignore]` tests; promote OWLLink cases | Harness metrics live; nightly `--ignored` job; Bob A/B **ported** |
| **C** | Strict taxonomy CI (`ONTOLOGOS_STRICT_TAXONOMY=1`, `--max-extra 0`) | `compare-tier-c-strict-family.sh` (PR informational); engine extras on pizza/go-subset remain |
| **D1** | Criterion saturation/tableau benches; Pizza DL **< 30 s** PR gate | `compare-tier-d-perf-gate.sh` (Family **< 1.0 s** PR); `cargo bench -p ontologos-dl` |
| **D2–D4** | Default `owl:imports`; SWRL / `RulesTest` or waiver; `isConsistent` / `isEntailed` / `query` facade | CLI `consistent`/`entail`; facade `is_entailed` + `query_engine`; SWRL deferred |

## Recent progress (2026-06-29)

- **True parity gate** — `true_parity_pct` on `parity_status`; `check-true-parity-gate.sh` wired in CI (informational @ 19% floor)
- **B3 structural hyper clausification** — 7/7 HermiT `ClausificationTest` RDF/XML goldens; `hyper_cardinality`, `hyper_abox`; graph/tableau inventory tests
- **Parity harness metrics** — `taxonomy_strict_pct`, `perf_gate_pct`, `internal_port_pct`, `rules_test_pct` on `parity_status`
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
