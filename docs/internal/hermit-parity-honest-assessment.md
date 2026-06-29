# Brutally Honest HermiT Parity Assessment

**Updated:** 2026-06-29  
**Audience:** Maintainers, evaluators, and adopters deciding whether OntoLogos replaces HermiT  
**Related:** [hermit-parity-gap-report.md](hermit-parity-gap-report.md) (live metrics) · [ROADMAP.md](../../ROADMAP.md) · [comparison.md](../comparison.md)

---

## Bottom line

| Question | Honest answer |
|----------|---------------|
| Has OntoLogos hit its **own v1.0 conformance gate**? | **Yes, on `main`.** `parity_pct = 100%`, 0 planned backlog, 1009 active tests, CI @ 30s. |
| Is it **100% HermiT parity** in the everyday sense? | **No.** Roughly **75–85%** for batch DL on supported constructs inside the gated suite; **~50%** as a drop-in Protégé/HermiT replacement. |
| Can you ship it as HermiT today? | **Not on PyPI/crates.io** — production is still **v0.9.0** (no stable DL). **v1.0.0 tag is pending publish**, not pending engine work. |

The project is honest about this in [comparison.md](../comparison.md) and [FAQ.md](../../FAQ.md): *"HermiT functional parity on the gated conformance corpora, not a guarantee for every real-world ontology."*

Regenerate live metrics:

```bash
bash benchmarks/scripts/hermit-burndown.sh status
bash benchmarks/scripts/report-conformance-coverage.sh
bash benchmarks/scripts/check-1.0-release-gates.sh
```

---

## What "100% parity" actually measures

The metric is **catalog harness completeness**, not total HermiT equivalence.

```text
in_scope_total = (591 Java − 55 internal − 70 excluded − 5 migrated) + 428 WG
               = 461 + 428 = 889

parity_pct = 100 × (1 − planned / in_scope_total)
```

**Typical live numbers** (from `parity_status` + `report-conformance-coverage.sh`):

| Metric | Value |
|--------|------:|
| Java catalog cases | 591 |
| OWL WG cases | 428 |
| **In-scope** (parity denominator) | **889** |
| **Out-of-scope Java** | **130 (22%)** |
| `java_planned` / `wg_planned` | 0 / 0 |
| Active CI tests @ 30s | 1009 |
| Dormant `#[ignore]` tests | 143 |
| Promoted axiom / WG IDs | 400 / 428 |

**Translation:** OntoLogos claims parity only after **redefining 130 Java HermiT tests out of scope** and running the rest under a **30-second per-operation wall clock**. That is a legitimate, well-documented gate — but it is not "we pass every test HermiT ever had."

```mermaid
flowchart TB
  subgraph hermit [HermiT catalog 1019 cases]
    Java591[Java 591]
    WG428[WG 428]
  end
  subgraph out [Explicitly out of scope 130 Java]
    Internal55[internal 55]
    Excluded70[excluded 70]
    Migrated5[migrated 5]
  end
  subgraph inscope [In-scope gate 889]
    Java461[Java 461 runnable]
    WGAll[WG 428]
  end
  subgraph pass [Phase 9 claim]
    Green[1009 tests green at 30s]
  end
  Java591 --> Internal55
  Java591 --> Excluded70
  Java591 --> Migrated5
  Java591 --> Java461
  WG428 --> WGAll
  Java461 --> Green
  WGAll --> Green
```

---

## What is genuinely strong (credit where due)

These are real achievements, not marketing:

1. **OWL WG entailment suite: 428/428 @ 30s** — strongest signal. These are W3C-standard semantic tests; passing all of them is meaningful DL correctness evidence ([hermit-parity-gap-report.md](hermit-parity-gap-report.md) D3).

2. **277/277 DL OFN axiom ports @ 30s** — HermiT-derived single-ontology entailment checks pass under CI budget.

3. **Tier B classification fixtures** — Pizza, Wine, GALEN, Propreo taxonomies match vendored HermiT goldens with **zero missing edges** (`benchmarks/scripts/compare-classification-fixtures.sh`).

4. **Tier C HermiT JAR cross-check** — HermiT ⊆ OntoLogos on family/pizza/go-subset (zero missing subsumptions). OntoLogos is a **superset**, not identical.

5. **Engineering discipline** — `check-1.0-release-gates.sh` is blocking in CI; internal docs track claims vs evidence.

6. **EL/RL/RDFS tracks** — Production-stable on v0.9.0; not HermiT-competitive territory but solid for those profiles.

---

## Where parity is explicitly waived (the uncomfortable part)

