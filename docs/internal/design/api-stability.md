# API stability and deprecation policy (v1.0)

**Status:** Adopted for the v1.0.0 release.

## Semver contract

OntoLogos follows [Semantic Versioning 2.0](https://semver.org/):

| Bump | When |
|------|------|
| **MAJOR** | Breaking public API or behavior change for stable profiles |
| **MINOR** | Additive API, preview promotions, new profiles |
| **PATCH** | Bug fixes, docs, performance without API change |

**Stable profiles** (EL, RL, RDFS, DL on v1.0): classification and consistency results must not change without a major bump or documented waiver.

**Preview profiles** (ALC, SWRL, `dl-preview`): may change behavior in minor releases until promoted.

## Preferred entry points

| Task | Preferred API | Avoid |
|------|---------------|-------|
| Multi-profile classify | `ontologos_facade::classify` | `ontologos_core::Reasoner::classify` (deprecated) |
| Classify result type | `ontologos_facade::ClassifyOutcome` (re-export) | Importing from `ontologos-el` when using facade only |
| ALC tableau types | `ontologos_alc::*` | `ontologos_dl::*` ALC re-exports (deprecated since 1.0.0) |
| File load | `ontologos_parser::load_ontology` | `Ontology::from_file` (returns `ParseNotAvailable`) |

## Deprecation process

1. Mark with `#[deprecated(since = "X.Y.Z", note = "...")]` and document the replacement.
2. Keep deprecated items for **at least one minor release** (target removal in 2.0 for items deprecated in 1.0).
3. Update user-facing docs (`facade-api.md`, `choosing-an-api.md`, FAQ) in the same release.
4. CI may use `#[allow(deprecated)]` in tests that intentionally exercise legacy stubs.

## Items deprecated in 1.0.0

| Item | Replacement |
|------|-------------|
| `Reasoner::classify()` | `ontologos_facade::classify` or profile crate helpers |
| `ontologos_dl::{Clause, ClauseSet, DlOntology, TableauSeed, …}` | `ontologos_alc` equivalents |

## Non-breaking additive changes (1.0.x)

The DIP engine registry refactor (v1.0.0) added without breaking existing public APIs:

- `ontologos_core::{EngineKind, ResolvedRoute, …}`
- `ontologos_profile::resolve_route`
- `*Engine` adapter structs in profile crates
- `ontologos_facade::ClassifyOutcome` re-export

## Related

- [Facade API](../guides/facade-api.md)
- [Choosing an API](../guides/choosing-an-api.md)
- [ALC vs DL boundary](alc-boundary.md)
- [Dependency-first ADR](dependency-first.md)
