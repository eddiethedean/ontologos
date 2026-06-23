# HermiT parity gap report

**Updated:** 2026-06-22 (live tooling — re-run scripts before trusting counts)  
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
```

---

## Executive summary (2026-06-23)

| Signal | Value |
|--------|------:|
| **parity_pct** (in-scope catalog) | **~75%** (234 planned / 958 in-scope) |
| Current ROADMAP phase | **3** (DL engine gaps — in progress) |
| Active CI conformance tests | **~680+** (233 axiom + wg + …) |
| Catalog `axiom` cases | **233** |
| Promoted axiom IDs | **233** (`promoted_axiom_ids.txt`) |
| Planned `engine_gap` (audit) | **0** (down from 72) |
| Bounded `engine_failures` | **0** |
| Planned Java backlog | **~234** — see [planned-backlog-triage.json](planned-backlog-triage.json) |
| Planned WG backlog | **67** |

**Status (2026-06-23):** `engine_gap` **0** (was 72); bounded `engine_failures` **0** (was 39); **233** catalog `axiom` cases; **233** promoted IDs. IanT6 (functional `f` clash + `add_role_edge` in ABox materialization), IanT7b (defer transitive saturation during ∃ expansion), IanT1c, IanT5, nominals3/6, smoke suite, and CE probe harness fixes landed. **Phase 3 engine exit criteria met** pending promotion scan + full conformance green.

**Recent fixes (2026-06-22):** ROADMAP parity phases (P0–P9); Widmann + WG dl-026/601/626 enabled; WG catalog override fix; optional DL corpus goldens + HermiT JAR cross-check harness.

---

## Conformance harness snapshot

### Catalog (`benchmarks/data/hermit/catalog/cases.json`)

| Status | Count | Meaning |
|--------|------:|---------|
| `axiom` | 136 | Active semantic checks |
| `planned` | 330 | Roadmap — triaged in `audit-planned-backlog.sh` |
| `clausify` | 33 | Structural DL clausification regression |
| `swrl` | 24 | SWRL forward chaining |
| `internal` | 55 | Engine-internal (ignored) |
| `ported` | 10 | Hand-written redirects |
| **Total** | **591** | |

### CI execution

| Metric | Value |
|--------|------:|
| Tests defined (conformance crate) | **1063** |
| **Active (default `cargo test`)** | **593** |
| Ignored | **470** |

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

See [taxonomy-tolerance.md](../reference/taxonomy-tolerance.md) and `benchmarks/scripts/compare-dl-hermit-crosscheck.sh`.

---

## Parity phases

See [ROADMAP.md § HermiT parity phases](../../ROADMAP.md#hermit-parity-phases-path-to-v100-tag). Progress formula:

```text
parity_pct = 100 × (1 − (java_planned + wg_planned) / 958)
```

---

## Promotion loop

```bash
bash benchmarks/scripts/promote-hermit-catalog.sh
```

Planned backlog categories: `bash benchmarks/scripts/audit-planned-backlog.sh`

---

## Historical note

Older revisions cited 67 DL failures, 100–211 active tests, or 119/121 OFN pass rate. Always prefer script output over this file.
