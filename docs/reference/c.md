# C/C++ API Reference

C/C++ bindings via **`ontologos-c`** (`libontologos_c`). Source-build only — see [Bindings overview](../guides/bindings-overview.md).

Tutorial: [C/C++ guide](../guides/c-cpp.md).

## Install

```bash
cargo build -p ontologos-c --release
# Headers: crates/ontologos-c/include/ontologos.h, ontologos.hpp
```

Requires Rust toolchain; C++17 optional for C++ wrapper.

## Profiles

C string literals matching Python: `"auto"`, `"rdfs"`, `"rl"`, `"el"`, `"dl"`, `"swrl"`, `"alc"`, `"dl-preview"`.

## Core types

| C type | Description |
|--------|-------------|
| `OntologosOntology*` | In-memory ontology handle |
| `OntologosReasoner*` | Reasoner handle |
| `OntologosBuilder*` | Builder handle |
| `OntologosError*` | Last error (thread-local) |

C++ wrapper classes in `ontologos.hpp` provide RAII (`Ontology`, `Reasoner`, `OntologyBuilder`).

## Reasoner (C)

| Function | Description |
|----------|-------------|
| `ontologos_reasoner_from_path(path, profile, &out)` | Load + construct |
| `ontologos_reasoner_new(ontology, profile, incremental, budget_secs, &out)` | From ontology |
| `ontologos_reasoner_classify(reasoner, &json_out)` | Classify → JSON string |
| `ontologos_reasoner_check_consistency(reasoner, &json_out)` | Consistency check |
| `ontologos_reasoner_explain(reasoner, &json_out)` | Proof graph |
| `ontologos_reasoner_free(reasoner)` | Release handle |

All JSON-returning functions allocate output strings — caller must `ontologos_string_free`.

## Ontology builder (C)

| Function | Description |
|----------|-------------|
| `ontologos_builder_new(&out)` | Create builder |
| `ontologos_builder_add_class(builder, iri)` | Declare class |
| `ontologos_builder_subclass_of(builder, sub, sup)` | Add axiom |
| `ontologos_builder_build(builder, &ontology_out)` | Build ontology |

## Errors

Check return code (`ONTologos_OK` vs error enum). `ontologos_last_error()` returns message for the calling thread.

## Related

- [Python API](python.md) — canonical parity reference
- [Security](../security.md)
- [Troubleshooting — bindings](../guides/troubleshooting.md#binding-build-failures)
