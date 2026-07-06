# Profile stability matrix

**Canonical source** for `--profile` / `profile=` behavior. Other pages link here — do not duplicate stability labels inline.

See [Install and channels](install-channels.md) and [Release status](../project/release-status.md).

## Profile stability (published channel)

Current published version: see [Release status](../project/release-status.md) (**1.1.2** on crates.io/PyPI today).

| Profile | Status | Engine | Production recommendation |
|---------|--------|--------|---------------------------|
| `rdfs` | **Stable** | `ontologos-rl` (`rdfs` module) → reasonable | Embed RDFS materialization |
| `rl` | **Stable** | `ontologos-rl` → reasonable | OWL RL saturation |
| `el` | **Stable** | `ontologos-el` (in-house) | OWL EL taxonomy |
| `auto` | **Stable** | Detect → EL, RL, or DL | Prefer explicit profile when known |
| `dl` | **Stable** | `ontologos-dl` | OWL 2 DL — gated HermiT catalog @ 30s |
| `swrl` | **Stable** | `ontologos-swrl` | DLSafe SWRL subset |
| `dl-preview` | **Preview** | `ontologos-dl` (gated) | No — use `dl` |
| `alc` | **Preview** | `ontologos-alc` | No |
| `ql` (detection only) | **Detection only** | None | Use ELK or another QL reasoner |

## What “stable” means here

- **Stable:** Suitable for production embedding within OntoLogos’s mapped construct subset. **`dl`** passes the in-scope HermiT gate; validate on your corpus — see [Evaluator scope](evaluator-scope.md).
- **Preview:** Explicit gating, incomplete rules, or `PreviewLimit` / `NotImplemented` on common paths.

### What “100% HermiT parity” means for `dl`

`parity_pct = 100%` counts only **889 in-scope** catalog cases (461 Java + 428 WG), not all 1019 HermiT-derived entries. **130 Java cases** are documented out of scope (`internal`, `excluded`, `migrated`). Tier C taxonomy checks allow OntoLogos to be a **sound superset** of HermiT, not identical output.

For the full breakdown, see [Evaluator scope](evaluator-scope.md).

## CLI quick reference

```bash
ontologos materialize ontology.owl              # explicit RDFS (stable)
ontologos classify --profile rl family.owl    # stable
ontologos classify --profile el pizza.owl       # stable (after download.sh for Pizza)
ontologos classify --profile auto ontology.owl  # stable routing
ontologos classify --profile dl ontology.owl    # stable — gated catalog parity
ontologos classify --profile swrl ontology.owl  # stable (DLSafe subset)
```

## Related

- [Evaluator scope](evaluator-scope.md) — what 100% does and does not mean
- [Preview profiles](preview-profiles.md) — ALC and `dl-preview` limitations
- [Comparison](../comparison.md) — vs ELK, HermiT, reasonable
- [Choosing an API](choosing-an-api.md) — Rust entry points
- [Evaluator playbook](evaluator-playbook.md) — 30-minute evaluation
