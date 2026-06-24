# Upgrade to the latest release

**Published today (crates.io / PyPI):** v0.9.0  
**In development on `main`:** 1.0.0 (HermiT parity in progress)

| Your situation | Guide |
|----------------|-------|
| Using v0.9.0 in production | Stay on `0.9.0` pins; read [Release status](../project/release-status.md) |
| Upgrading from v0.8.x | [v0.8.x → v0.9.0](v0.8.x-to-v0.9.0.md) then optionally [v0.9.x → v1.0.0](v0.9.x-to-v1.0.0.md) when tagged |
| Tracking `main` / 1.0.0 workspace | [v0.9.x → v1.0.0](v0.9.x-to-v1.0.0.md) |
| Jump from v0.8.x directly | [v0.8.x → v1.0.0](v0.8.x-to-v1.0.0.md) |

## v0.9.0 at a glance (published)

**Rust:** Bump all `ontologos-*` crate pins to `"0.9.0"` in `Cargo.toml`. No API changes from v0.8.0.

**Python:** `pip install -U ontologos`. New: `Ontology`, `OntologyBuilder`, `explain()`, incremental mutations, optional pandas/polars export. See [Python guide](../guides/python.md).

**CLI:** Unchanged commands; `classify --profile auto|el|rl|rdfs`, `materialize`, `explain`.

## v1.0.0 (when tagged)

The **v1.0.0** tag ships when [ROADMAP Phase 9](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md#phase-9--v100-tag-100-in-scope-parity) completes (100% in-scope HermiT catalog parity). Until then, `main` uses workspace **1.0.0** semver without a matching crates.io/PyPI release.

## Historical migrations

Older step-by-step guides: [Historical migrations](historical.md).

## Related

- [CHANGELOG](../project/changelog.md)
- [Release notes](../project/release-notes.md)
- [Profile stability matrix](../guides/profile-stability.md)
