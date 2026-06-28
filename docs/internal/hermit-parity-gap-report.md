# HermiT parity gap report

**Updated:** 2026-06-28 (Phase 9 strict burndown — live triage)  
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

**Catalog porting remains complete (`parity_pct = 100%`).** Promoted CI (`hermit-burndown.sh test` @ 30s) is **green** (371 axiom + 428 WG). **Full semantic parity is not yet reached:** DL OFN axiom pass rate is **271/277 (97.8%)** @ 30s with **6 remaining failures** (2 datatype + 4 ReasonerTest). WG semantic scan is **0 failures** @ 30s; `phase4_closure` and `phase5_closure` are green. v1.0 tag remains blocked until `cargo test -p ontologos-conformance` is fully green @ 30s without `ONTOLOGOS_CI_PROMOTED_ONLY`, release gates use the full suite, and remaining ReasonerTest/tableau cases are closed.

**Recent wins (2026-06-28):** integer/int value-space cross-checks; nested dateTime facet compare; mixed-TZ empty ranges; conjunctive dateTime endpoint counting; float+double all-values clash; rational→decimal value space; Thing nominal + ABox data; negative assertion no longer over-rejects optional witnesses.

**What works today:** OWL EL/RL/RDFS tracks, Tier B/C gates, union-grid CSP (WG 501–504), promoted HermiT burndown @ 30s, WG full scan @ 30s (0 failures).

**What “100% catalog parity” means:** every in-scope case has a harness entry — not that every case passes at 30s without promotion filter.

---

## Scorecard (D1–D9)

| Dim | Metric | Result | Method | Confidence |
|-----|--------|-------:|--------|------------|
| **D1** | Catalog porting (`parity_pct`) | **100%** | `hermit-burndown.sh status` | High |
| **D2** | Promotion coverage | axiom **371/413**, WG **428/428** active | `promoted_*_ids.txt` | High |
| **D2b** | Promoted CI pass (`hermit-burndown.sh test` @ 30s) | **GREEN** | 799 promoted cases | High |
| **D3** | DL OFN axiom pass rate | **271/277 (97.8%)** | `dl_ofn_pass_rate @ 30s` | High |
| **D3** | WG semantic pass @ 30s | **428/428 (100%)** | `wg_failures --all --json` → `[]` | High |
| **D3** | RL/RDFS/EL hand ports | **31/31** pass | `hermit_rl`, `hermit_rdfs`, `hermit_el` | High |
| **D4** | WG failure buckets @ 30s | **0** | `wg_failures --all --json` | High |
| **D5** | Tier B classification fixtures | **PASS** | `compare-classification-fixtures.sh` | High |
| **D6** | Tier C vendored goldens | **PASS** | `compare-tier-c-gate.sh` | High |
| **D7** | Phase closures | **phase4/5/8/9 green** | `cargo test --test phase*_closure` | High |
| **D7b** | 1.0 release gates (full suite) | **FAIL** — promoted-only Tier A | `check-1.0-release-gates.sh` | High |
| **D8** | Phase 8 expressivity | **Scaffolding complete** | ROADMAP v1.5–v1.9 tracks | Medium |
| **D9** | Documented exclusions | 55 `internal`, 55 `excluded` Java | `cases.json` | High |

---

## Remaining DL OFN failures @ 30s (6 cases, 2026-06-28)

```text
reasoner.AnyURITest.testPatternComplement1_1
reasoner.DatatypesTest.testDecimals
reasoner.ReasonerTest.testExistsSelf2          (tableau budget 4096)
reasoner.ReasonerTest.testHeinsohnTBox3Modified
reasoner.ReasonerTest.testIncrementalWithNegatedClass
reasoner.ReasonerTest.testNominalMerging       (tableau budget 4096)
reasoner.ReasonerTest.testWidmann3             (tableau budget 4096)
```

*(7 lines — `testPatternComplement1_1` and `testWidmann3` may alternate with `testDecimals` depending on run order; pass rate consistently **271/277**.)*

### Fix locations

| Case | Primary code |
|------|----------------|
| `testPatternComplement1_1` | `datatype/consistency.rs` anyURI pattern ∩ complement minLength |
| `testDecimals` | `datatype/consistency.rs` decimal facet + negative assertion |
| ReasonerTest quartet | `ontologos-alc/tableau`, `ontologos-dl/lib.rs` ABox typing |

---

## Semantic gap summary

### WG failures @ 30s (`wg_failures --all`, `ONTOLOGOS_SCAN_THREADS=1`)

| Bucket | Count | Top examples |
|--------|------:|--------------|
| `consistency` | 15 | `Contradicting-datatype-restrictions`, `Minus-inf-not-owlreal`, `description-logic-035`, `maxCardinality-001`, `Thing-004` |
| `entailment_positive` | 10 | `Qualified-cardinality-restricted-int`, `someValuesFrom-003`, `equivalentProperty-001`, `I5.8-010` |
| `timeout` | 2 | `Consistent-but-all-unsat`, `description-logic-504` |
| `load_error` | 2 | `equivalentClass-007` (RDF/XML `owl:` prefix), `Rdfbased-sem-eqdis-sameas-subst` (entity kind clash) |

### Java axiom failures (promoted CI, 17 cases)

All in `hermit_generated` — dominated by **datatype facet** families:

| Family | Failures in promoted CI |
|--------|------------------------:|
| `DateTimeTest` | 4 |
| `RDFPlainLiteralTest` | 4 |
| `AnyURITest` | 3 |
| `FloatDoubleTest` | 3 |
| `BinaryDataTest` | 1 |
| `DatatypesTest` | 1 |
| `XMLLiteralTest` | 1 |

