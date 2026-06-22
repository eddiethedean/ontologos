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

## Executive summary (2026-06-22)

| Signal | Value |
|--------|------:|
| **parity_pct** (in-scope catalog) | **~58%** (397 planned / 958 in-scope) |
| Current ROADMAP phase | **2** (assertion harvest) |
| Active CI conformance tests | **593** |
| 1.0 gate target | ≥400 (**pass**) — necessary not sufficient |
| `check-1.0-release-gates.sh` | **PASS** (Tier A + Tier C on `main`) |
| `check-hermit-parity-phases.sh` | **FAIL** until Phase 9 |
| Catalog `axiom` cases | **136** |
| Promoted axiom IDs | **136** (`promoted_axiom_ids.txt`) |
| DL OFN pass rate | **100%** (115/115 with assertions) |
| Planned DL semantic failures (`dl_failures`) | **0** |
| Planned Java backlog | **330** — see [planned-backlog-triage.md](planned-backlog-triage.md) |
| Planned WG backlog | **67** |

**Verdict:** Not full HermiT parity. Strong on EL, promoted OFN DL axioms, and Tier C gates (`family.owl`; optional `pizza.owl` / `go-subset.owl`). Production OWL DL still requires HermiT/Konclude per [FAQ.md](../../FAQ.md).

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
