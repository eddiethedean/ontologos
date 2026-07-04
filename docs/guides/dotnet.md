# .NET Guide

.NET bindings ship in the workspace crate **`ontologos-dotnet`** (NuGet package name `Ontologos` when published). They mirror the Python, Node, and Java APIs via shared logic in `ontologos-js`, using the stable C ABI in **`ontologos-ffi`**.

## Install and build

Published NuGet releases are not yet on a registry — build from a clone:

```bash
# Native library (from repository root)
cargo build -p ontologos-dotnet --release

# C# package + tests (runs cargo build via MSBuild)
dotnet test crates/ontologos-dotnet/csharp/Ontologos.sln
```

Requires **.NET 8+** and a Rust toolchain.

Without the solution runner, use the manual smoke script:

```bash
bash crates/ontologos-dotnet/scripts/smoke.sh
```

Tests load the native library from `target/release`. Override with:

```bash
export ONTOLOGOS_NATIVE_PATH=/absolute/path/to/libontologos_dotnet.dylib
dotnet test crates/ontologos-dotnet/csharp/Ontologos.sln
```

Or set `ONTOLOGOS_REPO_ROOT` to the repository root when running from an IDE.

## Quick start

```csharp
using Ontologos;

using var reasoner = Reasoner.FromPath("family.owl", "rl");
Console.WriteLine(reasoner.Classify());
```

Build in memory:

```csharp
using var builder = new OntologyBuilder();
builder.AddClass("http://example.org/Pizza");
builder.AddClass("http://example.org/Food");
builder.SubclassOf("http://example.org/Pizza", "http://example.org/Food");
using var ontology = builder.Build();
using var reasoner = new Reasoner(ontology, "el");
Console.WriteLine(reasoner.Classify());
```

## File loading

| Method | Parse mode | Sandbox | Use when |
|--------|------------|---------|----------|
| `Ontology.Load(path)` | Lenient | No | Trusted local corpora only |
| `Ontology.LoadIn(base, path)` | Strict | Yes | User uploads; path resolved under `base` |
| `Reasoner.FromPath(path)` | Lenient | No | Trusted paths |
| `Reasoner.LoadIn(base, path)` | Strict | Yes | Sandboxed server uploads |

For untrusted uploads, prefer **`LoadIn`** with a dedicated sandbox directory. Relative `path` values are resolved under `base`.

In-memory untrusted input should use **`Ontology.FromBytes`** / **`FromText`** (strict). Use **`FromBytesLenient`** / **`FromTextLenient`** only for trusted corpora.

## Resource limits

For JSON snapshots from users:

```csharp
var ontology = Ontology.FromJsonWithLimits(json, maxJsonBytes: 1_048_576L);
// or
var ontology = Ontology.FromJsonWithLimits(json, 1_048_576L, 100_000L, 1_000_000L, 8192L);
```

Defaults match [`Limits`](../reference/core.md) in `ontologos-core` (16 MiB JSON, 1M entities, etc.).

## Shared ontology and incremental edits

`Reasoner` and `Ontology` share the same in-memory graph when constructed together. Incremental axiom edits on the reasoner are visible on the ontology handle:

```csharp
using var ontology = builder.Build();
using var reasoner = new Reasoner(ontology, "el");
reasoner.AddSubclassOf("http://example.org/A", "http://example.org/B");
Console.WriteLine(ontology.AxiomCount); // reflects the new axiom
```

The handle uses `Rc<RefCell<…>>` internally — **single-threaded only**. Do not share across threads without separate instances.

## Errors

.NET surfaces parse and limit failures with typed exceptions: `ParseException`, `ResourceLimitException`, `IncompleteReasoningException`, and `OntologyConflictException`. All extend `OntologosException`.

Use `OntologosInfo.ErrorCodeFromMessage(message)` to extract a code string (`ParseError`, etc.) from a message prefix.

## Server deployment notes

- OWL file parsing uses a **process-wide horned-owl mutex** — concurrent `Load` / `LoadIn` calls serialize. Use a queue or worker pool with one load at a time per process.
- Set `budgetSecs` on `Reasoner` for DL workloads to avoid unbounded tableau work.
- See [Security](../security.md#dotnet-bindings) and [Production integration](production-integration.md).

## API surface

Namespace `Ontologos` exposes `Ontology`, `OntologyBuilder`, `Reasoner`, and `EntailmentCheck` with `Classify()`, `CheckConsistency()`, `IsEntailed()`, `Query()`, `Explain()`, and incremental axiom helpers — aligned with the [Python guide](python.md), [Node.js guide](node.md), and [Java guide](java.md).

Complex results return **JSON strings** for parsing with `System.Text.Json`.

## Native library

| Platform | File |
|----------|------|
| macOS | `libontologos_dotnet.dylib` |
| Linux | `libontologos_dotnet.so` |
| Windows | `ontologos_dotnet.dll` |

Load with `NativeLibrary.Load` after placing the library on the loader search path, or set `ONTOLOGOS_NATIVE_PATH`.
