# .NET API Reference

.NET bindings via **`ontologos-dotnet`** (namespace `Ontologos`). Source-build only — see [Bindings overview](../guides/bindings-overview.md).

Tutorial: [.NET guide](../guides/dotnet.md).

## Install

```bash
cargo build -p ontologos-dotnet --release
cd crates/ontologos-dotnet && dotnet test
```

Requires **.NET 8+** and Rust toolchain.

## Profiles

Same strings as Python: `"auto"`, `"rdfs"`, `"rl"`, `"el"`, `"dl"`, `"swrl"`, `"alc"`, `"dl-preview"`. See [Profile stability](../guides/profile-stability.md).

## `Reasoner`

Implements `IDisposable`.

### Constructors

| Constructor | Description |
|-------------|-------------|
| `Reasoner.FromPath(path, profile)` | Load file (lenient) |
| `Reasoner.LoadIn(base, path, profile)` | Sandboxed load |
| `Reasoner(ontology, profile)` | In-memory ontology |
| `Reasoner(ontology, profile, incremental, budgetSecs)` | Full config |

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `Classify()` | `string` (JSON) | Taxonomy or materialization report |
| `CheckConsistency()` | `string` (JSON) | `{ consistent, complete }` |
| `IsConsistent()` | `bool` | Incomplete → exception |
| `IsEntailed(...)` | `bool` | Subsumption, class, or property assertion |
| `Query(string query)` | `string` (JSON) | OWL QL results |
| `Explain()` | `string` (JSON) | Proof graph |
| `AddSubclassOf(sub, sup)` | `void` | Incremental mutation |
| `RemoveSubclassOf(sub, sup)` | `void` | Incremental mutation |
| `Dispose()` | `void` | Release native handle |

## `OntologyBuilder`

| Method | Description |
|--------|-------------|
| `AddClass(string iri)` | Declare class |
| `SubclassOf(string sub, string sup)` | Add `SubClassOf` |
| `Build()` | Returns `Ontology` |

## `Ontology`

| Method | Description |
|--------|-------------|
| `Load(string path)` | Lenient file load |
| `LoadIn(string base, string path)` | Sandboxed load |
| `FromJson(string json)` | JSON snapshot |
| `FromJsonWithLimits(string json, ulong maxBytes)` | Capped JSON load |
| `AxiomCount` | Mapped axiom count (property) |

## Native library

P/Invoke loads `libontologos_dotnet` from `target/release`. `DllImportResolver` searches next to the managed assembly and `target/release`.

## Related

- [Python API](python.md) — canonical parity reference
- [Troubleshooting — bindings](../guides/troubleshooting.md#binding-build-failures)
