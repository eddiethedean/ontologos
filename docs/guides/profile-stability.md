# Profile stability matrix

Single source of truth for `--profile` / `profile=` behavior. See [Release status](../project/release-status.md) for crates.io vs `main` channel details.

!!! info "Production OWL DL"
    **Published v0.9.0:** EL/RL/RDFS only on crates.io/PyPI. **`main` / upcoming v1.0.0:** gated HermiT conformance green @ 30s; `ontologos-dl` ready to publish. See [release checklist](../project/release-1.0-checklist.md) and [parity roadmap](../internal/parity-roadmap.md).

| Profile | User-facing status | Engine | HermiT parity | Production recommendation |
|---------|-------------------|--------|---------------|---------------------------|
| `rdfs` | **Stable** | `ontologos-rdfs` → reasonable | N/A (RDFS) | Yes — embed RDFS materialization |
| `rl` | **Stable** | `ontologos-rl` → reasonable | N/A (RL) | Yes — OWL RL saturation |
| `el` | **Stable** | `ontologos-el` (in-house) | EL-shaped corpora | Yes — OWL EL taxonomy |
| `auto` | **Stable** | Detect → EL, RL, or DL | Depends on ontology | Yes — prefer explicit profile when known |
| `dl` | **Stable** (v1.0.0 workspace; publish pending) | `ontologos-dl` | **100% in-scope catalog** gate — see [honest assessment](../internal/hermit-parity-honest-assessment.md) | `ontologos-dl = "1.0.0"` after tag; build from `main` today |
| `dl-preview` | **Preview** | `ontologos-dl` (gated) | Same engine as `dl` + extra checks | No |
| `alc` | **Preview** | `ontologos-alc` | Subset | No |
| `swrl` | **Stable** | `ontologos-swrl` | 24/24 RulesTest @ Tier A | Yes — DLSafe SWRL forward chaining + DL consistency |
| `ql` (detection only) | **Detection only** | None | N/A | Use ELK or another QL reasoner |

## What “stable” means here

- **Pre-release:** Reserved for profiles not yet at the in-scope gate. **`dl` is stable in the 1.0.0 workspace** pending crates.io publish.
- **Stable:** Suitable for production embedding within OntoLogos’s mapped construct subset. **`dl`** passes the in-scope HermiT gate; validate on your corpus for everyday HermiT equivalence — see [honest assessment](../internal/hermit-parity-honest-assessment.md).
- **Preview:** Explicit gating, incomplete rules, or `PreviewLimit` / `NotImplemented` on common paths.

### What “100% HermiT parity” means for `dl`

`parity_pct = 100%` counts only **889 in-scope** catalog cases (461 Java + 428 WG), not all 1019 HermiT-derived entries. **130 Java cases** are documented out of scope (`internal`, `excluded`, `migrated`). Tier C taxonomy checks allow OntoLogos to be a **sound superset** of HermiT, not identical output.

For the full breakdown, see [Brutally honest HermiT parity assessment](../internal/hermit-parity-honest-assessment.md).

## CLI quick reference

```bash
ontologos materialize ontology.owl              # explicit RDFS (stable)
ontologos classify --profile rl family.owl    # stable
ontologos classify --profile el pizza.owl       # stable (after download.sh for Pizza)
ontologos classify --profile auto ontology.owl  # stable routing; may hit DL on main
ontologos classify --profile dl ontology.owl    # stable on main / v1.0.0 — gated catalog parity
```

## Related

- [HermiT parity assessment](../internal/hermit-parity-honest-assessment.md) — what 100% does and does not mean
- [Preview profiles](preview-profiles.md) — limitations and error types for DL/ALC/SWRL
- [Comparison](../comparison.md) — vs ELK, HermiT, reasonable
- [Choosing an API](choosing-an-api.md) — Rust entry points
- [Evaluator playbook](evaluator-playbook.md) — 30-minute evaluation
