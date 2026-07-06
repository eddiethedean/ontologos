# Upgrade to the latest release

--8<-- "snippets/channel-banner.md"

**Latest release (crates.io / PyPI):** **v1.1.1**

| Your situation | Guide |
|----------------|-------|
| Fresh install | Pin `ontologos-* = "1.1.1"` or `pip install ontologos==1.1.1` |
| Upgrading from v1.0.x | [v1.0.x → v1.1.0](v1.0.x-to-v1.1.0.md) |
| Upgrading from v0.9.x | [v0.9.x → v1.0.0](v0.9.x-to-v1.0.0.md) |
| Upgrading from v0.8.x | [v0.8.x → v1.0.0](v0.8.x-to-v1.0.0.md) |
| Older releases | [Historical migrations](historical.md) |

## v1.1.1 at a glance

**Rust:** Bump all `ontologos-*` crate pins to `"1.1.1"`. Drop-in patch over v1.1.0 — parser, DL, SWRL, RL, and binding fixes only.

**Python:** `pip install -U ontologos` or `pip install ontologos==1.1.1`. Same `Reasoner` / `Ontology` API.

**CLI:** `classify --profile auto|el|rl|rdfs|dl|swrl`, `materialize`, `explain`, `query`.

## v1.1.0 at a glance

**Rust:** Bump all `ontologos-*` crate pins to `"1.1.0"`. No breaking API changes for facade/parser users.

**Python:** `pip install -U ontologos`. Same `Reasoner` / `Ontology` API.

**New bindings:** Java (JNI), .NET (P/Invoke), C/C++ (cdylib), Node (N-API), WASM — see [Bindings overview](../guides/bindings-overview.md).

**CLI:** `classify --profile auto|el|rl|rdfs|dl|swrl`, `materialize`, `explain`, `query`.

## v1.0.0 at a glance (prior release)

HermiT parity milestone — OWL 2 DL + SWRL stable. See [v0.9.x → v1.0.0](v0.9.x-to-v1.0.0.md) for breaking changes from 0.9.x.

## Historical migrations

Older step-by-step guides: [Historical migrations](historical.md).

## Related

- [CHANGELOG](../project/changelog.md)
- [Release notes](../project/release-notes.md)
- [Profile stability matrix](../guides/profile-stability.md)
