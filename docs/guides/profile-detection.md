# Profile Detection

Detect which OWL 2 profile best fits an ontology using [`ontologos-profile`](https://docs.rs/ontologos-profile/0.4.0).

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

- **Detected:** `Dl` (mapped axioms mix EL and RL shapes — e.g. existentials plus inverse/functional properties)
- **Diagnostics:** list mapped constructs that rule out EL and/or RL, plus any source-only constructs not stored in core

Classification reflects what the reasoner can use from the core model; diagnostics explain profile boundaries.

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
  "detected": "DL",
  "diagnostics": [
    {
      "construct": "InverseObjectProperties",
      "message": "construct is outside OWL 2 EL (mapped axiom rules out OWL 2 EL)"
    },
    {
      "construct": "SubClassOfExistential",
      "message": "construct is outside OWL 2 RL (mapped axiom rules out OWL 2 RL)"
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
- [ROADMAP](../../ROADMAP.md) — reasoning engines and milestones
