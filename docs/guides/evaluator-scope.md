# Evaluator scope and parity metrics

What HermiT parity claims mean — and what they do **not** mean. Canonical numbers live in [Release status](../project/release-status.md); verify live with:

```bash
bash benchmarks/scripts/hermit-burndown.sh status
```

## Two metrics — do not conflate them

| Metric | Meaning | Current on `main` |
|--------|---------|-------------------|
| **`parity_pct`** | In-scope catalog harness complete (zero `planned` Java + WG cases) | **100%** |
| **`true_parity_pct`** | Composite everyday HermiT equivalence (minimum of sub-metrics below) | **100%** |

Both gates are **blocking in CI** on `main`. They apply to the **gated conformance corpora**, not every real-world ontology.

### `parity_pct` formula

```text
in_scope_total = (591 Java − internal − excluded − migrated) + 428 WG = 889
parity_pct = 100% when java_planned = 0 and wg_planned = 0
```

**889 in-scope cases** are not all 1019 HermiT-derived catalog entries. **130 Java cases** are documented out of scope (`internal`, `excluded`, `migrated`). Tier C taxonomy checks allow OntoLogos to be a **sound superset** of HermiT, not identical output.

### `true_parity_pct` sub-metrics

Composite minimum of:

| Sub-metric | What it measures |
|------------|------------------|
| Literal catalog green | Active harness tests pass @ 30s |
| Strict taxonomy | Tier C **sound superset** taxonomy vs HermiT JAR (PR gate allows extras) |
| Strict taxonomy (identity) | `compare-tier-c-strict-family.sh` with `--max-extra 0` — **informational only** (waived: 26 extra edges on `family.owl` vs HermiT) |
| Internal ports | Hand-written HermiT ports (RL, RDFS, EL) |
| SWRL rules | DLSafe rule execution coverage |
| Perf gate | Family DL wall-clock budget |

## Runnable conformance @ 30s (blocking CI)

| Suite | Count |
|-------|------:|
| Java axiom tests | **450** |
| OWL WG tests | **428** |
| Active conformance tests total | **1048** / **1049** defined (**1** hand-written `#[ignore]`; **0** in generated catalogs) |

## What 100% does **not** guarantee

- Parity on ontologies **outside** the gated catalog
- Identical taxonomy output to HermiT on every corpus (Tier C allows sound superset)
- Production readiness on arbitrary real-world ontologies — validate your corpus
- Full SWRL beyond DLSafe subset
- Interactive editing (Protégé replacement)

## Tier overview

| Tier | Role | Blocking? |
|------|------|-----------|
| **A** | HermiT catalog + WG harness (`ontologos-conformance`) | Yes (PR CI) |
| **B** | Classification fixture comparison | Yes |
| **C** | HermiT JAR taxonomy cross-check (nightly + PR) | PR: sound superset (`--max-missing 0`); strict identity (`--max-extra 0`) **informational** |
| **Contract** | Public facade API (`ontologos-contract`) | Yes (every PR) |

## Channel availability for evaluators

| Channel | DL evaluation | Command |
|---------|---------------|---------|
| PyPI / crates.io **1.1.0** | Full DL + SWRL | `pip install ontologos==1.1.0` or `ontologos-* = "1.1.0"` |
| CLI (git) | Full profiles | `cargo install --git … --tag v1.1.0 ontologos-cli` |

See [Evaluator playbook](evaluator-playbook.md) and [Comparison](../comparison.md).

## Related

- [Release status](../project/release-status.md) — canonical version and metric table
- [Profile stability](profile-stability.md)
- [Conformance reference](../reference/conformance.md)
- [HermiT burndown](hermit-burndown.md) — contributors
