# Facade API (`ontologos-facade`)

The **`ontologos-facade`** crate is the unified routing layer for CLI, Python, and multi-profile Rust apps. It avoids circular dependencies between EL and DL engines while exposing one `classify()` entry point.

!!! note "Workspace crate"
    `ontologos-facade` is published on crates.io in the workspace release set. Prefer it over calling `ontologos_el::classify_with_profile` directly when you need **DL**, **ALC**, or **SWRL** routing.

## When to use the facade

| Use facade | Use profile crates directly |
|------------|----------------------------|
| `Profile::Auto` (including DL-detected ontologies) | Single known profile (e.g. only RDFS) |
| `Profile::Dl`, `Alc`, `Swrl` | Embedding reasonable/EL internals |
| CLI/Python parity in Rust tests | Minimal dependency footprint |

## Rust API

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_el::ClassifyOutcome;
use ontologos_facade;
use ontologos_parser::load_ontology;

let ontology = load_ontology(path)?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .build(ontology)?;

match ontologos_facade::classify(&mut reasoner)? {
    ClassifyOutcome::Taxonomy(t) => { /* EL, DL, ALC */ }
    ClassifyOutcome::Rdfs(report) => { /* RDFS materialization */ }
    ClassifyOutcome::Rl(report) => { /* RL saturation */ }
}
```

### Consistency check

```rust
let consistent = ontologos_facade::is_consistent(&reasoner)?;
```

### Profile routing

| `Profile` | Routed to |
|-----------|-----------|
| `Auto` | EL/RL if detected; **DL** if profile detection returns DL |
| `El`, `Rdfs`, `Rl` | `ontologos-el` router (EL/RL/RDFS paths) |
| `Alc` | `ontologos-alc::classify` |
| `Dl` | `ontologos-dl::classify` |
| `Swrl` | `ontologos-swrl::classify_with_swrl` (preview; often errors) |

Preview limitations: [Preview profiles](preview-profiles.md).

## What not to do

```rust
// DON'T — core stub returns NotImplemented or delegate hints
reasoner.classify()?;

// DON'T — EL router alone skips full DL hybrid for Profile::Dl
ontologos_el::classify_with_profile(&mut reasoner)?; // when profile is Dl
```

Use **`ontologos_facade::classify`** or the profile-specific engine crate.

## Dependencies

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
ontologos-facade = "0.9.0"
```

The facade pulls in `ontologos-el`, `ontologos-dl`, `ontologos-alc`, `ontologos-swrl`, `ontologos-rdfs`, and `ontologos-rl` transitively.

## Related

- [Choosing an API](choosing-an-api.md)
- [Preview profiles](preview-profiles.md)
- [Architecture](../architecture.md)
- [docs.rs/ontologos-facade](https://docs.rs/ontologos-facade/0.9.0) (when published)
