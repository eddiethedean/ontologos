# `owl:imports` behavior

How OntoLogos handles imported ontologies when loading files.

## Summary

| Aspect | Behavior |
|--------|----------|
| **Remote URLs** | **Not fetched** — `http://` or `https://` import IRIs are not downloaded over the network |
| **RDF/XML** (`.owl`, `.rdf`, `.xml`) | **Local imports merged** when using `load_ontology()` (`merge_imports: true`); `ParseLimits::default()` has `merge_imports: false` |
| **Turtle** (`.ttl`, `.turtle`) | **Not merged** — only axioms in the loaded file |
| **OWL Functional** (`.ofn`, `.func`) | **Not merged** — only axioms in the loaded file |
| **Transitive imports** | RDF/XML merge follows import chains for **local** files only (visited-set deduplication) |

OntoLogos does **not** implement a full OWL catalog resolver. Import IRIs must map to files relative to the loaded document (or sandbox base directory).

## Decision table: which loader?

| I use… | `merge_imports` default | Local RDF/XML imports merged? | When to set `merge_imports: false` |
|--------|-------------------------|-------------------------------|-------------------------------------|
| `load_ontology(path)` on **RDF/XML** | `true` | **Yes** | Single-document load only |
| `load_ontology(path)` on **Turtle / OFN** | n/a | **No** | — |
| `load_ontology_with_limits(path, ParseLimits::default())` | `false` | **No** | Set `merge_imports: true` explicitly for RDF/XML bundles |
| `load_ontology_in(base, path)` | same as above | Same rules | Untrusted uploads — prefer `false` unless sibling imports are trusted |

!!! warning "Common mistake"
    `load_ontology()` merges imports; `ParseLimits::default()` does **not**. If you switch to `load_ontology_with_limits` for size caps, set `merge_imports` explicitly.

## RDF/XML: default merge

For RDF/XML loads, `load_ontology` merges **declared** `owl:imports` that resolve to local files:

```rust
use ontologos_parser::{load_ontology, load_ontology_with_limits, ParseLimits};

// load_ontology() opts in to merge_imports = true for RDF/XML
let ontology = load_ontology(path)?;

// Single-document load only (no import merge)
let limits = ParseLimits {
    merge_imports: false,
    ..ParseLimits::default()
};
let ontology = load_ontology_with_limits(path, limits)?;
```

Imported documents are loaded recursively, but nested import merge is disabled per import file (`merge_imports: false` on nested loads) to avoid duplicate expansion — the visited set prevents cycles.

## When to merge upstream

Merge with external tooling when:

- Imports use **remote IRIs** (not on disk next to the ontology)
- You load **Turtle** or **OWL Functional** and need a multi-file bundle
- You need Protégé-style catalog resolution across arbitrary paths

**ROBOT example:**

```bash
robot merge --input ontology.owl --output merged.owl
```

Then load `merged.owl` with OntoLogos.

## Diagnostics

`owl:imports` declarations appear in `parse_meta.constructs` whether or not merge succeeds. Unresolvable imports do not fail the load — axioms from those imports are simply absent. Check `parse_meta.warnings` for skipped imports (e.g. axiom limit exceeded).

## Related

- [Parser API](parser.md) — `load_ontology`, `ParseLimits`
- [Load an OWL file](../getting-started/load-owl-file.md)
- [Known limitations](../guides/known-limitations.md)
- [Troubleshooting](../guides/troubleshooting.md)
