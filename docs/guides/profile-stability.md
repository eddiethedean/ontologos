# Profile stability matrix

**Canonical source** for `--profile` / `profile=` behavior. Other pages link here — do not duplicate stability labels inline.

See [Install and channels](install-channels.md) and [Release status](../project/release-status.md) for crates.io vs `main` details.

## By install channel

| Profile | PyPI / crates.io **0.9.0** | `main` workspace **1.0.0** |
|---------|------------------------------|----------------------------|
| `rdfs`, `rl`, `el`, `auto` | **Stable** — production-supported | **Stable** |
| `dl` | **Not production-supported** (may error or differ) | **Stable** (gated HermiT catalog @ 30s) |
| `swrl` | **Not available** | **Stable** (DLSafe subset) |
| `dl-preview`, `alc` | **Preview** (errors common) | **Preview** |
| `ql` (detection only) | Detection only | Detection only |

!!! info "OWL DL on PyPI"
    **`profile="dl"` on PyPI 0.9.0** is not HermiT parity. Build from `main` or wait for **v1.0.0** publish. See [release checklist](../project/release-1.0-checklist.md).

## Full matrix

| Profile | User-facing status | Engine | HermiT parity | Production recommendation |
|---------|-------------------|--------|---------------|---------------------------|
| `rdfs` | **Stable** | `ontologos-rl` (`rdfs` module) → reasonable | N/A (RDFS) | Yes on 0.9.0 — embed RDFS materialization |
| `rl` | **Stable** | `ontologos-rl` → reasonable | N/A (RL) | Yes on 0.9.0 — OWL RL saturation |
| `el` | **Stable** | `ontologos-el` (in-house) | EL-shaped corpora | Yes on 0.9.0 — OWL EL taxonomy |
| `auto` | **Stable** | Detect → EL, RL, or DL | Depends on ontology | Yes on 0.9.0 — prefer explicit profile when known |
| `dl` | **Stable on `main`**; **not on PyPI 0.9.0** | `ontologos-dl` | **100% in-scope catalog** — [Evaluator scope](evaluator-scope.md) | Build from `main`; `ontologos-dl = "1.0.0"` after tag |
| `dl-preview` | **Preview** | `ontologos-dl` (gated) | Same engine as `dl` + extra checks | No |
| `alc` | **Preview** | `ontologos-alc` | Subset | No |
| `swrl` | **Stable on `main`**; **not on PyPI 0.9.0** | `ontologos-swrl` | 24/24 RulesTest @ Tier A | Build from `main` for DLSafe SWRL |
| `ql` (detection only) | **Detection only** | None | N/A | Use ELK or another QL reasoner |

## What “stable” means here

- **Pre-release:** Reserved for profiles not yet at the in-scope gate. **`dl` is stable in the 1.0.0 workspace** pending crates.io publish.
- **Stable:** Suitable for production embedding within OntoLogos’s mapped construct subset. **`dl`** passes the in-scope HermiT gate on `main`; validate on your corpus — see [Evaluator scope](evaluator-scope.md).
- **Preview:** Explicit gating, incomplete rules, or `PreviewLimit` / `NotImplemented` on common paths.

### What “100% HermiT parity” means for `dl`

`parity_pct = 100%` counts only **889 in-scope** catalog cases (461 Java + 428 WG), not all 1019 HermiT-derived entries. **130 Java cases** are documented out of scope (`internal`, `excluded`, `migrated`). Tier C taxonomy checks allow OntoLogos to be a **sound superset** of HermiT, not identical output.

For the full breakdown, see [Evaluator scope](evaluator-scope.md).

## CLI quick reference

```bash
ontologos materialize ontology.owl              # explicit RDFS (stable)
ontologos classify --profile rl family.owl    # stable
ontologos classify --profile el pizza.owl       # stable (after download.sh for Pizza)
ontologos classify --profile auto ontology.owl  # stable routing; may hit DL on main
ontologos classify --profile dl ontology.owl    # stable on main / v1.0.0 — gated catalog parity
```

## Related

- [Evaluator scope](evaluator-scope.md) — what 100% does and does not mean
- [Preview profiles](preview-profiles.md) — ALC and `dl-preview` limitations
- [Comparison](../comparison.md) — vs ELK, HermiT, reasonable
- [Choosing an API](choosing-an-api.md) — Rust entry points
- [Evaluator playbook](evaluator-playbook.md) — 30-minute evaluation
