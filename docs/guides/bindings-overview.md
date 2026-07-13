# Bindings overview

Honest matrix for **published vs source-build** language bindings. Install channels: [Install and channels](install-channels.md). API parity: each binding mirrors the Python surface via `ontologos-js`.

## Quick decision

| I want to… | Install today | Registry |
|------------|---------------|----------|
| Embed in **Rust** | `ontologos-* = "1.1.3"` on [crates.io](https://crates.io/crates/ontologos-core) | **Published** |
| Use from **Python** | `pip install ontologos` | **Published** (PyPI) |
| Use from **Node.js** | Build `crates/ontologos-node` | Source-build |
| Use from **Java** | Build `crates/ontologos-java` | Source-build (Maven local) |
| Use from **.NET** | Build `crates/ontologos-dotnet` | Source-build |
| Use from **C/C++** | Build `crates/ontologos-c` | Source-build |
| Use in **browser (WASM)** | Build `crates/ontologos-wasm` | Source-build |

**v1.1.3** adds shared FFI and native bindings. Build bindings from a clone at tag **`v1.1.3`** or from `main`. See [Release status](../project/release-status.md).

## API parity

All bindings expose the same core types:

| Type | Python | Node | Java | .NET | C/C++ | WASM |
|------|--------|------|------|------|-------|------|
| `OntologyBuilder` | Yes | Yes | Yes | Yes | Yes | Yes |
| `Ontology` | Yes | Yes | Yes | Yes | Yes | Yes |
| `Reasoner` | Yes | Yes | Yes | Yes | Yes | Yes |
| `classify()` | Yes | Yes | Yes | Yes | Yes | Yes |
| `explain()` | Yes | Yes | Yes | Yes | Partial | Yes |
| Incremental edits | Yes | Yes | Yes | Yes | Yes | Yes |
| File load | Yes | Yes | Yes | Yes | Yes | No (bytes/JSON only) |

Reference pages: [Python](../reference/python.md) · [Node](../reference/node.md) · [Java](../reference/java.md) · [.NET](../reference/dotnet.md) · [C/C++](../reference/c.md) · [WASM](../reference/wasm.md)

Tutorial guides: [Python](python.md) · [Node](node.md) · [Java](java.md) · [.NET](dotnet.md) · [C/C++](c-cpp.md) · [WASM](wasm.md)

## End-to-end quick start (Node.js)

From a clone with `family.owl` in the working directory:

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos/crates/ontologos-node
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
npm install && npm run build
node -e "const {Reasoner}=require('ontologos'); console.log(Reasoner.fromPath('family.owl','rl').classify())"
```

## End-to-end quick start (Java)

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
cargo build -p ontologos-jni --release
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
cd crates/ontologos-java/java
mvn test -Dtest=SmokeTest
```

See [Java guide](java.md) for `Reasoner.fromPath("family.owl", "rl")`.

## Production profiles

On **v1.1.3** (PyPI / crates.io): `rdfs`, `rl`, `el`, `auto`, `dl`, `swrl` are production-supported. Preview: `alc`, `dl-preview`.

Full matrix: [Profile stability](profile-stability.md).

## Build troubleshooting

| Symptom | Fix |
|---------|-----|
| Java: `UnsatisfiedLinkError` | Set `java.library.path` to `target/release` or `-Dontologos.native.path=…` — see [Java guide](java.md) |
| Node: native addon rebuild after Rust change | `npm run build` in `crates/ontologos-node` |
| WASM: bundler cannot load `.wasm` | Follow wasm-bindgen integration for your bundler — see [WASM guide](wasm.md) |
| .NET: `DllNotFoundException` | Build `ontologos-dotnet` release cdylib first — see [.NET guide](dotnet.md) |
| C/C++: linker cannot find `libontologos_c` | `cargo build -p ontologos-c --release` — see [C/C++ guide](c-cpp.md) |

Full list: [Troubleshooting — bindings](troubleshooting.md#binding-build-failures).

## Related

- [Install and channels](install-channels.md)
- [Prerequisites](prerequisites.md)
- [Security](../security.md)
- [Production integration](production-integration.md)
