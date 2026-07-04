# OntoLogos .NET bindings

P/Invoke bindings for [OntoLogos](https://github.com/eddiethedean/ontologos), sharing logic with Node/WASM/Java via `ontologos-js`.

## Build

Requires **.NET 8+** and a Rust toolchain.

```bash
# From repository root
cargo build -p ontologos-dotnet --release

# C# package + tests (runs cargo build via MSBuild)
dotnet test crates/ontologos-dotnet/csharp/Ontologos.sln
```

Or use the manual smoke script:

```bash
bash crates/ontologos-dotnet/scripts/smoke.sh
```

Set `ONTOLOGOS_NATIVE_PATH` or `ONTOLOGOS_REPO_ROOT` when loading the native library from a non-default location.

## Usage

```csharp
using Ontologos;

using var builder = new OntologyBuilder();
builder.AddClass("http://example.org/Pizza");
builder.AddClass("http://example.org/Food");
builder.SubclassOf("http://example.org/Pizza", "http://example.org/Food");
using var ontology = builder.Build();
using var reasoner = new Reasoner(ontology, "el");
Console.WriteLine(reasoner.Classify());
```

Load OWL from disk:

```csharp
using var reasoner = Reasoner.FromPath("family.owl", "rl");
Console.WriteLine(reasoner.Classify());
```

## API

Namespace `Ontologos` mirrors the Python/Node/Java bindings:

| Type | Purpose |
|------|---------|
| `Ontology` | In-memory ontology; `FromJson`, `FromBytes`, `Load`, `LoadIn` |
| `OntologyBuilder` | Fluent TBox/ABox construction |
| `Reasoner` | `Classify`, `CheckConsistency`, `IsEntailed`, `Query`, incremental edits |
| `EntailmentCheck` | Input for entailment checks |

Complex results return **JSON strings** for parsing with `System.Text.Json`.

## Security

| API | Use when |
|-----|----------|
| `Ontology.LoadIn(base, path)` | User uploads (strict, sandboxed) |
| `Ontology.Load(path)` | Trusted local files only |
| `Ontology.FromBytes` / `FromText` | Untrusted in-memory input (strict) |
| `FromJsonWithLimits` | Untrusted JSON snapshots |

See [.NET guide](https://ontologos.readthedocs.io/en/latest/guides/dotnet.html) and [Security](https://ontologos.readthedocs.io/en/latest/security.html).

## Native library name

| Platform | File |
|----------|------|
| macOS | `libontologos_dotnet.dylib` |
| Linux | `libontologos_dotnet.so` |
| Windows | `ontologos_dotnet.dll` |

Load with `NativeLibrary.Load` after placing the library on the loader search path, or set `ONTOLOGOS_NATIVE_PATH`.
