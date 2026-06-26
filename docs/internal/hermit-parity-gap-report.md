# HermiT parity gap report

**Updated:** 2026-06-25 (Phase 4 burndown — in progress)  
**Target release:** **1.0** — functional HermiT replacement ([ROADMAP.md](../../ROADMAP.md) § [HermiT parity phases](../../ROADMAP.md#hermit-parity-phases-path-to-v100-tag))

**Triage commands (source of truth):**

```bash
ONTOLOGOS_DL_BUDGET_SECS=30 cargo run --release -p ontologos-conformance --bin wg_failures -- --json
cargo test -p ontologos-conformance --test phase4_closure --release
bash benchmarks/scripts/report-ci-gate-status.sh
bash benchmarks/scripts/report-conformance-coverage.sh
```

---

## Executive summary (2026-06-25)

| Signal | Value |
|--------|------:|
| **WG failures** (`wg_failures` @ 30s, 10 workers) | **~27** / 428 (targeted triage) |
| **WG passing** (estimated) | **~401** / 428 |
| Promoted WG IDs | **407** (`promoted_wg_ids.txt`) |

### WG failure buckets (2026-06-25, `ONTOLOGOS_DL_BUDGET_SECS=30`, 10 parallel workers)

| Bucket | Count | Notes |
|--------|------:|-------|
| `consistency` | 20 | description-logic, Rational-002, misc wine, Thing/Nothing |
| `entailment_positive` | 9 | QCR, DisjointUnion, SelfRestriction, I4/I5, Restriction-006 |
| `timeout` | 3 | Consistent-but-all-unsat, dl-504, misc-002 |
| `entailment_negative` | 0 | closed |
| `load_error` | 0 | — |
| `other` | 0 | closed |

### Recent fixes (2026-06-25 burndown)

- **Tableau WS0**: iterative `materialize_existential_chain`; `member_block_to_ofn` recursion fix (Restriction-006)
- **Parallel triage**: `ONTOLOGOS_SCAN_THREADS=10`, `ONTOLOGOS_DL_MAX_WORKERS=10` defaults in `wg_failures` / `promote_wg`
- **Spurious consistency**: `flower_auxiliary_unsatisfiable_classes` fallback removed; `class_assertion_only_consistency` for class-assertion-only ABoxes; `named_class_skip_atomic_unsat_precheck` for complex equivalents
- **Missed consistency**: `disjointWith-010` parser (anonymous `owl:Thing` OPAs); `abox_asserted_exact_zero_equiv_class` (dl-601); `union_csp` oneof grid (dl-502); `abox_exists_forall_role_clash` ordering fix
- **Tableau cardinality**: `And` conjuncts assert cardinality before `∃`; `ce_has_unqualified_cardinality_bound` skip in nested ABox materialize; `world_satisfies_filler` / `materialize_filler_on_world` for max-card reuse
- **Entailment**: `singleton_range_functional_entailment_guard` (FunctionalProperty-004); `entailment_via_subclass_nothing` structural/classify fast paths
- **`wg_phase4_check`**: **32/32** green at 30s DL budget

### Recent fixes (2026-06-24)

- **RDF supplement core merge** (`load.rs`): `merge_supplement_ontology` now remaps and merges **core** axioms — fixes dropped `ObjectPropertyDomain`/`Range` from `rdfs:domain`/`range` supplements
- **`parse_xml_base`**: quote-aware opening-tag scan — fixes `xml:base` when last attribute before `>`
- **`conflicting_instance_typing_non_entailment_guard`**: skip conflict when premise types are intersection/union members or equivalent to conclusion class (bool-intersection/union entailment)
- **Import fixture vendoring** (`generate_catalog.py`): merge companion import ontologies at catalog generation time
- **RDF preprocess**: direct-child guards for typed-node materialization; reified NPA, ill-founded list handling; restriction CE inline mapping
- **Triage tooling**: `scan_all_wg_failures()`, `wg_failures` bin rewrite, `phase4_closure.rs` / `phase4_priority.rs`
- **`wg_phase4_check`**: **32/32** regression tests green

### Concrete blockers

1. **equivalentClass / intersectionOf / unionOf / oneOf** (~15): TBox + ABox entailment guards or DL classify gaps
2. **description-logic consistency** (~15): genuine tableau/saturation gaps (nominals, QCR, complements)
3. **QCR / DisjointUnion / SelfRestriction** (New-Feature-*): cardinality and advanced CE reasoning
4. **Timeouts (×3)**: `Consistent-but-all-unsat`, dl-040, object QCR — optimize or early guards without raising 30s budget
5. **Entailment negatives (×4)**: `allValuesFrom-002` regression from core supplement merge; Keys/I5.8 patterns
6. **miscellaneous wine (×2)**: consistency expected true — parser or engine gap

**Phase 4 closure:** `phase4_closure` **fails** (`phase4_all_wg_failures_empty`, `phase4_promoted_wg_complete`) until `wg_failures` → 0 and `promote_wg` refreshes **428** promoted IDs.

---

## Conformance harness snapshot

Regenerate live counts:

```bash
bash benchmarks/scripts/report-conformance-coverage.sh
```

### Catalog (`benchmarks/data/hermit/catalog/cases.json`)

| Status | Count | Meaning |
|--------|------:|---------|
| `axiom` | 270 | Active semantic checks |
| `planned` | 200 | Manual port backlog (Phase 5) |
| `clausify` | 33 | Structural DL clausification regression |
| `swrl` | 19 | SWRL forward chaining |
| `internal` | 55 | Engine-internal (ignored) |
| **Total** | **591** | |

### OWL WG

| Status | Count |
|--------|------:|
| `wg` | 428 |
| `wg_planned` | 0 |

All **428** WG cases are active in `hermit_wg_generated.rs` (failure-first workflow).