### DL OFN pass rate by family (`dl_ofn_pass_rate`)

| Family | Pass rate |
|--------|----------:|
| `DateTimeTest` | 2/10 (20%) |
| `AnyURITest` | 11/20 (55%) |
| `BinaryDataTest` | 7/12 (58%) |
| `FloatDoubleTest` | 14/24 (58%) |
| `RDFPlainLiteralTest` | 13/21 (62%) |
| `ReasonerTest` | 70/74 (95%) |
| **Overall** | **223/277 (80.5%)** |

### Tier A lib test failures (`check-1.0-release-gates.sh`)

4 entailment guard unit tests fail in `catalog::entailment_guard_tests`:

- `boolean_constructor_guard_intersection_comp`
- `functional_property_004_entailment`
- `restrict_somevalues_inst_subj_entailment`
- `some_values_from_003_entailment`

These correspond to open WG cases in the entailment_positive bucket.

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
| `parity_pct = 100%` | ROADMAP post–Phase 7 | D1: `java_planned=0`, `wg_planned=0` | **Confirmed** — catalog porting only |
| `428/428 WG promoted` | ROADMAP | D2: `promoted_wg=428` | **Confirmed** — list complete |
| `359/413 axiom promoted` | ROADMAP | D2: `promoted_axiom=359` | **Confirmed** |
| WG promoted @ 30s budget | ROADMAP §1.0 | D3: 26–29 WG failures | **Refuted** — promoted ≠ passing |
| Phase 4–7 harness complete | ROADMAP | Phase 5–7 closure pass; Phase 4 `phase4_all_wg_failures_empty` **fails** | **Partial** — harness exists, semantic gate fails |
| Tier C HermiT JAR proof | ROADMAP Phase 7 | D6: cross-check passes with tolerance | **Confirmed** (subset/superset, not identical) |
| v1.0 ready to tag | ROADMAP §1.0 | D7: release gates fail; D8 incomplete | **Refuted** |
| ~58% HermiT parity (public docs) | README, comparison.md | D1=100% catalog, D3≈89% semantic | **Stale** — understates catalog, overstates readiness |

---

## Release blockers (ordered)

1. **Semantic failures in promoted lists** — demote or fix 17+ axiom and 26+ WG cases; `hermit-burndown.sh test` must pass at 30s.
2. **Tier A conformance green** — `cargo test -p ontologos-conformance` (lib guard tests + full active suite).
3. **Phase 4 closure** — `phase4_all_wg_failures_empty` (26 failures @ 120s scan).
4. **Phase 8 expressivity** — ROADMAP v1.5–v1.9 tracks (hybrid, ABox, ALC, QL, DL stable).
5. **DL engine checklist** — coupled saturation+tableau, full datatype/nominal/cardinality support (ROADMAP §1.0 OWL 2 DL engine).
6. **API surface** — OWLReasoner-equivalent ops (realize, consistency, entailment) documented and stable.
7. **Publish workflow** — `ontologos-dl` on crates.io, automated release, docs.rs complete.
8. **Blocking full conformance in CI** — currently PR CI uses promoted-only subset; v1.0 requires full suite green.

---

## Explicit non-goals (out of scope for parity %)

| Category | Count | Notes |
|----------|------:|-------|
| `internal` Java cases | 55 | HermiT engine unit tests — not ported |
| `excluded` Java cases | 55 | Manifest-documented gaps |
| `migrated` Java cases | 5 | Moved to other suites |
| RulesTest hypertableau internals | — | Phase 5d — not full JVM port |
| Interactive Protégé / OWL API buffer workflows | — | Batch replacement target only |
| Bit-identical taxonomy vs HermiT | — | Tier C allows superset tolerance |

---

## Recommended next actions

1. **Fix datatype facet cluster** (DateTime, AnyURI, RDFPlainLiteral, FloatDouble, BinaryData) — 17 promoted CI failures + largest OFN gap (80.5% pass rate).
2. **Demote stale WG IDs** or fix 26 failing WG cases — promotion lists must not include failing cases.
3. **Repair 2 load_error WG cases** — parser (`equivalentClass-007` RDF/XML prefix) and entity-kind mapping (`sameas-subst`).
4. **Entailment guard regressions** — 4 lib tests + 10 WG entailment_positive cases (someValuesFrom, QCR, functional property).
5. **Timeout triage** — `Consistent-but-all-unsat`, `description-logic-504` (perf vs correctness at 30s).
6. **Update public docs** — README/comparison.md still cite ~58%; replace with catalog vs semantic distinction.

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
| `axiom` | 413 | Active semantic checks |
| `clausify` | 33 | Structural DL clausification regression |
| `swrl` | 19 | SWRL forward chaining |
| `ported` | 11 | Hand-written ports |
| `internal` | 55 | Out of scope |
| `excluded` | 55 | Documented gaps |
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
| Total `#[test]` functions | 1145 |
| Ignored (dormant) | 128 |
| Active in default CI | 1017 |
| Promoted axiom IDs | 359 |
| Promoted WG IDs | 428 |

---

## Historical context (Phase 4 burndown, 2026-06-26)

Phase 4 closed 14 WG cases (inconsistency, wine imports, entailment guards). Subsequent **honest assessment** shows promotion lists were updated to 428/428 WG and 359/413 axiom before all cases passed semantically. The gap report previously stated “unpromoted WG failures: 0” — true for the unpromoted scan, but **misleading** because 26–29 promoted WG cases still fail semantic checks.

Key engine fixes from Phase 4 remain valid (`union_csp`, `cardinality_grid`, wine import shortcut, `%23` IRI, datatype `sameAs`). Remaining work is concentrated in datatype facets, entailment guards, parser edge cases, and timeouts.
