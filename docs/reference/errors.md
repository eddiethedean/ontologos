# Error Reference

## `ontologos_core::Error`

Errors from the core crate use [`Error`](https://docs.rs/ontologos-core/latest/ontologos_core/enum.Error.html).

### `InvalidIri`

**Cause:** IRI failed validation (empty, relative, disallowed scheme, control character, too long, whitespace).

**Recovery:** Use absolute `http`, `https`, or `urn` IRIs. See [security.md](../security.md).

### `EntityKindMismatch`

**Cause:** Same IRI registered with conflicting kinds (e.g. `Class` then `Individual`).

**Recovery:** Use consistent kinds per IRI. Error includes the resolved IRI string.

### `UnknownEntity`

**Cause:** `EntityId` or axiom reference points to an unregistered entity.

**Recovery:** Register entities before axioms that reference them.

### `InvalidAxiom`

**Cause:** Axiom failed validation (wrong entity kind, duplicate operands, self-inverse property, unknown IRI in JSON axiom, etc.).

**Recovery:** Check axiom shape against [SPEC.md](../../SPEC.md) and [json-snapshot-v2.md](../json-snapshot-v2.md).

### `ParseNotAvailable`

**Cause:** `Ontology::from_file` called on `ontologos-core` (parser not linked).

**Recovery:** Use `ontologos_parser::load_ontology` or `Ontology::builder()` / `from_json()`. See [Load an OWL file](../getting-started/load-owl-file.md).

### `Serialization`

**Cause:** JSON parse failure, unsupported `format_version`, limit exceeded, unknown fields, duplicate entities.

**Recovery:** Validate against [json-snapshot-v2.md](../json-snapshot-v2.md). Use `from_json_with_limits` for untrusted input.

### `NotImplemented`

**Cause:** `Reasoner::classify()`, CLI `classify`/`materialize`/`explain`, or other engine stubs.

**Recovery:** Use `ontologos profile` for v0.2; wait for roadmap milestone or use external reasoner (HermiT/ELK).

### `OntologyNotLoaded`

**Cause:** Reasoner used without ontology (reserved for future API).

### `Message`

**Cause:** Generic validation failure (e.g. invalid `parallelism` in `ReasonerBuilder`).

**Recovery:** Read the message string; typically configuration bounds (parallelism 1–64).

### Lookup vs validation

| API | Invalid IRI | Unknown entity |
|-----|-------------|----------------|
| `try_lookup_entity(iri)` | `Err(InvalidIri)` | `Ok(None)` |
| `lookup_entity(iri)` | `Ok(None)` | `Ok(None)` |

Prefer `try_lookup_entity` when distinguishing invalid input from missing entities.

---

## `ontologos_parser::Error`

| Variant | Cause | Recovery |
|---------|-------|----------|
| `UnsupportedFormat` | Unknown extension or undetectable format | Use `.owl`, `.rdf`, `.ttl`, `.ofn`; see [supported formats](../getting-started/load-owl-file.md) |
| `Parse` | Missing file, path traversal, size limit, horned-owl parse failure | Check path, run `validate_load_path`; use `ParseLimits` for uploads |
| `Core` | Wrapped `ontologos_core::Error` during mapping | Fix entity/axiom issues |

Parser warnings in `ParseMeta` are non-fatal.

---

## `ontologos_profile::Error`

| Variant | Cause | Recovery |
|---------|-------|----------|
| `Message` | Profile detection internal failure (rare) | Report issue; check ontology has valid `parse_meta` or axioms |

`detect_profile` normally returns `Ok(ProfileReport)` with `detected: Some(...)` or diagnostics.
