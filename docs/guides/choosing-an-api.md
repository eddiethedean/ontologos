# Choosing an API

OntoLogos exposes multiple entry points. This guide picks the right crate and function for your workflow.

## Decision tree

```mermaid
flowchart TD
  start[What_do_you_need]
  start --> buildCode[Build_ontology_in_code]
  start --> loadFile[Load_OWL_file]
  start --> jsonSnap[Load_JSON_snapshot]
  start --> reasonOnly[Reason_only]

  buildCode --> builder[Ontology_builder]
  jsonSnap --> fromJson[Ontology_from_json]
  loadFile --> loadOnt[ontologos_parser_load_ontology]

  builder --> goal{Reasoning_goal}
  fromJson --> goal
  loadOnt --> goal
  reasonOnly --> goal

  goal --> none[No_reasoning_yet]
  goal --> profile[Detect_profile]
  goal --> rdfsGoal[RDFS_TBox]
  goal --> rlGoal[OWL_RL]
  goal --> elGoal[OWL_EL_taxonomy]

  none --> coreOnly[ontologos_core_only]
  profile --> detect[ontologos_profile_detect_profile]
  rdfsGoal --> rdfsEng[RdfsEngine_materialize]
  rlGoal --> rlEng[RlEngine_saturate]
  elGoal --> elStub[Not_available_v0_5]
```

## By task

### Programmatic ontology construction

**Crate:** `ontologos-core` only

```rust
use ontologos_core::Ontology;

let ontology = Ontology::builder()
    .class("http://example.org/A")?
    .subclass_of("http://example.org/A", "http://example.org/B")?
    .build()?;
```

See [First ontology](../getting-started/first-ontology.md).

### JSON snapshot round-trip

**Crate:** `ontologos-core` only

```rust
let json = ontology.to_json()?;
let restored = Ontology::from_json(&json)?;
```

See [JSON snapshot v2](../json-snapshot-v2.md). Use `from_json_with_limits` for untrusted input.

### Load OWL/RDF files

**Crates:** `ontologos-parser` (+ `ontologos-core`)

```rust
use ontologos_parser::load_ontology;

let ontology = load_ontology(path)?;
```

Do **not** use `Ontology::from_file` — it returns `ParseNotAvailable` by design.

See [Load an OWL file](../getting-started/load-owl-file.md).

### Profile detection

**Crates:** `ontologos-parser`, `ontologos-profile`

```rust
use ontologos_parser::load_ontology;
use ontologos_profile::detect_profile;

let ontology = load_ontology(path)?;
let report = detect_profile(&ontology)?;
```

Or CLI: `ontologos profile file.owl`

### RDFS TBox materialization

**Crates:** `ontologos-rdfs` (+ parser if loading files)

**Direct:**

```rust
use ontologos_rdfs::RdfsEngine;

let report = RdfsEngine::new().materialize(&mut ontology)?;
```

**Via reasoner facade:**

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_rdfs::classify_reasoner;

let mut reasoner = Reasoner::builder().profile(Profile::Rdfs).build(ontology)?;
classify_reasoner(&mut reasoner)?;
```

CLI: `ontologos materialize` (RDFS) or `ontologos classify --profile rdfs|rl|el|auto`.

See [RDFS materialization](../getting-started/rdfs-materialization.md).

### OWL RL saturation

**Crates:** `ontologos-rl` (+ parser if loading files)

**Direct (recommended for v0.4):**

```rust
use ontologos_rl::RlEngine;

let report = RlEngine::new(1).saturate(&mut ontology)?;
```

**Via reasoner facade:**

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_rl::classify_reasoner;

let mut reasoner = Reasoner::builder().profile(Profile::Rl).build(ontology)?;
classify_reasoner(&mut reasoner)?;
```

Do **not** call `Reasoner::classify()` on core for RL — it returns a delegate hint.

CLI: `ontologos classify --profile rl`. Python: `Reasoner(path, profile="rl").classify()`.

See [OWL RL saturation](../getting-started/owl-rl-saturation.md).

### OWL EL taxonomy classification

**Crates:** `ontologos-el`, `ontologos-query` (+ parser if loading files)

```rust
use ontologos_el::ElClassifier;

let taxonomy = ElClassifier::new().classify(&ontology)?;
```

**Routed classification:**

```rust
use ontologos_el::classify_with_profile;

let outcome = classify_with_profile(&mut reasoner)?;
```

CLI: `ontologos classify --profile el`. Python: `Reasoner(path, profile="el").classify()`.

See [OWL EL classification](../getting-started/owl-el-classification.md).

### Python

**Package:** `pip install ontologos` (alpha)

```python
from ontologos import Reasoner

Reasoner("file.owl", profile="rdfs").classify()
Reasoner("file.owl", profile="rl").classify()
Reasoner("file.owl", profile="el").classify()
Reasoner("file.owl", profile="auto").classify()
```

See [Python guide](python.md).

## Dependency cheat sheet

| Workflow | Minimum dependencies |
|----------|---------------------|
| Builder / JSON only | `ontologos-core` |
| Load OWL files | `ontologos-core`, `ontologos-parser` |
| + Profile detection | `+ ontologos-profile` |
| + RDFS | `+ ontologos-rdfs` |
| + OWL RL | `+ ontologos-rl` (pulls in rdfs transitively) |
| + OWL EL + queries | `+ ontologos-el`, `+ ontologos-query` |

There is no single `ontologos` meta-crate on crates.io.

## Common mistakes

| Mistake | Fix |
|---------|-----|
| `Ontology::from_file` | Use `ontologos_parser::load_ontology` |
| `Reasoner::classify()` for RL/RDFS | Use profile crate helpers |
| Expect CLI `classify` to run DL | Use EL/RL/RDFS profiles; full DL is v2.0 |
| Compare axiom count to Protégé | See [supported constructs](../reference/supported-constructs.md) |
| `Profile::Auto` on core reasoner | Detect profile explicitly, pick engine |

## Related

- [Architecture](../architecture.md)
- [Error reference](../reference/errors.md)
- [FAQ](../project/faq.md)
