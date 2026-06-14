# HermiT parity gap report

**Updated:** 2026-06-13 (live tooling — do not trust stale counts in older sections below without re-running scripts)  
**Target release:** **1.0** — functional HermiT replacement ([ROADMAP.md](../../ROADMAP.md) §1.0)

**Triage commands (source of truth):**

```bash
bash benchmarks/scripts/report-conformance-coverage.sh
bash benchmarks/scripts/check-1.0-release-gates.sh
bash benchmarks/scripts/parity-scan.sh
cargo run --release -p ontologos-conformance --bin dl_failures
cargo run --release -p ontologos-conformance --bin dl_ofn_pass_rate
cargo run --release -p ontologos-conformance --bin promote_catalog
bash benchmarks/scripts/promote-hermit-catalog.sh
```

---

## Executive summary (2026-06-13)

| Signal | Value |
|--------|------:|
| Active CI conformance tests | ~211 |
| 1.0 gate target | ≥400 |
| Catalog `axiom` cases | 133 |
| Promoted axiom IDs | 133 (`promoted_axiom_ids.txt`) |
| DL OFN pass rate | ~98% (119/121 with assertions) |
| Planned DL semantic failures | 4 (`dl_failures` bin) |

Recent correctness fixes: Widmann/NI-rule blocking (UNSAT), flower-ontology cardinality (`c2 ⊑ c4`), RL `is_consistent` via saturation, existential equivalence subsumption bridge (`testSubsumption2/3`), parser DL mapping for complex intersections, WG entailment merge check.

**Remaining planned DL failures (incremental / nominals):**

- `testIncrementalWithNegatedClass`
- `testIncrementalWithNegatedHasSelf`
- `testIncrementalWithNegatedHasValue`
- `testNominals2`

---

## Conformance harness snapshot

### Catalog (`benchmarks/data/hermit/catalog/cases.json`)

| Status | Count | Meaning |
|--------|------:|---------|
| `planned` | ~356 | OFN/fixture present; engine not yet passing or missing assertions |
| `axiom` | 133 | Active semantic checks (`run_hermit_case`) |
| `clausify` | 33 | Structural DL clausification regression |
| `ported` | 10 | Hand-written in `hermit_rl` / `hermit_rdfs` / `hermit_el` |
| `internal` | 55 | Parser/normalization smoke |
| `excluded` | 2 | Documented out-of-scope |
| `fixture` | 2 | Resource XML goldens |
| `migrated` | 3 | Moved to another suite |
| **Total** | **594** | |

### CI execution

| Metric | Value |
|--------|------:|
| Tests defined (HermiT + WG) | 1066+ |
| **Active (default `cargo test`)** | **~211** |
| Ignored | ~855 |

### OWL WG

428 WG cases cataloged; 3 approved subset with vendored RDF. `wg_entailment_holds` uses premise∪conclusion taxonomy diff (not separate classify-and-compare).

---

## Promotion loop

```bash
bash benchmarks/scripts/promote-hermit-catalog.sh
```

Hand-authored catalog assertions: `HARDCODED_AXIOM_SUBSUMPTIONS` in `tests/hermit/generate_catalog.py` for OFN cases Java cannot extract (e.g. `testSubsumption2/3`). ~86 planned DL cases still lack extracted subsumptions — grow via Java `assertSubsumedBy` extraction and manual OFN assertions.

---

## CI / release gates

- `check-1.0-release-gates.sh` wired in `.github/workflows/ci.yml` and `release.yml` (informational until ≥400 active tests).
- `dl_ofn_pass_rate` delegates to `check_axiom_case` for honest pass-rate parity with promotion scan.

---

## Historical note

Older revisions of this document cited 67 DL failures, 26 axiom cases, and 177 active tests. Always prefer script output over this file for triage.
