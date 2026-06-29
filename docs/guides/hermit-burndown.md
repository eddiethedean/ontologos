# HermiT burndown guide

**Audience:** developers contributing to OntoLogos **v1.0** — HermiT parity is **complete** on the in-scope catalog (`parity_pct = 100%`, full suite @ 30s on `main`); this guide covers maintenance, regressions, and optional post-1.0 burndown.

**Start here if you are new:** you do not need a HermiT Java checkout for day-to-day work. Vendored fixtures under `benchmarks/data/hermit/` are enough.

---

## Why this exists

OntoLogos ships when it can **replace HermiT** for batch OWL DL classification and entailment on a ported test catalog — not when a single golden ontology happens to work.

The **HermiT burndown** is how we track that honestly:

| Question | Answer |
|----------|--------|
| **What is the goal?** | `parity_pct = 100%` — zero `planned` cases in the Java + OWL WG catalogs |
| **What blocks the v1.0 tag?** | [ROADMAP Phase 9](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md#phase-9--v100-tag-100-in-scope-parity) — catalog parity **and** expressivity gates |
| **What is “parity”?** | Every *in-scope* HermiT test has a runnable Rust conformance check that passes at the CI DL budget (30s) |
| **Why promoted lists?** | `phase9_closure` hygiene — see [Two tracks](#two-tracks-catalog-vs-ci) below |

---

## The scoreboard (one number)

```text
in_scope_total = (591 Java − internal − excluded − migrated) + 428 WG = 889
parity_pct     = 100 × (1 − (java_planned + wg_planned) / in_scope_total)
```

- **`planned`** = backlog — not yet ported or missing harvested assertions
- **`parity_pct`** = catalog porting progress (every in-scope case has a harness entry)
- **Semantic pass @ 30s** = active tests in `hermit_*_generated.rs` (blocking CI since Phase 9)

Check it any time:

```bash
bash benchmarks/scripts/hermit-burndown.sh status
```

Example output:

```text
HermiT burndown status
  parity_pct:      100.0%
  in_scope_total:  889
  backlog:         0 (java 0 + wg 0)
  promoted:        axiom 400 / wg 428 of 428 active
  runnable Java:   450
```

**`promoted`** lists record passing cases for `phase9_closure`; blocking CI runs the **full** active suite (no promotion filter since Phase 9).

---

## Two tracks: catalog vs CI

| Track | When | What runs | Purpose |
|-------|------|-----------|---------|
| **Catalog parity** | Always | `parity_status` / `hermit-burndown.sh status` | `parity_pct = 100%` when zero `planned` |
| **Full conformance (CI)** | Every PR since Phase 9 | All active `hermit_*_generated` tests @ 30s | Blocks merge on semantic regressions |
| **Promoted lists** | After fixes / `resync` | `phase9_closure` hygiene | Ensures `promoted_*_ids.txt` ⊆ passing |

```bash
# Full suite (same as blocking CI)
bash benchmarks/scripts/hermit-burndown.sh test-full

# Promoted-list hygiene + phase closures (also in release gates)
bash benchmarks/scripts/hermit-burndown.sh test
```

Blocking CI sets `ONTOLOGOS_DL_BUDGET_SECS=30` and runs the **full** active catalog (no `ONTOLOGOS_CI_PROMOTED_ONLY`). Nightly may use a longer budget via [conformance-nightly.yml](https://github.com/eddiethedean/ontologos/blob/main/.github/workflows/conformance-nightly.yml).

**Rule of thumb:** after fixing a case, run `promote` or `resync` so `phase9_closure` stays green.

---

## Mental model

```text
HermiT Java tests + OWL WG cases
        │
        ▼
tests/hermit/generate_catalog.py
        │
        ├── benchmarks/data/hermit/catalog/cases.json      (591 Java)
        ├── benchmarks/data/hermit/catalog/wg_cases.json   (428 WG)
        └── benchmarks/data/hermit/axioms/*.ofn            (fixtures)
        │
        ▼
crates/ontologos-conformance/tests/hermit_*_generated.rs   (one #[test] per case)
        │
        ▼
Engine crates (ontologos-dl, ontologos-alc, ontologos-rl, …)
        │
        ▼
promoted_axiom_ids.txt / promoted_wg_ids.txt               (CI gate lists)
```

| Catalog `status` | Meaning | Your job |
|------------------|---------|----------|
| `planned` | Backlog | Harvest assertions, hand-port, or fix engine then promote |
| `axiom` / `wg` | Runnable with semantic checks | Fix failures, then `promote` |
| `internal` / `excluded` / `migrated` | Out of scope | Ignore for parity % |

---

## Prerequisites

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # Pizza + checksum corpora
```

- Rust **1.88+** (see workspace `Cargo.toml`)
- No HermiT checkout required for burndown work (optional for full catalog regen)

---

## The daily loop (do this)

`hermit-burndown.sh` is the **only script you need to memorize**. Everything else is advanced.

```bash
# 1. Where are we?
bash benchmarks/scripts/hermit-burndown.sh status

# 2. What should I fix next?
bash benchmarks/scripts/hermit-burndown.sh triage

# 3. Fix engine / harvest assertions / hand-port (see below)

# 4. Regression check for your area
cargo test -p ontologos-conformance --release --test wg_phase4_check   # WG engine
cargo test -p ontologos-dl --test phase3_priority --release            # DL priority cases

# 5. Record passing cases for CI
bash benchmarks/scripts/hermit-burndown.sh promote

# 6. Verify PR gate
bash benchmarks/scripts/hermit-burndown.sh test
```

Print the loop anytime:

```bash
bash benchmarks/scripts/hermit-burndown.sh loop
```

### Why each step

| Step | Why |
|------|-----|
| `status` | Sub-second dashboard — parity %, backlog, unpromoted counts |
| `triage` | Scans only **unpromoted** WG failures (fast) + classifies Java `planned` backlog without slow engine runs |
| Fix | See [What kind of fix?](#what-kind-of-fix) |
| `promote` | Incremental — rescans only cases **not** already in `promoted_*_ids.txt`, updates CI lists |
| `test` | Same subset CI runs on PRs |

Use `triage --full` or `promote --full` when you need a complete catalog rescan (slow).

---

## What kind of fix?

After `triage`, pick the path that matches the failure:

### A. WG semantic failure (`status = wg`, test fails)

**Symptom:** `triage` lists OWL WG cases under consistency / entailment / timeout buckets.

**Where to work:** `crates/ontologos-dl`, `crates/ontologos-alc`, conformance harness in `crates/ontologos-conformance/src/catalog.rs`.

**Workflow:**

1. Pick one failure from `triage` (or `cargo run --release -p ontologos-conformance --bin wg_failures`)
2. Add a focused regression in `crates/ontologos-conformance/tests/wg_phase4_check.rs` if possible
3. Fix engine / parser / entailment guard
4. `hermit-burndown.sh promote` then `hermit-burndown.sh test`

### B. Java `planned` — missing assertions (Phase 5 harvest)

**Symptom:** `parity_status --audit-fast` shows `missing_assertions` (or `manual_port` for tests needing hand work).

**Where to work:** `tests/hermit/generate_catalog.py`, `tests/hermit/assertion_extractors.py`, `HARDCODED_*` blocks in the generator.

**Workflow:**

1. Find the Java test (optional: clone HermiT to `HermiT/` or set `ONTOLOGOS_HERMIT_ROOT`)
2. Extend assertion harvest → OFN fixture + expectations in `cases.json`
3. Regenerate: `python3 tests/hermit/generate_catalog.py --activate-all-from-disk`
4. If engine passes: `hermit-burndown.sh promote`

### C. Java `planned` — engine gap

**Symptom:** Case has assertions but `check_axiom_case` fails (`engine_gap` in full audit).

**Where to work:** Engine crate matching `case.engine` (`dl`, `rl`, `rdfs`, …).

**Workflow:**

1. `cargo run --release -p ontologos-conformance --bin engine_failures`
2. Fix engine
3. `hermit-burndown.sh promote --full` (or incremental if already `axiom` status)

### D. Hand-written port

**Symptom:** Test is RL/RDFS/EL logic better expressed inline than via OFN harvest.

**Where to work:** `crates/ontologos-conformance/tests/hermit_rl.rs`, `hermit_rdfs.rs`, `hermit_el.rs`; register in `tests/hermit/manifest.toml`.

See [tests/hermit/README.md](https://github.com/eddiethedean/ontologos/blob/main/tests/hermit/README.md) for catalog regeneration details.

---

## Command reference

### `hermit-burndown.sh` (preferred)

| Command | Speed | Use when |
|---------|-------|----------|
| `status` | &lt;1s | Starting a session; reporting progress |
| `triage` | Fast (unpromoted WG only) | Choosing the next fix |
| `triage --full` | Slow | Auditing entire catalog |
| `promote` | Incremental | After fixing one or more cases |
| `promote --full` | Slow | Refreshing all promoted lists |
| `test` | Medium | Pre-push CI parity |
| `test-full` | Slow | Claiming parity progress |
| `cleanup` | Instant | Stop stale burndown/cargo processes after interrupt |
| `loop` | Instant | Re-print the daily loop |

### `parity_status` (low-level dashboard)

Built to `target/release/parity_status` via `benchmarks/scripts/build-conformance-tools.sh`.

| Flag | Purpose |
|------|---------|
| *(none)* | Metrics only |
| `--scan` | Unpromoted WG failures |
| `--scan-full` | All WG failures |
| `--audit-fast` | Classify `planned` backlog without engine |
| `--audit` | Full planned backlog + engine failures |
| `--json` | Machine-readable output |

---

## Key files

| Path | Role |
|------|------|
| `benchmarks/scripts/hermit-burndown.sh` | **Start here** — unified workflow |
| `benchmarks/data/hermit/catalog/cases.json` | Java catalog + statuses |
| `benchmarks/data/hermit/catalog/wg_cases.json` | OWL WG catalog |
| `benchmarks/data/hermit/catalog/promoted_axiom_ids.txt` | CI gate — passing axiom cases |
| `benchmarks/data/hermit/catalog/promoted_wg_ids.txt` | CI gate — passing WG cases |
| `tests/hermit/generate_catalog.py` | Regenerate catalog + `hermit_*_generated.rs` |
| `crates/ontologos-conformance/src/catalog.rs` | Test runner, checks, scan tools |
| `ROADMAP.md` § HermiT parity phases | Phase checklist and exit criteria |
| `docs/internal/hermit-parity-gap-report.md` | Maintainer failure buckets (internal) |

Heavy steps (`triage`, `promote`, `test`, `test-full`) acquire an exclusive lock and **auto-clear stale processes** from prior interrupted runs. On Ctrl+C, child `cargo test` / scan binaries are terminated.

If a run was interrupted:

```bash
bash benchmarks/scripts/hermit-burndown.sh cleanup
```

---

## Environment variables

| Variable | Default | Meaning |
|----------|---------|---------|
| `ONTOLOGOS_DL_BUDGET_SECS` | `30` in CI; `120` in `test-full` | Wall-clock cap per DL operation |
| `ONTOLOGOS_CI_PROMOTED_ONLY` | unset in blocking CI | Legacy: skip non-promoted checks when `=1` |
| `ONTOLOGOS_DL_MAX_WORKERS` | `10` | Concurrent DL workers during scans |
| `ONTOLOGOS_SCAN_THREADS` | `10` | Rayon parallelism for catalog scans |

For final promotion after fixes, use a higher budget:

```bash
ONTOLOGOS_DL_BUDGET_SECS=120 bash benchmarks/scripts/hermit-burndown.sh promote
```

---

## CI vs local vs nightly

| Job | Workflow | Blocks PR? |
|-----|----------|------------|
| Full conformance @ 30s | `ci.yml` | **Yes** |
| Parity phase gate | `check-hermit-parity-phases.sh` | **Yes** |
| 1.0 release gates | `check-1.0-release-gates.sh` | **Yes** |
| Full HermiT suite (long budget) | `conformance-nightly.yml` | No (`continue-on-error`) |
| Tier C HermiT JAR cross-check | `conformance-nightly.yml` (`tier-c-hermit-crosscheck`) | No (nightly only) |
| Ignored tier | `conformance-nightly.yml` | No |

Before opening a PR that touches DL/conformance:

```bash
bash benchmarks/scripts/hermit-burndown.sh test
```

---

## Common mistakes

| Mistake | Why it is wrong | Do instead |
|---------|-----------------|------------|
| Leaving interrupted `cargo test` running | Orphan DL scans skew triage / lock the next run | `hermit-burndown.sh cleanup` before retrying |
| Only running `cargo test -p ontologos-conformance` and assuming parity improved | Misses WG / phase closures | `check-1.0-release-gates.sh` before claiming done |
| Editing `promoted_*_ids.txt` by hand | Lists are scan outputs; typos hide regressions | `hermit-burndown.sh promote` or `resync` |
| Full catalog scan on every iteration | 428 WG cases × DL budget is slow | Default `triage` / `promote` (unpromoted only) |
| Confusing catalog `parity_pct` with semantic pass | 100% catalog ≠ every case passes | Check `hermit_generated` + `wg_failures --all` |
| Skipping `download.sh` | Pizza and other corpora missing | Run once after clone |

---

## Phases (where we are)

See [ROADMAP.md — HermiT parity phases](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md#hermit-parity-phases-path-to-v100-tag).

| Phase | Focus | Status |
|-------|-------|--------|
| 0–7 | Metrics, harness, WG, Tier B/C | Complete |
| 8 | Expressivity v1.5–v1.9 | Complete |
| 9 | Full CI + release gates; publish + tag | **Ready** (publish not yet shipped) |
| 9 | `parity_pct = 100%` → tag **v1.0.0** | Gate |

---

## Getting help

1. `bash benchmarks/scripts/hermit-burndown.sh status` — current numbers
2. [Conformance reference](../reference/conformance.md) — tier A/B overview
3. [tests/hermit/README.md](https://github.com/eddiethedean/ontologos/blob/main/tests/hermit/README.md) — catalog regeneration
4. GitHub issue with `triage` output for the case you are stuck on
