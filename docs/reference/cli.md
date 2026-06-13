# CLI Reference

Binary: `ontologos` (from `ontologos-cli` crate).

> **v0.9.0:** `classify` routes by `--profile` (default `auto`) to OWL EL taxonomy, OWL RL saturation, or RDFS materialization. Use `materialize` for explicit RDFS. `explain` emits proof graphs (JSON or text).

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
| `--profile` | `auto`, `el`, `rl`, `rdfs` | `auto` | Engine for `classify` |
| `--format` | `text`, `json` | `text` | Output format |
| `--incremental` | flag | off | Enable incremental session mode (library multi-pass; single file load runs one pass) |

## Subcommands

| Command | Status | Description |
|---------|--------|-------------|
| `profile <file>` | **Works** | Detect OWL profile (EL/RL/QL/DL) |
| `classify <file>` | **Works** | Profile-routed classification / saturation |
| `materialize <file>` | **Works** | Explicit RDFS TBox materialization |
| `explain <file>` | **v0.9.0** | Proof graph JSON/text; supports `--profile` |

### `classify`

| `--profile` | Engine | Output |
|-------------|--------|--------|
| `el` | `ontologos-el` | Taxonomy (subsumptions, equivalences, unsatisfiable) |
| `rl` | `ontologos-rl` | Materialization report |
| `rdfs` | `ontologos-rdfs` | Materialization report |
| `auto` | detect → EL or RL | Taxonomy or materialization report |

DL-detected ontologies with `--profile auto` return an error; use an explicit profile or `materialize` for RDFS.

### Examples

```bash
ontologos classify --profile el benchmarks/data/pizza.owl
ontologos classify --profile rl benchmarks/data/family.owl
ontologos classify --profile rdfs ontology.owl
ontologos classify --profile auto ontology.owl
ontologos materialize ontology.owl
ontologos --format json classify --profile el pizza.owl
ontologos explain --profile el benchmarks/data/pizza.owl
ontologos --format json explain --profile el benchmarks/data/pizza.owl
ontologos classify --incremental --profile el ontology.owl
```

### `--incremental`

Single-shot CLI mode: loads the ontology, runs one incremental pass, and exits. Library and Python workflows can hold session state across multiple `classify()` calls after in-memory edits. See [Incremental reasoning](../guides/incremental-reasoning.md).

### JSON output shapes

**`classify --profile el` (JSON):**

```json
{
  "status": "classified",
  "subsumption_count": 84,
  "subsumptions": [["http://...", "http://..."]],
  "equivalences": [],
  "unsatisfiable": []
}
```

**`classify --profile rl|rdfs` / `materialize` (JSON):**

```json
{
  "status": "materialized",
  "initial_axiom_count": 57,
  "final_axiom_count": 62,
  "inferred_axioms": 5,
  "inferred_by_rule": {},
  "clashes": []
}
```

**`explain` (JSON):** Serialized `ProofGraph` — see [Explain API](explain.md).

**`profile` (JSON):**

```json
{
  "detected": "RL",
  "diagnostics": []
}
```

## Migration from v0.4

v0.4 `classify` always ran RDFS. See [v0.4.x → v0.5.0](../migration/v0.4.x-to-v0.5.0.md).
