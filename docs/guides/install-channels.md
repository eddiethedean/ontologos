# Install and release channels

Single source of truth for **what you can install today**. Canonical version table: [Release status](../project/release-status.md).

--8<-- "snippets/channel-banner.md"

## Quick decision

| I want to… | Install | Profiles for production |
|------------|---------|-------------------------|
| Embed in Rust (crates.io) | `ontologos-* = "1.1.4"` in `Cargo.toml` | **EL, RL, RDFS, DL, SWRL** |
| Use from Python (PyPI) | `pip install ontologos` | **EL, RL, RDFS, DL, SWRL** |
| Use from Node.js | `npm install ontologos` | Same as Python |
| Use from WASM / browser | `npm install @ontologos/wasm` · [Wasmer](https://wasmer.io/eddiethedean/ontologos) | Same as Python |
| Use from Java / .NET / C/C++ | Build binding crate (see [Bindings overview](bindings-overview.md)) | Same as Python |
| Run the CLI | `cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.4 ontologos-cli` or clone + build | All profiles on tagged release |
| Contribute / conformance | Clone + `./benchmarks/scripts/download.sh` | Full engine set |

**Default recommendation:** pin **`1.1.4`** on all `ontologos-*` crates and bump them together.

## Published channel (v1.1.4)

| Surface | Version | Install |
|---------|---------|---------|
| **crates.io** | 1.1.4 | `cargo add ontologos-core@1.1.4` (+ parser, facade, profile crates as needed) |
| **PyPI** | 1.1.4 | `pip install ontologos` |
| **npm** | 1.1.4 | `npm install ontologos` · `npm install @ontologos/wasm` |
| **Wasmer** | 1.1.4 | [`eddiethedean/ontologos`](https://wasmer.io/eddiethedean/ontologos) (wasm-bindgen module) |
| **docs.rs** | 1.1.4 | Links in [Reference](../reference/facade.md) reflect this channel |
| **Read the Docs** | latest | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) |

**Production-ready on this channel:** RDFS materialization, OWL RL saturation, OWL EL taxonomy, **OWL 2 DL**, **DLSafe SWRL**, profile detection, explanations (EL full), incremental sessions, OWL QL queries.

**Preview only:** `profile="alc"`, `profile="dl-preview"` — see [Preview profiles](preview-profiles.md).

### WebAssembly (npm + Wasmer)

| Piece | Source | Guide |
|-------|--------|-------|
| **npm package** | [`@ontologos/wasm`](https://www.npmjs.com/package/@ontologos/wasm) | [WASM](wasm.md) |
| **Wasmer `.wasm`** | [Wasmer `eddiethedean/ontologos`](https://wasmer.io/eddiethedean/ontologos) | [WASM](wasm.md) |

### Language bindings (source-build)

| Language | Build from | Guide |
|----------|------------|-------|
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
| **npm** | `npm view ontologos version` · `npm view @ontologos/wasm version` |
| **Wasmer** | [wasmer.io/eddiethedean/ontologos](https://wasmer.io/eddiethedean/ontologos) (version on package page) |

Expected: **1.1.4** from registries.

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
cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.4 ontologos-cli
```

## API documentation

| Channel | Rust API docs |
|---------|---------------|
| **Published 1.1.4** | [docs.rs](https://docs.rs/ontologos-core/1.1.4) |
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
