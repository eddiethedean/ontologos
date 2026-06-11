# Error Reference

Errors returned by `ontologos_core` use the unified [`Error`](https://docs.rs/ontologos-core/latest/ontologos_core/enum.Error.html) enum.

## `InvalidIri`

**Cause:** IRI failed validation (empty, relative, disallowed scheme, control character, too long, whitespace).

**Recovery:** Use absolute `http`, `https`, or `urn` IRIs. See [security.md](../security.md).

## `EntityKindMismatch`

**Cause:** Same IRI registered with conflicting kinds (e.g. `Class` then `Individual`).

**Recovery:** Use consistent kinds per IRI. Error includes the resolved IRI string.

## `UnknownEntity`

**Cause:** `EntityId` or axiom reference points to an unregistered entity.

**Recovery:** Register entities before axioms that reference them.

## `InvalidAxiom`

**Cause:** Axiom failed validation (wrong entity kind, duplicate operands, self-inverse property, unknown IRI in JSON axiom, etc.).

**Recovery:** Check axiom shape against [SPEC.md](../../SPEC.md) and entity kinds.

## `ParseNotAvailable`

**Cause:** `Ontology::from_file` or CLI load before v0.2 parser.

**Recovery:** Use `Ontology::builder()` or `Ontology::from_json()` with JSON v2.

## `Serialization`

**Cause:** JSON parse failure, unsupported `format_version`, limit exceeded, unknown fields, duplicate entities.

**Recovery:** Validate against [json-snapshot-v2.md](../json-snapshot-v2.md). Use `from_json_with_limits` for untrusted input.

## `NotImplemented`

**Cause:** `Reasoner::classify()` or other engine stubs called.

**Recovery:** Wait for roadmap milestone or use external reasoner (HermiT/ELK).

## `OntologyNotLoaded`

**Cause:** Reasoner used without ontology (reserved for future API).

## `Message`

**Cause:** Generic validation failure (e.g. invalid `parallelism` in `ReasonerBuilder`).

**Recovery:** Read the message string; typically configuration bounds (parallelism 1–64).

## Lookup vs validation

| API | Invalid IRI | Unknown entity |
|-----|-------------|----------------|
| `try_lookup_entity(iri)` | `Err(InvalidIri)` | `Ok(None)` |
| `lookup_entity(iri)` | `Ok(None)` | `Ok(None)` |

Prefer `try_lookup_entity` when distinguishing invalid input from missing entities.
