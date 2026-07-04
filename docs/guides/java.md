# Java Guide

Java bindings ship in the workspace crate **`ontologos-jni`** (Maven artifact `dev.ontologos:ontologos`). They mirror the Python and Node APIs via shared logic in `ontologos-js`.

## Install and build

Published Maven releases are not yet on a registry — build from a clone:

```bash
# Native library (from repository root)
cargo build -p ontologos-jni --release

# Java package + tests (runs cargo build via Maven)
cd crates/ontologos-java/java
mvn test
```

Requires **Java 17+**, **Maven 3.9+**, and a Rust toolchain.

Without Maven, use the manual smoke script:

```bash
bash crates/ontologos-java/scripts/smoke.sh
```

Tests load the native library from `target/release` via `java.library.path`. Override with:

```bash
mvn test -Dontologos.native.path=/absolute/path/to/libontologos_jni.dylib
```

Or set `ONTOLOGOS_REPO_ROOT` to the repository root when running from an IDE.

## Quick start

```java
import dev.ontologos.*;

try (Reasoner reasoner = Reasoner.fromPath("family.owl", "rl")) {
    String report = reasoner.classify();
    System.out.println(report);
}
```

Build in memory:

```java
try (OntologyBuilder builder = new OntologyBuilder()) {
    builder.addClass("http://example.org/Pizza");
    builder.addClass("http://example.org/Food");
    builder.subclassOf("http://example.org/Pizza", "http://example.org/Food");
    try (Ontology ontology = builder.build();
         Reasoner reasoner = new Reasoner(ontology, "el")) {
        System.out.println(reasoner.classify());
    }
}
```

## File loading

| Method | Parse mode | Sandbox | Use when |
|--------|------------|---------|----------|
| `Ontology.load(path)` | Lenient | No | Trusted local corpora only |
| `Ontology.loadIn(base, path)` | Strict | Yes | User uploads; path resolved under `base` |
| `Reasoner.fromPath(path)` | Lenient | No | Trusted paths |
| `Reasoner.loadIn(base, path)` | Strict | Yes | Sandboxed server uploads |

For untrusted uploads, prefer **`loadIn`** with a dedicated sandbox directory. Relative `path` values are resolved under `base`.

In-memory untrusted input should use **`Ontology.fromBytes`** / **`fromText`** (strict). Use **`fromBytesLenient`** / **`fromTextLenient`** only for trusted corpora.

## Resource limits

For JSON snapshots from users:

```java
Ontology ontology = Ontology.fromJsonWithLimits(json, 1_048_576L); // 1 MiB cap
// or
Ontology ontology = Ontology.fromObjectWithLimits(
        json, 1_048_576L, 100_000L, 1_000_000L, 8192L);
```

Defaults match [`Limits`](../reference/core.md) in `ontologos-core` (16 MiB JSON, 1M entities, etc.).

## Shared ontology and incremental edits

`Reasoner` and `Ontology` share the same in-memory graph when constructed together. Incremental axiom edits on the reasoner are visible on the ontology handle:

```java
try (Ontology ontology = builder.build();
     Reasoner reasoner = new Reasoner(ontology, "el")) {
    reasoner.addSubclassOf("http://example.org/A", "http://example.org/B");
    System.out.println(ontology.getAxiomCount()); // reflects the new axiom
}
```

The handle uses `Rc<RefCell<…>>` internally — **single-threaded only**. Do not share across threads without separate instances.

## Errors

Java surfaces parse and limit failures with typed exceptions: `ParseException`, `ResourceLimitException`, `IncompleteReasoningException`, and `OntologyConflictException`. All extend `OntologosException`.

Use `Ontologos.errorCodeFromMessage(message)` to extract a code string (`ParseError`, etc.) from a message prefix.

## Server deployment notes

- OWL file parsing uses a **process-wide horned-owl mutex** — concurrent `load` / `loadIn` calls serialize. Use a queue or worker pool with one load at a time per process.
- Set `budgetSecs` on `Reasoner` for DL workloads to avoid unbounded tableau work.
- See [Security](../security.md#java-bindings) and [Production integration](production-integration.md).

## API surface

Package `dev.ontologos` exposes `Ontology`, `OntologyBuilder`, `Reasoner`, and `EntailmentCheck` with `classify()`, `checkConsistency()`, `isEntailed()`, `query()`, `explain()`, and incremental axiom helpers — aligned with the [Python guide](python.md) and [Node.js guide](node.md).

Complex results return **JSON strings** for parsing with Jackson, Gson, or similar.

## Native library

| Platform | File |
|----------|------|
| macOS | `libontologos_jni.dylib` |
| Linux | `libontologos_jni.so` |
| Windows | `ontologos_jni.dll` |

Place the library on `java.library.path`, call `System.load` with an absolute path, or set `-Dontologos.native.path`.
