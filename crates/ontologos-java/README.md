# OntoLogos Java bindings

JNI bindings for [OntoLogos](https://github.com/eddiethedean/ontologos), sharing logic with Node/WASM via `ontologos-js`.

## Build

Requires **Java 17+**, **Maven 3.9+**, and a Rust toolchain.

```bash
# From repository root
cargo build -p ontologos-jni --release

# Java package + tests (runs cargo build via Maven)
cd crates/ontologos-java/java
mvn test
```

Tests load the native library from `target/release` via `-Djava.library.path`. Override with:

```bash
mvn test -Dontologos.native.path=/absolute/path/to/libontologos_jni.dylib
```

## Usage

```java
import dev.ontologos.*;

try (OntologyBuilder builder = new OntologyBuilder()) {
    builder.addClass("http://example.org/Pizza");
    builder.addClass("http://example.org/Food");
    builder.subclassOf("http://example.org/Pizza", "http://example.org/Food");
    try (Ontology ontology = builder.build();
         Reasoner reasoner = new Reasoner(ontology, "el")) {
        String report = reasoner.classify();
        System.out.println(report);
    }
}
```

Load OWL from disk:

```java
try (Reasoner reasoner = Reasoner.fromPath("family.owl", "rl")) {
    System.out.println(reasoner.classify());
}
```

## API

Package `dev.ontologos` mirrors the Python/Node bindings:

| Class | Purpose |
|-------|---------|
| `Ontology` | In-memory ontology; `fromJson`, `fromBytes`, `load`, `loadIn` |
| `OntologyBuilder` | Fluent TBox/ABox construction |
| `Reasoner` | `classify`, `checkConsistency`, `isEntailed`, `query`, incremental edits |
| `EntailmentCheck` | Builder for entailment checks |

Complex results (`classify`, `explain`, `query`) return **JSON strings** for easy parsing with Jackson or Gson.

## Security

| API | Use when |
|-----|----------|
| `Ontology.loadIn(base, path)` | User uploads (strict, sandboxed) |
| `Ontology.load(path)` | Trusted local files only |
| `Ontology.fromBytes` / `fromText` | Untrusted in-memory input (strict) |
| `fromJsonWithLimits` | Untrusted JSON snapshots |

See [Java guide](https://ontologos.readthedocs.io/en/latest/guides/java.html) and [Security](https://ontologos.readthedocs.io/en/latest/security.html).

## Native library name

| Platform | File |
|----------|------|
| macOS | `libontologos_jni.dylib` |
| Linux | `libontologos_jni.so` |
| Windows | `ontologos_jni.dll` |

Load with `System.loadLibrary("ontologos_jni")` after placing the library on `java.library.path`.
