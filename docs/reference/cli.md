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

| Command | v0.2 status | Description |
|---------|-------------|-------------|
| `profile <file>` | **Works** | Detect OWL profile (EL/RL/QL/DL) |
| `classify <file>` | Stub | Loads file; `Reasoner::classify` → `NotImplemented` |
| `materialize <file>` | Stub | Loads file; RDFS engine → `NotImplemented` |
| `explain <file>` | Stub | Loads file; explain → `NotImplemented` |

All commands load the ontology via `ontologos_parser::load_ontology` first.

## `profile` output

### Text

```text
detected profile: El
diagnostics:
  - ObjectAllValuesFrom: construct observed in source but outside detected El profile (not mapped to core)
```

When no diagnostics: `diagnostics: none`.

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

`detected` is `null` if detection fails (rare).

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
./target/release/ontologos classify benchmarks/data/pizza.owl   # exits 1 (NotImplemented)
```

## Related

- [Profile detection](../guides/profile-detection.md)
- [Load an OWL file](../getting-started/load-owl-file.md)
