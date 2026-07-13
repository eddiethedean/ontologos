# Test Suite Verification Report

Date: 2026-07-12  
Scope: Full phased AI test-suite verification plan

## Executive summary

**Overall confidence: Moderate** (up from Moderate–Low at baseline)

The suite now has stronger independent oracles in hotspots (conformance guards, facade/contract smokes, profile pins, semantic fixtures). HermiT/WG volume still dominates (~900 generated tests); confidence in DL entailment depends on guard soundness fixes and the new guard/full agreement meta-test.

## Incorrect tests (fixed)

| Test / area | Defect | Fix |
|-------------|--------|-----|
| `consistent_but_all_unsat_entailment_guard` | Returned `Ok(true)` on IRI-shape match without verifying each class is ⊥ | Removed weak guard; deferred WG case (`status=deferred`, `#[ignore]`) until `named_classes_unsatisfiable` proves ⊥ |
| `contract` / `facade` family smokes | Tautology + wrong RL oracle (`Child ⊑ Relative` is DL union) | Pin RL profile + `hasChild` range `Person` (matches `rl/tests/corpus.rs`) |
| `profile_corpora::family_dl_profile_detected_or_rl` | Disjunctive `Rl \| Dl` | Pin `Rl` per profile-detection guide |
| `classify_smoke::detects_disjoint_unsatisfiable_class` | Disjunctive unsat | Both classes must be ⊥ |
| `test_version.py::test_classify_rdfs_profile_materializes` | `inferred_axioms >= 0` | Require `>= 1` and strict axiom count increase |

## Weak tests (strengthened)

| Area | Change |
|------|--------|
| `profile_rules` | Negative-only `assert_ne!` → exact `OwlProfile` per detection rules (e.g. `ObjectAllValuesFrom` → Ql) |
| `pizza_el.rs` | `!is_empty()` → `A ⊑ B` pair |
| `hermit_el.rs` | Added golden edge count floor |
| `hermit_rules.rs` | SWRL body/head atom counts |
| `js/smoke.rs` | Exact subsumption pair + count |
| `json_roundtrip.rs` | DL CE + SWRL atom payload equality after round-trip |
| Dump-only DL triage tests | Gated with `#[ignore]` |

## Tests added

| File | Purpose |
|------|---------|
| `entailment_guards::positive_entailment_guards_agree_with_full_reasoner_on_wg_samples` | Guards must not be looser than full reasoner |
| `el/tests/negative_subsumption.rs` | EL must not invent reverse/sibling subsumptions |
| `rl/tests/documented_gaps.rs` | Ignored test for reasonable `equivalentProperty` gap |
| `contract/tests/must_not.rs` | `from_file` → `ParseNotAvailable`; facade classify path |
| `contract/tests/semantic_fixtures.rs` | Cross-crate semantic oracle from JSON |
| `conformance/tests/catalog_honesty.rs` | Surfaces covered/excluded inventory |
| `parser/tests/mapping_oracle.rs` | Family.owl axiom kind counts |
| `py/tests/test_semantic_fixtures.py` | Python parity with shared fixtures |
| `benchmarks/data/semantic-fixtures.json` | Shared binding oracle |
| `docs/internal/test-oracle-policy.md` | Maintainer anti-pattern guide |

## Tests removed

None deleted. Dump-only triage tests demoted to `#[ignore]` (9 functions).

## Missing coverage (remaining)

- Full audit of all ~40 positive entailment guards (one class fixed; meta-test covers 4 WG samples)
- `cargo-mutants` run not executed in this pass (documented in oracle policy)
- Java/.NET/C binding semantic tests still smoke-only
- Full `ontologos-conformance --release` not run (long-running; stratified tests executed)

## AI failure patterns found

1. **Shared hallucinations** — weak consistent-but-all-unsat guard matched implementation-shaped IRIs
2. **Tautologies** — facade/contract family smokes always passed
3. **Floor asserts** — `>= 0`, `>= 1`, `!is_empty()` in Python/RL/EL tests
4. **Silent skips** — missing fixture early return
5. **Dump-only tests** — WG triage `eprintln!` without asserts counted as green
6. **Disjunctive success** — `Rl \| Dl`, `unsat A \| unsat B`

## Verification commands executed

```bash
cargo test -p ontologos-conformance --test entailment_guards
cargo test -p ontologos-contract --test fixtures --test must_not --test semantic_fixtures
cargo test -p ontologos-facade getting_started_classify_family_auto
cargo test -p ontologos-profile --test profile_rules
cargo test -p ontologos-conformance --test profile_corpora family_detects
cargo test -p ontologos-el --test negative_subsumption --test pizza_el
cargo test -p ontologos-dl --test classify_smoke
cargo test -p ontologos-core --test json_roundtrip
cargo test -p ontologos-js
cargo test -p ontologos-parser --test mapping_oracle
cargo test -p ontologos-conformance --test catalog_honesty
```

(Run output captured during implementation; full workspace + conformance release suite recommended before release.)

## Confidence assessment

| Layer | Confidence |
|-------|------------|
| EL taxonomy (Pizza, minimal, negative) | **Moderate–High** |
| RL/Family Auto routing | **Moderate** |
| DL entailment guards | **Moderate** (meta-test + one guard removed) |
| Bindings | **Moderate** (shared fixtures; JS/Python strengthened) |
| Full HermiT catalog | **Moderate** (unchanged volume; guard layer improved) |

Behavioral verification improved materially in hotspots; full **High** confidence requires mutation testing pass and full conformance release run without guard/full mismatches.
