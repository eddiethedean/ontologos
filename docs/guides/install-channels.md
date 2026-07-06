# Install and release channels

Single source of truth for **what you can install today**. Canonical version table: [Release status](../project/release-status.md).

--8<-- "snippets/channel-banner.md"

## Quick decision

| I want to… | Install | Profiles for production |
|------------|---------|-------------------------|
| Embed in Rust (crates.io) | `ontologos-* = "1.1.1"` in `Cargo.toml` | **EL, RL, RDFS, DL, SWRL** |
| Use from Python (PyPI) | `pip install ontologos` | **EL, RL, RDFS, DL, SWRL** |
| Use from Node.js | Build `crates/ontologos-node` | Same as Python |
| Use from Java / .NET / C/C++ | Build binding crate (see [Bindings overview](bindings-overview.md)) | Same as Python |
| Run the CLI | `cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.1 ontologos-cli` or clone + build | All profiles on tagged release |
| Contribute / conformance | Clone + `./benchmarks/scripts/download.sh` | Full engine set |

**Default recommendation:** pin **`1.1.1`** on all `ontologos-*` crates and bump them together.

## Published channel (v1.1.1)

| Surface | Version | Install |
|---------|---------|---------|
| **crates.io** | 1.1.1 | `cargo add ontologos-core@1.1.1` (+ parser, facade, profile crates as needed) |
| **PyPI** | 1.1.1 | `pip install ontologos` |
| **docs.rs** | 1.1.1 | Links in [Reference](../reference/facade.md) reflect this channel |
| **Read the Docs** | latest | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) |

**Production-ready on this channel:** RDFS materialization, OWL RL saturation, OWL EL taxonomy, **OWL 2 DL**, **DLSafe SWRL**, profile detection, explanations (EL full), incremental sessions, OWL QL queries.

**Preview only:** `profile="alc"`, `profile="dl-preview"` — see [Preview profiles](preview-profiles.md).

### Language bindings (source-build)

| Language | Build from | Guide |
|----------|------------|-------|
| **Node.js** | `crates/ontologos-node` | [Node.js](node.md) · [Bindings overview](bindings-overview.md) |
| **WebAssembly** | `crates/ontologos-wasm` | [WASM](wasm.md) |
| **Java** | `crates/ontologos-java` | [Java](java.md) |
| **.NET** | `crates/ontologos-dotnet` | [.NET](dotnet.md) |
| **C/C++** | `crates/ontologos-c` | [C/C++](c-cpp.md) |

Native libraries: `cargo build -p ontologos-jni -p ontologos-dotnet -p ontologos-c --release`

## What version am I running?

| Surface | Command |
|---------|---------|
| **Python** | `python -c "import ontologos; print(ontologos.__version__)"` |
| **CLI** | `ontologos --version` |
| **Rust dependency** | `cargo tree -p ontologos-core \| head -1` |

Expected: **1.1.1** from registries.

## Build from source

Use git when you need unreleased `main`, the CLI without a tag pin, or conformance benchmarks:

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh
cargo build -p ontologos-cli --release
bash scripts/ci-bindings.sh
```

## CLI install (not on crates.io)

See [CLI installation](../getting-started/cli-install.md).

```bash
cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.1 ontologos-cli
```

## API documentation

| Channel | Rust API docs |
|---------|---------------|
| **Published 1.1.1** | [docs.rs](https://docs.rs/ontologos-core/1.1.1) |
| **`main` (development)** | `cargo doc --open -p ontologos-facade` from a clone |

## Upgrading

| From | Guide |
|------|-------|
| v1.0.x | [v1.0.x → v1.1.0](../migration/v1.0.x-to-v1.1.0.md) |
| v0.9.x | [v0.9.x → v1.0.0](../migration/v0.9.x-to-v1.0.0.md) |

## Related

- [Release status](../project/release-status.md)
- [Profile stability matrix](profile-stability.md)
- [Known limitations](known-limitations.md)
- [Before you integrate](before-you-integrate.md)
- [Migration hub](../migration/index.md)
- [Prerequisites](prerequisites.md)
