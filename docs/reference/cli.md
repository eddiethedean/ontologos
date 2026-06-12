# CLI Reference

Binary: `ontologos` (from `ontologos-cli` crate).

## Build

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos --help
```

## Global options

| Flag | Values | Default | Description |
|------|--------|---------|-------------|
| `--format` | `text`, `json` | `text` | Output format |

## Subcommands

| Command | v0.4 status | Description |
|---------|-------------|-------------|
| `profile <file>` | **Works** | Detect OWL profile (EL/RL/QL/DL) |
| `materialize <file>` | **Works** | RDFS TBox materialization with report |
| `classify <file>` | **RDFS** | RDFS materialization via reasoner (`Profile::Rdfs`); same inference report as `materialize` with `status: classified` |
| `explain <file>` | Stub | Loads file; explain → `NotImplemented` (v0.6) |

All commands load the ontology via `ontologos_parser::load_ontology` first.

`classify` and `materialize` both run the RDFS engine only. OWL RL saturation is available via `ontologos-rl` (library) or Python `profile="rl"`. OWL EL taxonomy classification and CLI profile routing ship in v0.5.

## `profile` output

### Text

Family example:

```text
detected profile: Rl
diagnostics:
  - ObjectIntersectionOf: construct observed in source but outside detected Rl profile (not mapped to core)
```

Pizza reports `detected profile: Dl` with mapped-construct diagnostics. When no diagnostics: `diagnostics: none`.

### JSON

```json
{
  "detected": "EL",
  "diagnostics": [
    {
      "construct": "ObjectAllValuesFrom",
      "message": "..."
    }
  ]
}
```

`detected` is always present when profile detection succeeds on a loaded ontology.

## `classify` and `materialize` output

Both commands emit the same inference report fields; only `status` differs (`classified` vs `materialized`).

### Text

```text
status: materialized
initial_axiom_count: 57
final_axiom_count: 62
inferred_axioms: 5
inferred_by_rule:
  rng_inherit: 5
```

### JSON

```json
{
  "status": "materialized",
  "initial_axiom_count": 57,
  "final_axiom_count": 62,
  "inferred_axioms": 5,
  "inferred_by_rule": {
    "rng_inherit": 5
  }
}
```

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (parse failure, I/O, engine stub, etc.) |

Errors print to stderr: `error: ...`

## Examples

```bash
./benchmarks/scripts/download.sh

./target/release/ontologos profile benchmarks/data/pizza.owl
./target/release/ontologos --format json profile benchmarks/data/family.owl
./target/release/ontologos materialize benchmarks/data/family.owl
./target/release/ontologos classify benchmarks/data/family.owl  # status: classified + inference report
./target/release/ontologos --format json classify benchmarks/data/family.owl
```

## Related

- [Profile detection](../guides/profile-detection.md)
- [Load an OWL file](../getting-started/load-owl-file.md)
