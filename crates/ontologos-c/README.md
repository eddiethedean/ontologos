# OntoLogos C/C++ bindings

C ABI bindings for [OntoLogos](https://github.com/eddiethedean/ontologos), sharing logic with .NET/Java/Node via `ontologos-ffi` and `ontologos-js`.

## Build

Requires a **C11** / **C++17** toolchain and **Rust**.

```bash
# From repository root
cargo build -p ontologos-c --release

# C + C++ smoke tests (CMake)
bash crates/ontologos-c/scripts/smoke.sh
```

Headers live in `crates/ontologos-c/include/`:

- `ontologos.h` — stable C API
- `ontologos.hpp` — optional C++ RAII wrappers

## Usage (C)

```c
#include "ontologos.h"

ontologos_handle_t builder = ontologos_builder_new();
builder = ontologos_builder_add_class(builder, "http://example.org/Pizza");
builder = ontologos_builder_add_class(builder, "http://example.org/Food");
builder = ontologos_builder_subclass_of(
    builder, "http://example.org/Pizza", "http://example.org/Food");
ontologos_handle_t ontology = ontologos_builder_build(builder);

ontologos_handle_t reasoner = ontologos_reasoner_new(ontology, "el", 0, -1);
char *report = ontologos_reasoner_classify(reasoner);
/* parse JSON, then: */
ontologos_string_free(report);
ontologos_reasoner_close(reasoner);
ontologos_ontology_close(ontology);
```

Check `ontologos_last_error_code()` / `ontologos_last_error_message()` after failures; free returned strings with `ontologos_string_free`.

## Usage (C++)

```cpp
#include "ontologos.hpp"

ontologos::OntologyBuilder builder;
builder.add_class("http://example.org/Pizza")
    .add_class("http://example.org/Food")
    .subclass_of("http://example.org/Pizza", "http://example.org/Food");
ontologos::Ontology ontology = builder.build();
ontologos::Reasoner reasoner(ontology, "el");
std::string report = reasoner.classify();
```

## Native library

| Platform | File |
|----------|------|
| macOS | `libontologos_c.dylib` |
| Linux | `libontologos_c.so` |
| Windows | `ontologos_c.dll` |

Link with `-lontologos_c` (or full path) and include `ontologos.h`.

See [C/C++ guide](https://ontologos.readthedocs.io/en/latest/guides/c-cpp.html).
