# Preview profiles (ALC and dl-preview)

Limitations for **preview** profiles only. Stable profiles (EL, RL, RDFS, `dl`, SWRL on `main`) are documented in the [Profile stability matrix](profile-stability.md).

!!! warning "Preview only (`alc`, `dl-preview`)"
    Preview engines may return `PreviewLimit` or `ResourceLimit`. For production OWL DL, use `--profile dl` (stable on v1.0.0). See [Profile stability matrix](profile-stability.md).

## Profile summary (preview)

| Profile | Status | Engine | Typical output |
|---------|--------|--------|----------------|
| `dl-preview` | Preview | `ontologos-dl` (gated) | Taxonomy + explicit preview checks |
| `alc` | Preview | `ontologos-alc` | Taxonomy (tableau-lite) |

Stable `dl` (same engine, no extra gating): [Profile stability matrix](profile-stability.md).

## CLI

```bash
# Explicit preview mode (extra gating checks)
ontologos classify --profile dl-preview benchmarks/data/family.owl

# ALC tableau-lite
ontologos classify --profile alc ontology.owl
```

For stable DL on v1.0.0:

```bash
ontologos classify --profile dl ontology.owl
```

## Python

```python
from ontologos import Reasoner

# Preview only
Reasoner(path="ontology.owl", profile="dl-preview").classify()
Reasoner(path="ontology.owl", profile="alc").classify()

# Production OWL DL
Reasoner(path="ontology.owl", profile="dl").classify()
```

See [Install and channels](install-channels.md) and [Profile stability matrix](profile-stability.md).

## Rust

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{self, ClassifyOutcome};

let mut reasoner = Reasoner::builder()
    .profile(Profile::DlPreview)  // preview gating; use Profile::Dl for stable production path
    .build(ontology)?;
match ontologos_facade::classify(&mut reasoner)? {
    ClassifyOutcome::Taxonomy(t) => {
        println!("subsumptions: {}", t.subsumption_count());
    }
    ClassifyOutcome::Rdfs(r) => println!("inferred: {}", r.inferred_total()),
    ClassifyOutcome::Rl(r) => println!("inferred: {}", r.inferred_total()),
}
```

See [Facade API](facade-api.md) and [Classify quick start](../getting-started/classify-quickstart.md).

## Known limitations

| Limitation | Affected profiles | Symptom |
|------------|-------------------|---------|
| Incomplete tableau rules | `alc`, `dl-preview` | HasValue, HasKey, some cardinalities ignored |
| Expansion budget | `alc`, `dl-preview` | `ResourceLimit` after 4096 expansions |
| Entailment cap | `alc`, `dl`, `dl-preview` | Pairwise subsumption skipped when >192 named classes (`MAX_CLASSES_FOR_ENTAILMENT_INFER`) |
| Preview construct gate | `dl-preview` | `PreviewLimit` when EL-forbidden constructs present |
| Partial OWL mapping | All | Skipped axioms in `parse_meta.warnings` |

Full construct list: [Supported constructs](../reference/supported-constructs.md).

## Error types

| Error | Meaning | Action |
|-------|---------|--------|
| `PreviewLimit` | Construct or feature not in preview scope | Use stable profile — see [Profile stability](profile-stability.md) |
| `ResourceLimit` | Tableau expansion budget exhausted | Simplify ontology or increase DL budget |
| `Profile` / `WrongProfile` | Profile mismatch | Check `ontologos profile` output |

See [Error reference](../reference/errors.md) and [Troubleshooting](troubleshooting.md).

## Related

- [Profile stability matrix](profile-stability.md) — canonical status for all profiles
- [Facade API](facade-api.md)
- [Evaluator playbook](evaluator-playbook.md)
