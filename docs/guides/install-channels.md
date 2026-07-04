# Install and release channels

Single source of truth for **what you can install today**.

--8<-- "snippets/channel-banner.md"

## Quick decision

| I want to… | Install | Profiles for production |
|------------|---------|-------------------------|
| Embed in Rust (crates.io) | `ontologos-* = "1.1.0"` in `Cargo.toml` | **EL, RL, RDFS, DL, SWRL** |
| Use from Python (PyPI) | `pip install ontologos` | **EL, RL, RDFS, DL, SWRL** |
| Use from Node.js | Build `crates/ontologos-node` | Same as Python |
| Use from Java / .NET / C/C++ | Build binding crate (see guides) | Same as Python |
| Run the CLI | `cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.0 ontologos-cli` or clone + build | All profiles on tagged release |
| Contribute / conformance | Clone + `./benchmarks/scripts/download.sh` | Full engine set |

**Default recommendation:** pin **`1.1.0`** on all `ontologos-*` crates and bump them together.

## Published channel (1.1.0)

| Surface | Version | Install |
|---------|---------|---------|
| **crates.io** | 1.1.0 | `cargo add ontologos-core@1.1.0` (+ parser, facade, profile crates as needed) |
| **PyPI** | 1.1.0 | `pip install ontologos` |
| **docs.rs** | 1.1.0 | Links in [Reference](../reference/facade.md) reflect this channel |
| **Read the Docs** | latest | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) |

**Production-ready on this channel:** RDFS materialization, OWL RL saturation, OWL EL taxonomy, **OWL 2 DL**, **DLSafe SWRL**, profile detection, explanations (EL full), incremental sessions, OWL QL queries.

**Preview only:** `profile="alc"`, `profile="dl-preview"` — see [Preview profiles](preview-profiles.md).

### Language bindings (source-build)

| Language | Build from | Guide |
|----------|------------|-------|
| **Node.js** | `crates/ontologos-node` | [Node.js](node.md) |
| **WebAssembly** | `crates/ontologos-wasm` | [WASM](wasm.md) |
| **Java** | `crates/ontologos-java` | [Java](java.md) |
| **.NET** | `crates/ontologos-dotnet` | [.NET](dotnet.md) |
| **C/C++** | `crates/ontologos-c` | [C/C++](c-cpp.md) |

Native libraries: `cargo build -p ontologos-jni -p ontologos-dotnet -p ontologos-c --release`

## Build from source

Use git when you need unreleased `main`, the CLI without a tag pin, or conformance benchmarks:

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # optional for Family; required for Pizza/HermiT
cargo build -p ontologos-cli --release
bash scripts/ci-bindings.sh       # optional: Java, .NET, C/C++ smoke
```

## CLI install (not on crates.io)

See the dedicated [CLI installation](../getting-started/cli-install.md) guide.

```bash
# Tagged release (requires Rust 1.88+)
cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.0 ontologos-cli

# Or from a clone
cargo build -p ontologos-cli --release
./target/release/ontologos --help
```

See [CLI reference](../reference/cli.md) and [Troubleshooting](troubleshooting.md#command-not-found-ontologos).

## API documentation

| Channel | Rust API docs |
|---------|---------------|
| **Published 1.1.0** | [docs.rs](https://docs.rs/ontologos-core/1.1.0) |
| **`main` (development)** | `cargo doc --open -p ontologos-facade` from a clone |

## Upgrading

| From | Guide |
|------|-------|
| v1.0.x | [v1.0.x → v1.1.0](../migration/v1.0.x-to-v1.1.0.md) |
| v0.9.x | [v0.9.x → v1.0.0](../migration/v0.9.x-to-v1.0.0.md) |

## Related

- [Release status](../project/release-status.md) — live metrics
- [Profile stability matrix](profile-stability.md) — per-profile production guidance
- [Known limitations](known-limitations.md) — imports, mapping, axiom counts
- [Migration hub](../migration/index.md) — upgrade paths
- [Prerequisites](prerequisites.md) — Rust 1.88+, Python 3.10+, JDK 17+ (Java), .NET 8+ (.NET)
