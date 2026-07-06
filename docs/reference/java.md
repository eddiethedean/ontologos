# Java API Reference

Java bindings via **`ontologos-jni`** (Maven `dev.ontologos:ontologos`). Source-build only — see [Bindings overview](../guides/bindings-overview.md).

Tutorial: [Java guide](../guides/java.md).

## Install

```bash
cargo build -p ontologos-jni --release
cd crates/ontologos-java/java && mvn test
```

Requires **Java 17+**, **Maven 3.9+**, Rust toolchain.

## Profiles

Same strings as Python: `"auto"`, `"rdfs"`, `"rl"`, `"el"`, `"dl"`, `"swrl"`, `"alc"`, `"dl-preview"`. See [Profile stability](../guides/profile-stability.md).

## `Reasoner` (`dev.ontologos.Reasoner`)

Implements `AutoCloseable` — use try-with-resources.

### Constructors / factories

| Method | Description |
|--------|-------------|
| `Reasoner.fromPath(path, profile)` | Load file (lenient) |
| `Reasoner.loadIn(base, path, profile)` | Sandboxed load |
| `new Reasoner(ontology, profile)` | In-memory ontology |
| `Reasoner(ontology, profile, incremental, budgetSecs)` | Full config |

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `classify()` | `String` (JSON) | Taxonomy or materialization report |
| `checkConsistency()` | `String` (JSON) | `{ consistent, complete }` |
| `isConsistent()` | `boolean` | Incomplete → exception |
| `isEntailed(...)` | `boolean` | Subsumption, class, or property assertion |
| `query(String query)` | `String` (JSON) | OWL QL results |
| `explain()` | `String` (JSON) | Proof graph |
| `addSubclassOf(sub, sup)` | `void` | Incremental mutation |
| `removeSubclassOf(sub, sup)` | `void` | Incremental mutation |
| `close()` | `void` | Release native handle |

Parse JSON responses with your preferred library (Jackson, Gson, etc.).

## `OntologyBuilder`

| Method | Description |
|--------|-------------|
| `addClass(String iri)` | Declare class |
| `subclassOf(String sub, String sup)` | Add `SubClassOf` |
| `build()` | Returns `Ontology` |

## `Ontology`

| Method | Description |
|--------|-------------|
| `load(String path)` | Lenient file load |
| `loadIn(String base, String path)` | Sandboxed file load |
| `fromJson(String json)` | JSON snapshot |
| `fromJsonWithLimits(String json, long maxBytes)` | Capped JSON load |
| `getAxiomCount()` | Mapped axiom count |

## Native library path

Tests load from `target/release/libontologos_jni.so` (platform-dependent). Override:

```bash
mvn test -Dontologos.native.path=/absolute/path/to/libontologos_jni.dylib
```

Or set `ONTOLOGOS_REPO_ROOT` when running from an IDE.

## Related

- [Python API](python.md) — canonical parity reference
- [Troubleshooting — bindings](../guides/troubleshooting.md#binding-build-failures)
