# Profile stability matrix

Single source of truth for `--profile` / `profile=` behavior. See [Release status](../project/release-status.md) for crates.io vs `main` channel details.

!!! info "Production OWL DL"
    **Published v0.9.0:** use Protégé + HermiT or Konclude for production OWL DL. **`main` (1.0.0 workspace):** gated HermiT conformance is green @ 30s (`parity_pct = 100%` catalog; **450+428** active tests); validate on your corpus before replacing HermiT. See [FAQ](../../FAQ.md).

| Profile | User-facing status | Engine | HermiT parity | Production recommendation |
|---------|-------------------|--------|---------------|---------------------------|
| `rdfs` | **Stable** | `ontologos-rdfs` → reasonable | N/A (RDFS) | Yes — embed RDFS materialization |
| `rl` | **Stable** | `ontologos-rl` → reasonable | N/A (RL) | Yes — OWL RL saturation |
| `el` | **Stable** | `ontologos-el` (in-house) | EL-shaped corpora | Yes — OWL EL taxonomy |
| `auto` | **Stable** | Detect → EL, RL, or DL | Depends on ontology | Yes — prefer explicit profile when known |
| `dl` | **Pre-release** | `ontologos-dl` | **100%** catalog; **450+428** gated @ 30s | Gated corpora on `main`; PyPI still 0.9.0 |
| `dl-preview` | **Preview** | `ontologos-dl` (gated) | Same as `dl` + extra checks | No |
| `alc` | **Preview** | `ontologos-alc` | Subset | No |
| `swrl` | **Preview** | `ontologos-swrl` | Minimal | No |
| `ql` (detection only) | **Detection only** | None | N/A | Use ELK or another QL reasoner |

## What “stable” means here

- **Stable:** Suitable for production embedding of that OWL profile *within OntoLogos’s mapped construct subset*. Not a guarantee of HermiT-equivalent DL.
- **Pre-release:** API and engine are active on `main`; behavior changes as parity work lands; may return `ResourceLimit` on hard cases.
- **Preview:** Explicit gating, incomplete rules, or `PreviewLimit` / `NotImplemented` on common paths.

## CLI quick reference

```bash
ontologos materialize ontology.owl              # explicit RDFS (stable)
ontologos classify --profile rl family.owl    # stable
ontologos classify --profile el pizza.owl       # stable (after download.sh for Pizza)
ontologos classify --profile auto ontology.owl  # stable routing; may hit DL pre-release
ontologos classify --profile dl ontology.owl    # pre-release — not HermiT parity
```

## Related

- [Preview profiles](preview-profiles.md) — limitations and error types for DL/ALC/SWRL
- [Comparison](../comparison.md) — vs ELK, HermiT, reasonable
- [Choosing an API](choosing-an-api.md) — Rust entry points
- [Evaluator playbook](evaluator-playbook.md) — 30-minute evaluation
