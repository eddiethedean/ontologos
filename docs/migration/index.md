# Upgrade to the latest release

--8<-- "snippets/channel-banner.md"

**Latest release (crates.io / PyPI):** **v1.1.0**

| Your situation | Guide |
|----------------|-------|
| Fresh install | Pin `ontologos-* = "1.1.0"` or `pip install ontologos==1.1.0` |
| Upgrading from v1.0.x | [v1.0.x → v1.1.0](v1.0.x-to-v1.1.0.md) |
| Upgrading from v0.9.x | [v0.9.x → v1.0.0](v0.9.x-to-v1.0.0.md) |
| Upgrading from v0.8.x | [v0.8.x → v1.0.0](v0.8.x-to-v1.0.0.md) |
| Older releases | [Historical migrations](historical.md) |

## v1.1.0 at a glance

**Rust:** Bump all `ontologos-*` crate pins to `"1.1.0"`. No breaking API changes for facade/parser users.

**Python:** `pip install -U ontologos`. Same `Reasoner` / `Ontology` API.

**New bindings:** Java (JNI), .NET (P/Invoke), C/C++ (cdylib), Node (N-API), WASM — see [Choosing an API](../guides/choosing-an-api.md).

**CLI:** `classify --profile auto|el|rl|rdfs|dl|swrl`, `materialize`, `explain`, `query`.

## v1.0.0 at a glance (prior major)

**Breaking changes from 0.9.x:** JSON writers emit v3; shim crates removed (`ontologos-rdfs` → `ontologos-rl`, `ontologos-query` → `ontologos-ql`). See [v0.9.x → v1.0.0](v0.9.x-to-v1.0.0.md).

## Historical migrations

Older step-by-step guides: [Historical migrations](historical.md).

## Related

- [CHANGELOG](../project/changelog.md)
- [Release notes](../project/release-notes.md)
- [Profile stability matrix](../guides/profile-stability.md)
