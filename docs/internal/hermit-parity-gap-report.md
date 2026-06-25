# HermiT parity gap report

**Updated:** 2026-06-24 (Phase 4 in progress)  
**Target release:** **1.0** — functional HermiT replacement ([ROADMAP.md](../../ROADMAP.md) § [HermiT parity phases](../../ROADMAP.md#hermit-parity-phases-path-to-v100-tag))

**Triage commands (source of truth):**

```bash
ONTOLOGOS_DL_BUDGET_SECS=30 cargo run --release -p ontologos-conformance --bin wg_failures -- --json
cargo test -p ontologos-conformance --test phase4_closure --release
bash benchmarks/scripts/report-ci-gate-status.sh
bash benchmarks/scripts/report-conformance-coverage.sh
```

---

## Executive summary (2026-06-24)

| Signal | Value |
|--------|------:|
| **WG failures** (`wg_failures` @ 30s) | **65** / 428 |
| **WG passing** | **363** / 428 |
| **parity_pct** (in-scope catalog) | **~79%** |
| Current ROADMAP phase | **4** (WG fixtures) — in progress |
| Catalog `wg` cases | **428** (`wg_planned = 0`) |
| Promoted WG IDs | **331** (`promoted_wg_ids.txt`) |

### WG failure buckets (2026-06-24, `ONTOLOGOS_DL_BUDGET_SECS=30`)

| Bucket | Count | Notes |
|--------|------:|-------|
| `entailment_positive` | 36 | equivalentClass, intersectionOf/unionOf, QCR, property characteristics |
| `consistency` | 20 | description-logic, Rational-002, misc wine, Thing/Nothing |
| `entailment_negative` | 4 | allValuesFrom-002, I5.8-007, dl-209, misc-302 |
| `timeout` | 3 | Consistent-but-all-unsat, dl-040, object QCR |
| `load_error` | 0 | — |
| `other` | 2 | — |

### Recent fixes (2026-06-24)

- **RDF supplement core merge** (`load.rs`): `merge_supplement_ontology` now remaps and merges **core** axioms — fixes dropped `ObjectPropertyDomain`/`Range` from `rdfs:domain`/`range` supplements
- **`parse_xml_base`**: quote-aware opening-tag scan — fixes `xml:base` when last attribute before `>`
- **`conflicting_instance_typing_non_entailment_guard`**: skip conflict when premise types are intersection/union members or equivalent to conclusion class (bool-intersection/union entailment)
- **Import fixture vendoring** (`generate_catalog.py`): merge companion import ontologies at catalog generation time
- **RDF preprocess**: direct-child guards for typed-node materialization; reified NPA, ill-founded list handling; restriction CE inline mapping
- **Triage tooling**: `scan_all_wg_failures()`, `wg_failures` bin rewrite, `phase4_closure.rs` / `phase4_priority.rs`
- **`wg_phase4_check`**: **9/9** regression tests green

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
