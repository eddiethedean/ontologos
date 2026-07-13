# Parser API Reference

OWL and RDF file loading via [`ontologos-parser`](https://docs.rs/ontologos-parser/1.1.4).

## Overview

| Function | Purpose |
|----------|---------|
| [`load_ontology`](https://docs.rs/ontologos-parser/1.1.4/ontologos_parser/fn.load_ontology.html) | Load with default `ParseLimits` |
| [`load_ontology_with_limits`](https://docs.rs/ontologos-parser/1.1.4/ontologos_parser/fn.load_ontology_with_limits.html) | Load with custom limits |
| [`load_ontology_with_limits_and_base`](https://docs.rs/ontologos-parser/1.1.4/ontologos_parser/fn.load_ontology_with_limits_and_base.html) | Load with sandbox base directory |
| [`load_ontology_in`](https://docs.rs/ontologos-parser/1.1.4/ontologos_parser/fn.load_ontology_in.html) | Load relative to a base directory |
| [`load_ontology_lenient`](https://docs.rs/ontologos-parser/1.1.4/ontologos_parser/fn.load_ontology_lenient.html) | Lenient limits (`strict: false`) |
| [`load_ofn_from_str`](https://docs.rs/ontologos-parser/1.1.4/ontologos_parser/fn.load_ofn_from_str.html) | Parse OWL Functional Syntax from a string |

**Do not use** `ontologos_core::Ontology::from_file` — it returns `ParseNotAvailable`. Always use the parser crate.

## Supported formats

| Extension | Format |
|-----------|--------|
| `.owl` | OWL/XML or RDF/XML (content sniffed) |
| `.rdf`, `.xml` | RDF/XML |
| `.ttl`, `.turtle` | Turtle |
| `.ofn`, `.func` | OWL Functional Syntax |

## Quick start

```rust
use ontologos_parser::load_ontology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ontology = load_ontology(std::path::Path::new("family.owl"))?;
    println!("mapped axioms: {}", ontology.axiom_count());
    if let Some(meta) = ontology.parse_meta() {
        println!("skipped: {}", meta.skipped_axiom_count);
    }
    Ok(())
}
```

## `ParseLimits`

Resource limits enforced during parse. See [Security](../security.md).

| Field | Default | Description |
|-------|---------|-------------|
| `max_file_bytes` | 64 MiB | Maximum file size |
| `max_axioms` | 10_000_000 | Maximum axioms in core |
| `max_entities` | 1_000_000 | Maximum registered entities |
| `max_expanded_bytes` | 4× file size | RDF/XML entity expansion cap |
| `max_preprocess_bytes` | 8× file size | Cumulative preprocess budget |
| `max_harvested_assertions` | 100_000 | RDF/XML OFN supplement cap |
| `strict` | `true` | Error when limits cause skips |
| `merge_imports` | `false` | Merge local `owl:imports` for **RDF/XML** only — `load_ontology()` sets `true` |
| `validate_output` | `true` | Post-load validation |

```rust
use ontologos_parser::{load_ontology_with_limits, ParseLimits};

let limits = ParseLimits {
    max_file_bytes: 1_048_576,
    merge_imports: false,
    ..ParseLimits::default()
};
let ontology = load_ontology_with_limits(path, limits)?;
```

`ParseLimits::lenient()` sets `strict: false` and `validate_output: false`.

## `owl:imports`

Import behavior is **format-dependent**. See [OWL imports reference](owl-imports.md).

- **RDF/XML:** `load_ontology()` merges local imports (`merge_imports: true`); `ParseLimits::default()` does not — set `merge_imports: true` explicitly when needed
- **Turtle / OWL Functional:** not merged
- **Remote URLs:** never fetched

## Parse metadata

After loading, `ontology.parse_meta()` contains:

| Field | Meaning |
|-------|---------|
| `mapped_axiom_count` | Axioms stored in core |
| `skipped_axiom_count` | Logical components not mapped |
| `logical_axiom_count` | `mapped + skipped` |
| `constructs` | OWL constructs observed |
| `warnings` | Skipped shapes and mapping messages |

## Errors

| Error | Typical cause |
|-------|---------------|
| `UnsupportedFormat` | Unrecognized extension |
| `Parse` | Malformed file, size limit, strict limit violation |

See [Error reference](errors.md).

## Related

- [Load an OWL file](../getting-started/load-owl-file.md)
- [OWL imports](owl-imports.md)
- [Supported constructs](supported-constructs.md)
- [docs.rs/ontologos-parser](https://docs.rs/ontologos-parser/1.1.4)
