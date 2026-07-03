# Reference index

API and CLI reference for OntoLogos. Install channels: [Install and channels](../guides/install-channels.md).

--8<-- "snippets/channel-banner.md"

## By persona

| I need… | Start here |
|---------|------------|
| Pick a crate / entry point | [Choosing an API](../guides/choosing-an-api.md) · [Rust integration contract](../guides/rust-integration-contract.md) |
| Unified classify / consistency | [Facade API](facade.md) · [Facade guide](../guides/facade-api.md) |
| Core data model | [Core API](core.md) |
| Load OWL files | [Parser API](parser.md) |
| Profile detection | [Profile API](profile.md) |
| OWL EL | [EL API](el.md) |
| OWL RL / RDFS | [RL API](rl.md) · [RL rules](rl-rules.md) |
| OWL 2 DL | [DL API](dl.md) |
| SWRL | [SWRL API](swrl.md) |
| Python bindings | [Python API](python.md) · [Python guide](../guides/python.md) |
| CLI | [CLI reference](cli.md) |
| Errors | [Errors](errors.md) |
| Supported OWL constructs | [Supported constructs](supported-constructs.md) |

## Rust crates (docs.rs)

Published API docs: [docs.rs/ontologos-core/1.0.0](https://docs.rs/ontologos-core/1.0.0). Local: `cargo doc --open -p ontologos-facade`.

| Crate | Reference |
|-------|-----------|
| `ontologos-core` | [core.md](core.md) · [docs.rs](https://docs.rs/ontologos-core/1.0.0) |
| `ontologos-parser` | [parser.md](parser.md) |
| `ontologos-profile` | [profile.md](profile.md) · [docs.rs](https://docs.rs/ontologos-profile/1.0.0) |
| `ontologos-facade` | [facade.md](facade.md) |
| `ontologos-rl` | [rl.md](rl.md) · [RL rules](rl-rules.md) |
| `ontologos-el` | [el.md](el.md) |
| `ontologos-dl` | [dl.md](dl.md) |
| `ontologos-ql` | [ql.md](ql.md) · [query.md](query.md) |
| `ontologos-explain` | [explain.md](explain.md) |
| `ontologos-swrl` | [swrl.md](swrl.md) |

## Interop and conformance

| Topic | Page |
|-------|------|
| JSON snapshot v3 | [json-snapshot-v3.md](../json-snapshot-v3.md) |
| JSON snapshot v2 (legacy) | [json-snapshot-v2.md](../json-snapshot-v2.md) |
| Conformance / HermiT parity | [conformance.md](conformance.md) |
| Contract tests | [contract-tests.md](../examples/contract-tests.md) |
| Evaluator scope | [Evaluator scope](../guides/evaluator-scope.md) |
| Taxonomy tolerance | [taxonomy-tolerance.md](taxonomy-tolerance.md) |
| OWL imports | [owl-imports.md](owl-imports.md) |
| Reasonable adapter limits | [reasonable-limits.md](reasonable-limits.md) |

## Related

- [Architecture](../architecture.md)
- [Deployment](../guides/deployment.md)
- [Production integration](../guides/production-integration.md)
- [Troubleshooting](../guides/troubleshooting.md)
