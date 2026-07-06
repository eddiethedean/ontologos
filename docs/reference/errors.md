# Error Reference

Classification and consistency live in **`ontologos_facade`** — not on `ontologos_core::Reasoner`. See [Facade reference](facade.md).

## `ontologos_facade::Error`

Transparent wrapper over profile engine errors. See [docs.rs](https://docs.rs/ontologos-facade/1.1.2/ontologos_facade/enum.Error.html).

| Variant | Source | Typical cause |
|---------|--------|---------------|
| `El` | `ontologos_el::Error` | EL classification failure |
| `Dl` | `ontologos_dl::Error` | DL inconsistency, preview limit, budget |
| `Rl` | `ontologos_rl::Error` | Wrong profile or core error |
| `Core` | `ontologos_core::Error` | Validation, incomplete consistency |

### DL errors (via `Dl`)

| Error | Cause | Recovery |
|-------|-------|----------|
| `Inconsistent` | Ontology proved inconsistent | Fix axioms; do not classify |
| `PreviewLimit` | Construct outside preview scope | Use stable profile or simplify |
| `IncompleteConsistency` | Budget or tableau limit (`complete == false`) | Increase `budget_secs`; use `check_consistency` |
| `ResourceLimit` | Tableau expansion exhausted | Reduce ontology size |

**Recovery for classification:** Use `ontologos_facade::classify` with the correct `Profile`. CLI: `ontologos classify --profile auto|el|rl|rdfs|dl`. Python: `Reasoner(path, profile="auto").classify()`.

### Incomplete consistency

**Cause:** `ontologos_facade::is_consistent` when `ConsistencyResult::complete == false` (DL budget).

**Recovery:** Call `ontologos_facade::check_consistency` and inspect `complete`. Increase `ReasonerConfig::budget_secs`. Python: `check_consistency()` or catch `IncompleteReasoningError`.

---

## `ontologos_core::Error`

Errors from the core crate: [`Error`](https://docs.rs/ontologos-core/1.1.2/ontologos_core/enum.Error.html).

### `InvalidIri`

**Cause:** IRI failed validation (empty, relative, disallowed scheme, control character, too long, whitespace).

**Recovery:** Use absolute `http`, `https`, or `urn` IRIs. See [security.md](../security.md).

### `EntityKindMismatch`

**Cause:** Same IRI registered with conflicting kinds (e.g. `Class` then `Individual`).

**Recovery:** Use consistent kinds per IRI.

### `UnknownEntity`

**Cause:** `EntityId` or axiom reference points to an unregistered entity.

**Recovery:** Register entities before axioms that reference them.

### `InvalidAxiom`

**Cause:** Axiom failed validation (wrong entity kind, duplicate operands, self-inverse property, unknown IRI in JSON axiom, etc.).

**Recovery:** Check axiom shape against [SPEC.md](https://github.com/eddiethedean/ontologos/blob/main/SPEC.md) and [JSON snapshot](../json-snapshot-v3.md).

### `ParseNotAvailable`

**Cause:** `Ontology::from_file` called on `ontologos-core` (parser not linked).

**Recovery:** Use `ontologos_parser::load_ontology` or `Ontology::builder()` / `from_json()`. See [Load an OWL file](../getting-started/load-owl-file.md).

### `Serialization`

**Cause:** JSON parse failure, unsupported `format_version`, limit exceeded, unknown fields, duplicate entities.

**Recovery:** Validate against [JSON snapshot v3](../json-snapshot-v3.md). Use `from_json_with_limits` for untrusted input. Format v1 is rejected.

### `NotImplemented`

**Cause:** Reserved for APIs not yet available on core (e.g. some SWRL paths).

**Recovery:** Use `ontologos_facade::classify` or the profile-specific engine crate.

### `OntologyNotLoaded`

**Cause:** Reasoner used without ontology (reserved for future API).

### `Message`

**Cause:** Generic validation failure (e.g. invalid `parallelism` in `ReasonerBuilder`).

**Recovery:** Read the message string; for parallelism, use bounds 1–64.

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