### 1. Twenty-two percent of Java HermiT catalog never counted

From `tests/hermit/generate_catalog.py` `EXCLUDED_IDS` and status rules:

| Bucket | Count | Examples |
|--------|------:|---------|
| `internal` | 55 | HermiT engine unit tests (normalization, blocking, tableau internals) |
| `excluded` | 70 | Ian/ComplexConcept CE (13), OWLLink Bob tests, datatype literal parsing, RIA regularity (4), hierarchy printing, OWL API error paths |
| `migrated` | 5 | Moved elsewhere |

Notable exclusions with **real semantic gaps** behind them:

- **13 Ian/ComplexConcept CE cases** — excluded for *tableau soundness gaps*; partially covered in `ontologos-alc/tests/ian_ce_sat.rs` but not in conformance
- **4 RIA regularity tests** — "full OWL 2 algorithm deferred"
- **OWLLink interactive/buffered tests** — largely excluded; buffered reasoner parity not claimed
- **Property hierarchy with inverses** — parser not mapped (`testSubProperties`, `testObjectPropertyHierarchy`)

### 2. Taxonomy is not HermiT-identical — by design

[Tier C tolerance policy](../reference/taxonomy-tolerance.md):

| Corpus | HermiT edges | Extra edges allowed |
|--------|-------------:|--------------------:|
| `family.owl` | 39 | 13 |
| `pizza.owl` | 8,453 | **8,285** |
| `go-subset.owl` | 3,240 | **3,160** |

Rule: up to **5 extra edges or 1% of HermiT count**, whichever is larger. OntoLogos is validated as **sound superset**, not bit-identical output. For Pizza DL, that means ~98% "extra" taxonomy noise is acceptable in nightly cross-checks.

**Brutal truth:** If your workflow depends on HermiT printing exactly the same hierarchy, OntoLogos is not at parity.

### 3. Not a hypertableau port — different engine entirely

[hermit-replacement.md](research/hermit-replacement.md) states plainly:

> OntoLogos does **not** plan a line-by-line HermiT hypertableau port.

`ontologos-dl` is a **Konclude-style hybrid** (saturation + tableau via `ontologos-alc`). Algorithmic parity with HermiT is **0%** by definition; only semantic agreement on test corpora is claimed.

### 4. SWRL / RulesTest: minimal coverage

- 24 `RulesTest` cases in catalog; only **19 active `swrl`** tests
- ROADMAP: full `RulesTest` **deferred out of 1.x**
- `swrl` profile is **Preview**, not production

### 5. Parser and construct gaps vs HermiT's full OWL surface

From [supported-constructs.md](../reference/supported-constructs.md):

- `owl:imports` **not resolved**
- Negative property assertions skipped in core
- RDFS gaps via reasonable: **no** `subPropertyOf` transitive closure, **no** domain/range inheritance
- RL clash detection incomplete (functional duplicates, asymmetric reverse pairs)
- `axiom_count()` ≠ Protégé logical axiom count

HermiT loads and reasons over constructs OntoLogos may scan-but-skip or handle only on the DL path.

### 6. Performance is far from HermiT/Konclude

[taxonomy-tolerance.md](../reference/taxonomy-tolerance.md) timeout table:

| Corpus | ROADMAP target | Actual (release) |
|--------|----------------|------------------|
| Family DL | <100 ms | ~0.5 s |
| Pizza DL | <30 s | **~5 min** |
| go-subset DL | <10 s | **~2 min** |

Pizza DL is **not PR-gated** (`RUN_SLOW_DL_GATES=1` only). Konclude 10× benchmark was **waived** via ADR for v1.0. Phase 8 "expressivity complete" includes multiple **waived** checklist items ([ROADMAP.md](../../ROADMAP.md) v1.9 section).

**Brutal truth:** OntoLogos may be semantically close on small tests but is not a performance replacement for HermiT/Konclude on medium DL ontologies.

### 7. OWL API / Protégé replacement: not close

[hermit-replacement.md](research/hermit-replacement.md) OWL API matrix — many methods are partial or deferred:

| Capability | HermiT | OntoLogos |
|------------|--------|-----------|
| `isConsistent` | Yes | Partial (DL / EL fragments) |
| `isEntailed` | Yes | 0.6 explain + 1.8 query (incomplete) |
| `flush` / buffered changes | Yes | 0.7 incremental (not OWLLink-parity) |
| Interactive IDE workflow | Yes | Explicit non-goal |

See [profile-stability.md](../guides/profile-stability.md) for channel-specific DL recommendations (`main` vs PyPI v0.9.0).

