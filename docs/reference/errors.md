# Error Reference

## `ontologos_core::Error`

Errors from the core crate use [`Error`](https://docs.rs/ontologos-core/0.9.0/ontologos_core/enum.Error.html).

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

**Recovery:** Check axiom shape against [SPEC.md](../project/spec.md) and [json-snapshot-v2.md](../json-snapshot-v2.md).

### `ParseNotAvailable`

**Cause:** `Ontology::from_file` called on `ontologos-core` (parser not linked).

**Recovery:** Use `ontologos_parser::load_ontology` or `Ontology::builder()` / `from_json()`. See [Load an OWL file](../getting-started/load-owl-file.md).

### `Serialization`

**Cause:** JSON parse failure, unsupported `format_version`, limit exceeded, unknown fields, duplicate entities.

**Recovery:** Validate against [json-snapshot-v2.md](../json-snapshot-v2.md). Use `from_json_with_limits` for untrusted input.

### `NotImplemented`

**Cause:** `Reasoner::classify()` on `ontologos-core` does not dispatch to profile engines (by design).

| API | Behavior |
|-----|----------|
| `Reasoner::classify()` with `Profile::El` | Returns `NotImplemented` |
| `Reasoner::classify()` with `Profile::Auto` | Returns `NotImplemented` |

**Recovery:** Use `ontologos_el::classify_reasoner`, `ontologos_rdfs::materialize_reasoner`, or `ontologos_rl::materialize_reasoner`. CLI: `ontologos classify --profile auto|el|rl|rdfs`. Python: `Reasoner(path, profile="auto")` routes via `classify_with_profile`.

Calling `Reasoner::classify()` with `Profile::Rdfs` or `Profile::Rl` returns [`Error::Message`](https://docs.rs/ontologos-core/0.9.0/ontologos_core/enum.Error.html#variant.Message) pointing at `ontologos_rdfs::classify_reasoner` or `ontologos_rl::classify_reasoner` (core does not link profile engines).

### `OntologyNotLoaded`

**Cause:** Reasoner used without ontology (reserved for future API).

### `Message`

**Cause:** Generic validation failure (e.g. invalid `parallelism` in `ReasonerBuilder`) or `Reasoner::classify()` called with `Profile::Rdfs` / `Profile::Rl` (use profile-specific crates instead).

**Recovery:** Read the message string; for parallelism, use bounds 1–64; for RDFS, call `ontologos_rdfs::classify_reasoner` or `materialize_reasoner`; for RL, call `ontologos_rl::classify_reasoner` or `RlEngine::saturate`.

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

---

## `ontologos_rdfs::Error`

| Variant | Cause | Recovery |
|---------|-------|----------|
| `WrongProfile` | Engine invoked with mismatched reasoner profile | Build reasoner with `Profile::Rdfs`; use `RdfsEngine::materialize` directly on `Ontology` to skip profile check |
| `Core` | Wrapped `ontologos_core::Error` | See core section above |

RDFS materialization does not fail on empty ontologies; it returns a report with zero inferences.

---

## `ontologos_rl::Error`

| Variant | Cause | Recovery |
|---------|-------|----------|
| `WrongProfile` | `classify_reasoner` called with non-RL profile | Use `Profile::Rl` or call `RlEngine::saturate` directly |
| `Core` | Wrapped core error (e.g. invalid `parallelism` on `RlEngine::try_new`) | Use parallelism `1..=64`; fix underlying axiom/entity issues |

### RL materialization report notes

`MaterializationReport::clashes` lists human-readable inconsistency messages when detected (direct disjoint class types on an individual; `sameAs` / `differentFrom` conflicts). Clashes do not abort saturation — review the report after `saturate`.

See [RL rules reference](rl-rules.md) for rule names in `inferred_by_rule`.
