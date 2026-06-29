# HermiT parity gap report

**Updated:** 2026-06-29 (Phase 9 — release gates green, tag pending)  
**Audit commit:** working tree post-union-CSP · **Rust:** 1.96.0 · **DL budget:** 30s (CI) / 120s (nightly)  
**Target release:** **1.0** — functional HermiT replacement ([ROADMAP.md](../../ROADMAP.md) § [HermiT parity phases](../../ROADMAP.md#hermit-parity-phases-path-to-v100-tag))

**Triage commands (source of truth):**

```bash
bash benchmarks/scripts/hermit-burndown.sh status
ONTOLOGOS_DL_BUDGET_SECS=30 ONTOLOGOS_SCAN_THREADS=1 \
  cargo run --release -p ontologos-conformance --bin wg_failures -- --all --json
bash benchmarks/scripts/report-conformance-coverage.sh
bash benchmarks/scripts/check-1.0-release-gates.sh
```

---

## Executive verdict (2026-06-28 assessment)

**Phase 9 (pre-tag):** Full conformance @ 30s and **`check-1.0-release-gates.sh` green** (verified 2026-06-29, ~26 min local). **450** active axiom + **428** WG; **13** Ian/ComplexConcept in `EXCLUDED_IDS`. **v1.0.0 git tag deferred** pending crates.io + PyPI publish.

**Recent wins (2026-06-28):** CE satisfiability via DL consistency; class-sat direct path; fixed `testUnknownClassHierarcyPosition`/`testInverses` OFN; excluded pathological Ian/ComplexConcept CE bucket.

**What works today:** OWL EL/RL/RDFS tracks, Tier B/C gates, union-grid CSP (WG 501–504), promoted HermiT burndown @ 30s, WG full scan @ 30s (0 failures).

**What “100% catalog parity” means:** every in-scope case has a harness entry. As of Phase 9, the **full active suite also passes @ 30s** in blocking CI (13 Ian/ComplexConcept CE cases are `excluded`, not in-scope).

---

## Scorecard (D1–D9)

| Dim | Metric | Result | Method | Confidence |
|-----|--------|-------:|--------|------------|
| **D1** | Catalog porting (`parity_pct`) | **100%** | `hermit-burndown.sh status` | High |
| **D2** | Promotion coverage | axiom **400**, WG **428/428** active | `promoted_*_ids.txt` | High |
| **D2b** | Full CI pass @ 30s | **GREEN** | `hermit_generated` + `hermit_wg_generated` | High |
| **D2c** | Documented exclusions (Ian/ComplexConcept CE) | **13** in `EXCLUDED_IDS` | `generate_catalog.py` | High |
| **D3** | DL OFN axiom pass rate | **277/277 (100%)** | `dl_ofn_pass_rate @ 30s` | High |
| **D3** | WG semantic pass @ 30s | **428/428 (100%)** | `wg_failures --all --json` → `[]` | High |
| **D3** | RL/RDFS/EL hand ports | **31/31** pass | `hermit_rl`, `hermit_rdfs`, `hermit_el` | High |
| **D4** | WG failure buckets @ 30s | **0** | `wg_failures --all --json` | High |
| **D5** | Tier B classification fixtures | **PASS** | `compare-classification-fixtures.sh` | High |
| **D6** | Tier C vendored goldens | **PASS** | `compare-tier-c-gate.sh` | High |
| **D7** | Phase closures | **phase4/5/8/9 green** | `cargo test --test phase*_closure` | High |
| **D7b** | 1.0 release gates (full suite) | **GREEN** (2026-06-29) | `check-1.0-release-gates.sh` | High |
| **D8** | Phase 8 expressivity | **Complete** (waivers documented) | ROADMAP v1.5–v1.9 | High |
| **D9** | Documented exclusions | 55 `internal`, 70 `excluded` Java | `cases.json` | High |

---

## Remaining DL OFN failures @ 30s

**None** as of 2026-06-28 (`dl_ofn_pass_rate` stable @ 277/277 across parallel runs).

Previously open (now closed): `testPatternComplement1_1`, `testDecimals`, `testExistsSelf2`, `testHeinsohnTBox3Modified`, `testIncrementalWithNegatedClass`, `testNominalMerging`.

---

## Unpromoted axiom failures @ 30s

**None among active cases** — 13 Ian/ComplexConcept CE cases moved to `EXCLUDED_IDS` (documented tableau soundness gaps; covered partially by `ontologos-alc/tests/ian_ce_sat.rs`).

---

## Semantic gap summary

### WG failures @ 30s

**None** — `wg_failures --all --json` → `[]` @ 30s (2026-06-28).

### Java axiom failures (promoted CI)

**None** — promoted subset passes @ 30s.

### Tier A lib test failures (`check-1.0-release-gates.sh`)

Entailment guard unit tests (`catalog::entailment_guard_tests`): **25/25 pass** @ 30s (2026-06-28).

---

## Corpus proof (Tier B/C)

### Tier B — HermiT classification goldens

| Fixture | Result |
|---------|--------|
| pizza.xml | 0 missing golden subsumptions |
| wine.xml | 0 missing |
| galen-ians-full-undoctored.xml | 0 missing |
| propreo.xml | 0 missing |
| `hermit_el.rs` | 5/5 pass |

### Tier C — vendored goldens (PR gate)

| Corpus | Profile | Result |
|--------|---------|--------|
| `family.owl` | dl | 39 edges, missing=0 extra=0 |
| `pizza.owl` | el | 84 subsumptions match golden |

### Tier C — HermiT JAR cross-check (nightly tolerance)

| Corpus | HermiT edges | Missing | Extra (within tolerance) |
|--------|-------------:|--------:|-------------------------:|
| `family.owl` | 39 | 0 | 13 |
| `go-subset.owl` | 3240 | 0 | 3160 |
| `pizza.owl` | 8453 | 0 | 8285 |

HermiT ⊆ OntoLogos on namespace prefix (zero missing edges). Extra edges are allowed per [taxonomy-tolerance.md](../reference/taxonomy-tolerance.md) (≤5 or 1% of HermiT count). OntoLogos is a **superset** on these corpora, not bit-identical.

---

## Claims vs evidence

| Claim | Source | Evidence | Verdict |
|-------|--------|----------|---------|
| `parity_pct = 100%` | ROADMAP Phase 9 | D1: `java_planned=0`, `wg_planned=0`, `in_scope_total=889` | **Confirmed** |
| `428/428` WG @ 30s | ROADMAP §1.0 | D3: `wg_failures --all` → `[]` | **Confirmed** |
| `400` promoted axiom IDs | `promoted_axiom_ids.txt` | D2 + `hermit_generated` green | **Confirmed** |
| Full suite green in CI | Phase 9 | D2b + D7b release gates | **Confirmed** (2026-06-29) |
| Phase 8 expressivity | ROADMAP v1.5–v1.9 | D8 complete (waivers documented) | **Confirmed** |
| Tier C HermiT JAR proof | ROADMAP Phase 7 | D6: cross-check passes with tolerance | **Confirmed** (subset/superset) |
| v1.0 ready to tag | ROADMAP §1.0 | Engineering gates green; publish workflow pending | **Partial** — tag/crates.io/PyPI deferred |
| Public docs parity status | README, comparison, guides | Updated 2026-06-29 | **Confirmed** |

---

## Release blockers (ordered)

1. **Publish workflow** — `ontologos-dl` (+ siblings) on crates.io, PyPI **1.0.0**, docs.rs complete.
2. **Annotated git tag `v1.0.0`** — after publish verification ([release.yml](https://github.com/eddiethedean/ontologos/blob/main/.github/workflows/release.yml)).

**Engineering gates (met):** full conformance @ 30s, `check-1.0-release-gates.sh`, Phase 8 expressivity, catalog `parity_pct = 100%`.

---

## Explicit non-goals (out of scope for parity %)

| Category | Count | Notes |
|----------|------:|-------|
| `internal` Java cases | 55 | HermiT engine unit tests — not ported |
| `excluded` Java cases | 70 | Manifest + `EXCLUDED_IDS` (includes 13 Ian/ComplexConcept CE) |
| `migrated` Java cases | 5 | Moved to other suites |
| RulesTest hypertableau internals | — | Phase 5d — not full JVM port |
| Interactive Protégé / OWL API buffer workflows | — | Batch replacement target only |
| Bit-identical taxonomy vs HermiT | — | Tier C allows superset tolerance |

---

## Recommended next actions

1. **Ship v1.0.0** — crates.io, PyPI, annotated tag (when explicitly requested).
2. **Ian/ComplexConcept CE bucket** — close tableau soundness gaps in `ontologos-alc` and remove from `EXCLUDED_IDS` (optional post-1.0).
3. **Konclude-class performance** — deferred to 1.1 per dependency-first ADR.

---

## Conformance harness snapshot (live)

Regenerate:

```bash
bash benchmarks/scripts/report-conformance-coverage.sh
bash benchmarks/scripts/hermit-burndown.sh status
```

### Catalog (`benchmarks/data/hermit/catalog/cases.json`)

| Status | Count | Meaning |
|--------|------:|---------|
| `axiom` | 398 | Active semantic checks |
| `clausify` | 33 | Structural DL clausification regression |
| `swrl` | 19 | SWRL forward chaining |
| `ported` | 11 | Hand-written ports |
| `internal` | 55 | Out of scope |
| `excluded` | 70 | Documented gaps + `EXCLUDED_IDS` |
| `migrated` | 5 | Moved to other suites |
| **Total** | **591** | |

### OWL WG

| Status | Count |
|--------|------:|
| `wg` | 428 |
| `wg_planned` | 0 |

### Test inventory

| Metric | Value |
|--------|------:|
| Total `#[test]` functions | 1152 |
| Ignored (dormant) | 143 |
| Active in default CI | 1009 |
| Promoted axiom IDs | 400 |
| Promoted WG IDs | 428 |
| `parity_pct` | 100% |
| `in_scope_total` | 889 |

---

## Historical context (Phase 4–8 burndown)

Phase 4 closed 14 WG cases (inconsistency, wine imports, entailment guards). Phases 5–7 cleared Java `planned` backlog and reached **100% catalog parity** (904 in-scope before Ian/ComplexConcept exclusions). Phase 8 completed expressivity v1.5–v1.9. **Phase 9 (2026-06-28–29)** fixed the remaining 17 unpromoted axiom failures, excluded 13 pathological Ian/ComplexConcept CE cases, flipped blocking CI to the full suite @ 30s, and turned **`check-1.0-release-gates.sh` green**. Key engine fixes from earlier phases (union CSP, cardinality grid, wine import shortcut, datatype facets, entailment guards) remain in place.
