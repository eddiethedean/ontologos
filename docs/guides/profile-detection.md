# Profile Detection

v0.2 reports which OWL 2 profile best fits an ontology using [`ontologos-profile`](https://docs.rs/ontologos-profile).

## CLI

```bash
./benchmarks/scripts/download.sh
cargo build -p ontologos-cli --release

# Text output
./target/release/ontologos profile benchmarks/data/pizza.owl

# JSON output
./target/release/ontologos --format json profile benchmarks/data/family.owl
```

## Library

```rust
use ontologos_parser::load_ontology;
use ontologos_profile::{detect_profile, OwlProfile};

let ontology = load_ontology(path)?;
let report = detect_profile(&ontology)?;

if let Some(OwlProfile::El) = report.detected {
    println!("EL ontology");
}

for diag in &report.diagnostics {
    println!("{}: {}", diag.construct, diag.message);
}
```

## Hybrid contract

Profile detection uses two construct sets from `ParseMeta`:

| Set | Source | Used for |
|-----|--------|----------|
| `profile_constructs` | Successfully mapped TBox axioms | **Detected profile** (EL / RL / QL / DL) |
| `constructs` | Full parse scan | **Diagnostics** for source-only constructs |

**Example — Pizza ontology:**

- **Detected:** `El` (mapped axioms fit EL)
- **Diagnostics:** may list `ObjectAllValuesFrom`, `ObjectUnionOf`, etc. observed in the source but not mapped into core

This is intentional: classification reflects what the reasoner can use from the core model; diagnostics flag constructs still present in the file.

## Profiles

| Profile | Meaning |
|---------|---------|
| `El` | OWL 2 EL — polynomial EL classification target (v0.5) |
| `Rl` | OWL 2 RL — rule-based materialization target (v0.4) |
| `Ql` | OWL 2 QL — query language profile |
| `Dl` | Outside EL/RL/QL — full DL fallback |

## JSON output shape

`ontologos --format json profile file.owl` emits:

```json
{
  "detected": "EL",
  "diagnostics": [
    {
      "construct": "ObjectAllValuesFrom",
      "message": "construct observed in source but outside detected El profile (not mapped to core)"
    }
  ]
}
```

`detected` uses uppercase serde names (`EL`, `RL`, `QL`, `DL`). Text CLI output uses `El`, `Rl`, etc.

## Benchmark corpora

| Corpus | Expected profile | Notes |
|--------|------------------|-------|
| Pizza | `Dl` | Mapped inverse/functional axioms; DL constructs in source |
| Family | `Rl` | RL property axioms mapped; some source-only diagnostics possible |

Corpus files: run `./benchmarks/scripts/download.sh`. Expected counts are **mapper output**, not Protégé logical axiom totals.

## Next steps

- [Supported constructs](../reference/supported-constructs.md)
- [Troubleshooting](troubleshooting.md)
- [ROADMAP](../../ROADMAP.md) — reasoning engines (v0.3+)
