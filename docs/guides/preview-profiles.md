# Preview profiles (DL, ALC, SWRL)

OntoLogos **0.9.0** ships stable EL, RL, and RDFS profiles. **ALC**, **DL**, **dl-preview**, and **SWRL** are available for early testing on `main` — not production HermiT replacements.

!!! warning "Preview only"
    Preview engines are incomplete. Use Protégé + HermiT or Konclude for production OWL DL workflows. See [Comparison](../comparison.md) and [Release status](../project/release-status.md).

## Profile summary

| Profile | Status | Engine | Typical output |
|---------|--------|--------|----------------|
| `auto` | Stable | Detect → EL, RL, or DL | Taxonomy or materialization report |
| `el` | Stable | `ontologos-el` | Taxonomy |
| `rl` | Stable | `ontologos-rl` | Materialization report |
| `rdfs` | Stable | `ontologos-rdfs` | Materialization report |
| `dl-preview` | Preview | `ontologos-dl` (gated) | Taxonomy + CLI warning |
| `dl` | Preview | `ontologos-dl` | Taxonomy |
| `alc` | Preview | `ontologos-alc` | Taxonomy |
| `swrl` | Preview | — | Errors (`NotImplemented` / `PreviewLimit`) |

## CLI

```bash
# DL preview (explicit warning)
ontologos classify --profile dl-preview benchmarks/data/family.owl

# Full DL path (same engine, no extra gating)
ontologos classify --profile dl benchmarks/data/family.owl

# ALC tableau-lite
ontologos classify --profile alc ontology.owl
```

`--profile auto` on a DL-detected ontology routes through the DL classifier (hybrid EL + saturation + tableau).

## Python

```python
from ontologos import Reasoner

Reasoner(path="ontology.owl", profile="dl").classify()
Reasoner(path="ontology.owl", profile="dl-preview").classify()  # gated preview mode
Reasoner(path="ontology.owl", profile="alc").classify()
```

Use `profile="swrl"` only to probe error handling — rule execution is not implemented.

## Rust

Prefer the unified facade for multi-profile apps:

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_facade;

let mut reasoner = Reasoner::builder()
    .profile(Profile::Dl)
    .build(ontology)?;
let outcome = ontologos_facade::classify(&mut reasoner)?;
```

See [Facade API](facade-api.md) and [Choosing an API](choosing-an-api.md).

## Known limitations

| Limitation | Affected profiles | Symptom |
|------------|-------------------|---------|
| Incomplete tableau rules | `dl`, `alc` | HasValue, HasKey, some cardinalities ignored |
| Expansion budget | `dl`, `alc` | `ResourceLimit` after 4096 expansions |
| Entailment cap | `dl`, `alc` | Pairwise subsumption skipped when >32 named classes |
| Preview construct gate | `dl-preview` | `PreviewLimit` when EL-forbidden constructs present |
| SWRL not implemented | `swrl` | `NotImplemented` or `PreviewLimit` |
| Partial OWL mapping | All | Skipped axioms in `parse_meta.warnings` |

Full construct list: [Supported constructs](../reference/supported-constructs.md).

## Error types

| Error | Meaning | Action |
|-------|---------|--------|
| `PreviewLimit` | Construct or feature not in preview scope | Use stable profile or wait for 1.0 |
| `ResourceLimit` | Tableau expansion budget exhausted | Simplify ontology or retry later |
| `NotImplemented` (SWRL) | No executable SWRL rules mapped | Use EL/RL/DL profiles instead |
| `Profile` / `WrongProfile` | Profile mismatch | Check `ontologos profile` output |

See [Error reference](../reference/errors.md) and [Troubleshooting](troubleshooting.md).

## Related

- [Facade API](facade-api.md)
- [Architecture](../architecture.md)
- [Evaluator playbook](evaluator-playbook.md)
- [Roadmap summary](../project/roadmap-summary.md) — path to 1.0 HermiT parity
