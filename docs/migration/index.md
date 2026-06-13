# Upgrade to the latest release

Quick paths for upgrading to **v0.9.0**. For step-by-step history, follow the migration chain in the nav.

## Jump here

| From | Guide |
|------|-------|
| **v0.8.x (most users)** | [v0.8.x → v0.9.0](v0.8.x-to-v0.9.0.md) — Python ecosystem; Rust version bump only |
| **v0.7.x** | [v0.7.x → v0.8.0](v0.7.x-to-v0.8.0.md) then [v0.8.x → v0.9.0](v0.8.x-to-v0.9.0.md) |
| **v0.5.x or older** | Chain guides from your version through the list below |

## v0.9.0 at a glance

**Rust:** Bump all `ontologos-*` crate pins to `"0.9.0"` in `Cargo.toml`. No API changes from v0.8.0.

**Python:** `pip install -U ontologos`. New: `Ontology`, `OntologyBuilder`, `explain()`, incremental mutations, optional pandas/polars export. See [Python guide](../guides/python.md).

**CLI:** Unchanged commands; `classify --profile auto|el|rl|rdfs`, `materialize`, `explain`.

## Full migration chain

1. [v0.1 → v0.2](v0.1-to-v0.2.md)
2. [v0.2 → v0.3](v0.2-to-v0.3.md)
3. [v0.3.0 → v0.3.1](v0.3.0-to-v0.3.1.md)
4. [v0.3.x → v0.4.0](v0.3.x-to-v0.4.0.md)
5. [v0.4.x → v0.5.0](v0.4.x-to-v0.5.0.md) — **breaking:** CLI `classify` semantics
6. [v0.5.x → v0.6.0](v0.5.x-to-v0.6.0.md)
7. [v0.6.x → v0.7.0](v0.6.x-to-v0.7.0.md)
8. [v0.7.x → v0.8.0](v0.7.x-to-v0.8.0.md) — incremental reasoning
9. [v0.8.x → v0.9.0](v0.8.x-to-v0.9.0.md) — Python maturity

## Related

- [CHANGELOG](../project/changelog.md)
- [Release notes](../project/release-notes.md)
- [Roadmap summary](../project/roadmap-summary.md)
