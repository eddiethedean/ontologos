# C/C++ Guide

C and C++ bindings ship in the workspace crate **`ontologos-c`** (`libontologos_c`). The stable C ABI is implemented in **`ontologos-ffi`** and shared with the .NET bindings.

## Install and build

Build from a clone:

```bash
# Native library (from repository root)
cargo build -p ontologos-c --release

# C + C++ smoke tests (CMake)
bash crates/ontologos-c/scripts/smoke.sh
```

Requires a **C11** / **C++17** toolchain, **CMake 3.16+**, and Rust.

Headers: `crates/ontologos-c/include/ontologos.h` (C) and `ontologos.hpp` (optional C++).

## Quick start (C)

```c
#include "ontologos.h"
#include <stdio.h>

ontologos_handle_t reasoner =
    ontologos_reasoner_from_path("family.owl", "rl", 0, -1, 0);
char *report = ontologos_reasoner_classify(reasoner);
if (report) {
    puts(report);
    ontologos_string_free(report);
}
ontologos_reasoner_close(reasoner);
```

Build in memory:

```c
ontologos_handle_t builder = ontologos_builder_new();
builder = ontologos_builder_add_class(builder, "http://example.org/Pizza");
builder = ontologos_builder_add_class(builder, "http://example.org/Food");
builder = ontologos_builder_subclass_of(
    builder, "http://example.org/Pizza", "http://example.org/Food");
ontologos_handle_t ontology = ontologos_builder_build(builder);
ontologos_handle_t reasoner = ontologos_reasoner_new(ontology, "el", 0, -1);
char *report = ontologos_reasoner_classify(reasoner);
ontologos_string_free(report);
ontologos_reasoner_close(reasoner);
ontologos_ontology_close(ontology);
```

## Quick start (C++)

```cpp
#include "ontologos.hpp"
#include <iostream>

ontologos::OntologyBuilder builder;
builder.add_class("http://example.org/Pizza")
    .subclass_of("http://example.org/Pizza", "http://example.org/Food");
ontologos::Ontology ontology = builder.build();
ontologos::Reasoner reasoner(ontology, "el");
std::cout << reasoner.classify() << '\n';
```

The C++ header provides RAII wrappers for common flows; the full API remains in `ontologos.h`.

## File loading

| Function | Parse mode | Sandbox | Use when |
|----------|------------|---------|----------|
| `ontologos_ontology_load` | Lenient | No | Trusted local corpora only |
| `ontologos_ontology_load_in` | Strict | Yes | User uploads |
| `ontologos_reasoner_from_path` | Lenient | No | Trusted paths |
| `ontologos_reasoner_load_in` | Strict | Yes | Sandboxed server uploads |

For untrusted in-memory input use **`ontologos_ontology_from_bytes`** / **`from_text`** (strict). Use lenient variants only for trusted corpora.

## Resource limits

For JSON snapshots from users:

```c
ontologos_handle_t ontology = ontologos_ontology_from_json_with_limits(
    json, 1048576, -1, -1, -1); /* 1 MiB JSON cap; -1 = default */
```

Defaults match [`Limits`](../reference/core.md) in `ontologos-core`.

## Errors and memory

- Failures set thread-local errors — read with `ontologos_last_error_code()` and `ontologos_last_error_message()`, then `ontologos_clear_last_error()`.
- Strings returned by the library must be freed with `ontologos_string_free`.
- Handles must be closed with the matching `*_close` function (or C++ destructors).

## API surface

The C API mirrors Python/Node/Java/.NET: `classify`, `check_consistency`, `is_entailed`, `query`, `explain`, and incremental axiom helpers. Complex results are **JSON strings**.

## Linking

| Platform | Library |
|----------|---------|
| macOS | `libontologos_c.dylib` |
| Linux | `libontologos_c.so` |
| Windows | `ontologos_c.dll` |

Example (Unix):

```bash
cc -Icrates/ontologos-c/include app.c -Ltarget/release -lontologos_c -Wl,-rpath,target/release
```

See [Security](../security.md#c-c-bindings) and [Production integration](production-integration.md).
