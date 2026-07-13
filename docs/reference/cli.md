# CLI Reference

Binary: `ontologos` (from `ontologos-cli` crate).

> **v1.0.0:** Ten subcommands; `classify` routes by `--profile` (default `auto`). Use `check_consistency` semantics via `consistent` (JSON includes `complete`). Profile status: [Profile stability matrix](../guides/profile-stability.md).

## Install

`ontologos-cli` is **not published to crates.io** (`publish = false`). Build from a repository clone or install from git.

### Build from clone

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
cargo build -p ontologos-cli --release
./target/release/ontologos --help
```

### Install from git

Requires Rust 1.88+:

```bash
cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli
ontologos --help
```

## Global options

| Flag | Values | Default | Description |
|------|--------|---------|-------------|
| `--profile` | `auto`, `el`, `rl`, `rdfs`, `alc`, `dl`, `dl-preview`, `swrl` | `auto` | Engine profile (see per-command notes) |
| `--format` | `text`, `json` | `text` | Output format |
| `--incremental` | flag | off | Incremental session (`classify`, `materialize` only; no-op elsewhere) |
| `--budget-secs` | seconds | unlimited | Wall-clock budget for DL consistency/classify paths |

**Exit codes:** `0` success, `1` any error (parse, incomplete consistency, inconsistent ontology, etc.). No differentiated exit codes in v1.0.

**`parse_meta`:** When the parser emitted warnings or skipped axioms, JSON output may include `parse_meta` (`warnings`, `mapped_axiom_count`, `skipped_axiom_count`, `logical_axiom_count`). Omitted when clean.

## Subcommands

| Command | Honors `--profile` | Honors `--incremental` | Description |
|---------|-------------------|------------------------|-------------|
| `profile <file>` | No | No | Detect OWL profile (EL/RL/QL/DL) |
| `classify <file>` | Yes | Yes | Profile-routed classification / saturation |
| `materialize <file>` | No (always RDFS) | Yes | Explicit RDFS TBox materialization |
| `explain <file>` | Yes | No | Proof graph JSON/text |
| `query <file>` | Yes | No | OWL QL conjunctive query (requires taxonomy outcome) |
| `instances <file>` | No | No | ABox individuals after RL materialization |
| `consistent <file>` | Yes | No | Consistency check (`check_consistency` semantics) |
| `entail <file>` | Yes | No | Entailment for SubClassOf / ClassAssertion / ObjectPropertyAssertion |
| `subproperties <file>` | Yes | No | Sub-object-properties of a named property |
| `property-values <file>` | Yes | No | Object property values for an individual |

---

### `profile <file>`

Detects the OWL 2 profile and hybrid module layout.

**JSON:**

```json
{
  "detected": "RL",
  "diagnostics": [],
  "hybrid_modules": 1
}
```

---

### `classify <file>`

| `--profile` | Engine | Output |
|-------------|--------|--------|
| `el` | `ontologos-el` | Taxonomy (subsumptions, equivalences, unsatisfiable) |
| `rl` | `ontologos-rl` | Materialization report |
| `rdfs` | `ontologos-rl` (`rdfs` module) | Materialization report |
| `auto` | detect → EL, RL, or DL | Taxonomy or materialization report |
| `dl-preview` | `ontologos-dl` (gated) | Taxonomy + preview warning on stderr |
| `dl` | `ontologos-dl` | Taxonomy (stable on v1.1.3) |
| `alc` | `ontologos-alc` | Taxonomy (preview) |
| `swrl` | `ontologos-swrl` | DLSafe rules + DL consistency (stable on v1.1.3) |

Profile status: [Profile stability matrix](../guides/profile-stability.md).

**EL / DL / ALC taxonomy JSON:**

```json
{
  "status": "classified",
  "subsumption_count": 84,
  "subsumptions": [["http://...", "http://..."]],
  "equivalences": [],
  "unsatisfiable": []
}
```

**RL / RDFS JSON:**

```json
{
  "status": "classified",
  "initial_axiom_count": 57,
  "final_axiom_count": 62,
  "inferred_axioms": 5,
  "inferred_by_rule": {},
  "clash_count": 0
}
```

---

### `materialize <file>`

Always runs RDFS materialization (`--profile` ignored). JSON shape matches RL/RDFS classify with `"status": "materialized"`.

---

### `explain <file>`

Proof graph after profile-routed classification. See [Explain API](explain.md).

**JSON:** `node_count`, `nodes` (each with `rule`, `premises`, optional conclusion fields), optional `parse_meta`.

---

### `query <file> --query <cq>`

Runs `classify` with `--profile`, then answers an OWL QL conjunctive query (e.g. `Type(?x, http://ex.org/A)`). Requires a taxonomy classification outcome (EL/DL/ALC/auto-EL).

**JSON:**

```json
{
  "answers": 2,
  "results": [
    {
      "bindings": [
        { "variable": "x", "iri": "http://ex.org/ind1" }
      ]
    }
  ]
}
```

---

### `instances <file>`

RL-backed ABox materialization: consistency flag, inference count, sameAs cluster count.

**JSON:**

```json
{
  "consistent": true,
  "rl_inferences": 12,
  "same_as_clusters": 0
}
```

Text mode lists individuals, types, and sameAs clusters.

---

### `consistent <file>`

Production consistency API. When `complete` is `false`, stderr warns and `consistent` must not be treated as proof.

**JSON:**

```json
{
  "consistent": true,
  "complete": true
}
```

---

### `entail <file>`

Exactly one entailment form:

| Flags | Check |
|-------|-------|
| `--sub`, `--sup` | `SubClassOf` |
| `--individual`, `--class` | `ClassAssertion` |
| `--subject`, `--property`, `--object` | `ObjectPropertyAssertion` |

**JSON:**

```json
{
  "entailed": true,
  "SubClassOf": { "sub": "http://...", "sup": "http://..." }
}
```

(`ClassAssertion` / `ObjectPropertyAssertion` variants use the same flattened untagged shape.)

---

### `subproperties <file> --property <iri> [--direct]`

**JSON:**

```json
{
  "property": "http://ex.org/hasPart",
  "direct": false,
  "subproperties": ["http://ex.org/partOf"]
}
```

---

### `property-values <file> --subject <iri> --property <iri>`

Uses ABox RL lookup (`get_object_property_values`); the global `--profile` flag does **not** affect this command.

**JSON:**

```json
{
  "subject": "http://ex.org/alice",
  "property": "http://ex.org/hasChild",
  "values": ["http://ex.org/bob"]
}
```

---

## Examples

```bash
ontologos profile benchmarks/data/family.owl
ontologos classify --profile el benchmarks/data/pizza.owl
ontologos classify --profile rl benchmarks/data/family.owl
ontologos classify --profile auto ontology.owl
ontologos classify --profile dl-preview benchmarks/data/family.owl
ontologos --format json consistent --profile dl ontology.owl
ontologos --format json query --profile el pizza.owl --query 'Type(?x, http://www.co-ode.org/ontologies/pizza/pizza.owl#VegetarianPizza)'
ontologos entail --profile el pizza.owl --sub http://... --sup http://...
ontologos subproperties --profile el ontology.owl --property http://ex.org/hasPart
ontologos property-values family.owl --subject http://... --property http://...
ontologos instances family.owl
ontologos materialize ontology.owl
ontologos --format json explain --profile el benchmarks/data/pizza.owl
ontologos classify --incremental --profile el ontology.owl
```

### `--incremental`

Single-shot CLI mode: loads the ontology, runs one incremental pass, and exits. Library and Python workflows can hold session state across multiple `classify()` calls after in-memory edits. See [Incremental reasoning](../guides/incremental-reasoning.md).

## Migration from v0.4

v0.4 `classify` always ran RDFS. See [v0.4.x → v0.5.0](../migration/v0.4.x-to-v0.5.0.md).
