# HermiT parity gap report

**Updated:** 2026-06-23 (Phase 3 complete)  
**Target release:** **1.0** — functional HermiT replacement ([ROADMAP.md](../../ROADMAP.md) § [HermiT parity phases](../../ROADMAP.md#hermit-parity-phases-path-to-v100-tag))

**Triage commands (source of truth):**

```bash
bash benchmarks/scripts/report-ci-gate-status.sh
bash benchmarks/scripts/report-conformance-coverage.sh
bash benchmarks/scripts/check-1.0-release-gates.sh
bash benchmarks/scripts/check-hermit-parity-phases.sh
bash benchmarks/scripts/audit-planned-backlog.sh
bash benchmarks/scripts/parity-scan.sh
cargo run --release -p ontologos-conformance --bin dl_failures
cargo run --release -p ontologos-conformance --bin dl_ofn_pass_rate
cargo run --release -p ontologos-conformance --bin promote_catalog
bash benchmarks/scripts/promote-hermit-catalog.sh
cargo test -p ontologos-conformance --test phase3_closure
```

---

## Executive summary (2026-06-23)

| Signal | Value |
|--------|------:|
| **parity_pct** (in-scope catalog) | **~72%** (267 planned / 958 in-scope) |
| Current ROADMAP phase | **4** (WG fixtures) — **Phase 3 complete** |
| Catalog `axiom` cases | **270** |
| Promoted axiom IDs | **270** (`promoted_axiom_ids.txt`) |
| Planned `engine_gap` (audit) | **0** |
| Planned `promotion_candidate` (audit) | **0** |
| Planned `missing_assertions` (audit) | **0** |
| Bounded `engine_failures` | **0** |
| Planned Java backlog | **200** (`manual_port` only) |
| Planned WG backlog | **67** |

**Phase 3 exit (met):** `engine_gap` **0**; `promotion_candidate` **0**; `missing_assertions` **0**; **270** promoted axiom cases (+43); direct-type retrieval for `testDirect`; `phase3_closure` gate **4/4** green.

**Next (Phase 4):** vendor WG premise RDF (~52), fill WG expectations (~15), `wg_planned → 0`.

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
| `wg` (active) | 361 |
| `planned` | 67 |

---

## Tier C corpora

| Corpus | CI default | HermiT cross-check |
|--------|------------|-------------------|
| `family.owl` | Yes | When `HERMIT_JAR` set |
| `pizza.owl` | Optional (`RUN_SLOW_DL_GATES=1`) | When `HERMIT_JAR` set |
| `go-subset.owl` | Optional (`RUN_SLOW_DL_GATES=1`) | When `HERMIT_JAR` set |