### 8. Production channel lag

| Channel | DL available? | Parity claim |
|---------|---------------|--------------|
| **PyPI / crates.io v0.9.0** | No stable DL | EL/RL/RDFS only |
| **`main` workspace 1.0.0** | Yes, gates green | Full gated parity |
| **v1.0.0 tag** | Not shipped | Publish pending |

Until v1.0 ships, **production HermiT users cannot switch** without building from git.

### 9. CI budget artifacts

- **30s wall clock** per DL operation in blocking CI — hard cases excluded (`testIanBackjumping3`) or may behave differently at 120s (nightly only)
- `check-1.0-release-gates.sh` **skips** `planned_engine_failure_scan` and `ian_backjumping3_axiom_check_completes_within_budget`
- **143 ignored** conformance tests exist (run nightly, not blocking PR CI)

---

## Dimension scorecard (brutal edition)

| Dimension | Score | Notes |
|-----------|------:|-------|
| **Self-defined catalog gate (`parity_pct`)** | **100%** | Met on `main`; well-defined, auditable |
| **All HermiT+WG catalog cases (literal)** | **~87%** | 889/1019 in scope; rest waived |
| **Semantic entailment on gated tests** | **~95%** | 428 WG + 277 OFN + clausify/swrl; exclusions for known gaps |
| **Taxonomy identity vs HermiT** | **~30–50%** | Sound superset policy; Pizza allows 8k+ extra edges |
| **OWL API / Protégé drop-in** | **~40%** | Batch classify only; no IDE, buffered OWLLink, imports |
| **SWRL / RulesTest** | **~20%** | 19 tests; full suite deferred |
| **Performance vs HermiT** | **~15–25%** | Family OK-ish; Pizza/go-subset minutes not seconds |
| **Production availability** | **~0% for DL** | v0.9.0 has no DL; 1.0 not tagged |
| **Algorithmic equivalence** | **0%** | Different engine by design |

**Weighted honest overall:** For the **stated v1.0 goal** ("replace HermiT for batch DL classification on gated corpora within supported constructs"): **~80–85%** — engine work largely done, publish and real-corpus validation remain.

For **"I can uninstall HermiT and never look back"**: **~50–60%** — exclusions, taxonomy differences, performance, parser gaps, and missing Protégé/OWL API surface are all real blockers.

---

## What the project gets right about honesty

Unlike many parity claims, OntoLogos documents the gaps:

- [comparison.md](../comparison.md): *"not yet a drop-in HermiT replacement on arbitrary ontologies"*
- [FAQ.md](../../FAQ.md): validate against HermiT/Konclude outside the gated suite
- [hermit-parity-gap-report.md](hermit-parity-gap-report.md): explicit non-goals table
- [hermit-replacement.md](research/hermit-replacement.md): no hypertableau port; 2.0 is Konclude-class perf

The marketing tension is that **ROADMAP/README headline "100% parity"** while comparison/FAQ correctly narrow the claim. The number is true for the gate; it overstates everyday HermiT equivalence.

---

## Remaining work beyond the conformance gate

Already done on `main`:

- Catalog porting, WG suite, release gate script green

Still open for a credible **production HermiT replacement** narrative:

1. **Ship v1.0.0** — crates.io `ontologos-dl`, PyPI, docs.rs ([ROADMAP Phase 9](../../ROADMAP.md))
2. **Close or re-include 13 Ian/ComplexConcept CE exclusions** — tableau soundness, not harness bookkeeping
3. **Tighten taxonomy output** — reduce superset noise on Pizza/go-subset (or document why extras are harmless)
4. **Pizza DL performance** — ~5 min vs <30 s target; not blocking gate but blocks real adoption
5. **Doc alignment** — [profile-stability.md](../guides/profile-stability.md) reflects `main` vs PyPI channel semantics
6. **Optional but meaningful:** OWLLink subset, imports resolution, full RulesTest, RIA regularity algorithm

---

## Verdict for evaluators

**If you need HermiT today:** use HermiT or Konclude. PyPI OntoLogos 0.9.0 does not include stable DL.

**If you can build from `main` and your ontology fits the supported-construct subset:** OntoLogos is **credible for batch DL classification** on corpora similar to the gated suite — with the caveat that taxonomy output may differ (more edges) and hard ontologies may hit 30s limits.

**If "100% parity" means pass every HermiT test, identical hierarchies, Protégé workflows, and Konclude speed:** OntoLogos is **not there** and the project's own docs admit several of those gaps. The **100% figure is a harness milestone**, not a universal HermiT clone score.
