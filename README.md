# OntoLogos

[![CI](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml)
[![Documentation](https://readthedocs.org/projects/ontologos/badge/?version=latest)](https://ontologos.readthedocs.io/en/latest/)
[![crates.io](https://img.shields.io/crates/v/ontologos-core.svg)](https://crates.io/crates/ontologos-core)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)

**OntoLogos** is native Rust ontology reasoning orchestration: load OWL files, detect profiles, and delegate to **reasonable** (RL/RDFS) and **horned-owl** (parsing) through stable facades. **OWL EL** uses the in-house completion engine in `ontologos-el`.

OntoLogos solves: *"We want OWL reasoning embedded in Rust or Python services—not a Java stack or ad-hoc script glue."*

Library-first orchestration: **load → detect profile → classify/materialize**, not a Protégé replacement.

**Install today:** `pip install ontologos` or `ontologos-core = "0.9.0"` — production-ready for **EL, RL, and RDFS**.

**OWL 2 DL and SWRL** ship in workspace **1.0.0** on `main` (build from source; [v1.0.0 tag](docs/project/release-1.0-checklist.md) pending). See [Install and channels](https://ontologos.readthedocs.io/en/latest/guides/install-channels.html).

> **Known limitations:** OntoLogos does not resolve `owl:imports` and maps a subset of OWL axioms. See [Known limitations](https://ontologos.readthedocs.io/en/latest/guides/known-limitations.html).

**In 30 seconds:** `pip install ontologos` or add `ontologos-parser = "0.9.0"` to `Cargo.toml` and load `family.owl`. **Requires Rust 1.88+** for library users — see [Prerequisites](https://ontologos.readthedocs.io/en/latest/guides/prerequisites.html).

> **Using OntoLogos in your app?** You do not need to clone this repo. Use crates.io / PyPI and follow the [5-minute guide](https://ontologos.readthedocs.io/en/latest/getting-started/). Clone only to contribute, run benchmarks, or build the CLI.

| | |
|---|---|
| **Published** | **v0.9.0** on crates.io / PyPI |
| **`main` workspace** | **1.0.0** pre-release · [CHANGELOG](CHANGELOG.md) |
| **crates.io** | [ontologos-core](https://crates.io/crates/ontologos-core) and siblings |
| **PyPI** | [`pip install ontologos`](https://pypi.org/project/ontologos/) |
| **Docs** | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) |
| **Rust** | [Prerequisites decision table](https://ontologos.readthedocs.io/en/latest/guides/prerequisites.html) |

---

## Choose your path

| Path | Start here |
|------|------------|
| **Not sure?** | [Start here](https://ontologos.readthedocs.io/en/latest/guides/start-here.html) on Read the Docs |
| **Rust (no clone)** | [5-minute crates.io guide](https://ontologos.readthedocs.io/en/latest/getting-started/#cratesio-only-no-clone) |
| **Python** | `pip install ontologos` → [Python guide](https://ontologos.readthedocs.io/en/latest/guides/python.html) |
| **CLI** *(requires git install — not on crates.io)* | `cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli` or clone → [CLI reference](https://ontologos.readthedocs.io/en/latest/reference/cli.html) |
| **Evaluate vs HermiT/ELK** | [Evaluator playbook](https://ontologos.readthedocs.io/en/latest/guides/evaluator-playbook.html) · [Comparison](https://ontologos.readthedocs.io/en/latest/comparison.html) |
| **Contribute** | Clone → [CONTRIBUTING](CONTRIBUTING.md) |

---

## Table of contents

- [Choose your path](#choose-your-path)
- [Why OntoLogos](#why-ontologos)
- [Which crates do I need?](#which-crates-do-i-need)
- [Features](#features)
- [Quick start](#quick-start)
- [How it works](#how-it-works)
- [Example](#example)
- [Packages](#packages)
- [Documentation](#documentation)
- [Development](#development)
- [License](#license)

---

## Why OntoLogos

| Audience | What you get |
|----------|--------------|
| **Rust application developers** | Composable crates, `Ontology` / `OntologyBuilder`, JSON snapshots, incremental sessions |
| **Python data pipelines** | `Reasoner`, `OntologyBuilder`, `explain()`, optional pandas/polars export |
| **OWL RL / RDFS workflows** | Forward-chaining via **reasonable** through `ontologos-rl` (`rdfs` and `abox` modules) |
| **OWL EL taxonomies** | In-house completion in `ontologos-el` (no Java) |
| **Early DL adopters** | `ontologos-dl` on `main`: **100%** catalog parity, full gated conformance @ 30s — PyPI still **0.9.0**; validate on your corpus |

---

## Which crates do I need?

| Goal | Install |
|------|---------|
| Core ontology graph | `ontologos-core` |
| Load `.owl` / `.ttl` | `ontologos-parser` |
| Profile detection | `ontologos-profile` |
| RDFS materialization | `ontologos-rl` (`rdfs` module) |
| OWL RL saturation | `ontologos-rl` |
| OWL EL taxonomy | `ontologos-el` |
| Multi-profile routing | `ontologos-facade` |
| Explanations | `ontologos-explain` |
| Taxonomy queries | `ontologos-ql` |
| CLI binary | Build `ontologos-cli` from this repo (not on crates.io) |
| Python | `pip install ontologos` |

See [Choosing an API](https://ontologos.readthedocs.io/en/latest/guides/choosing-an-api.html) for entry points. There is no umbrella `ontologos` meta-crate on crates.io.

---

## Features

| Area | Examples |
|------|----------|
| **Loading** | OWL Functional, RDF/XML, Turtle via horned-owl |
| **Profiles** | EL, RL, RDFS, QL detection; `auto` routing |
| **Reasoning** | RDFS materialize, RL saturate, EL classify, OWL DL (on `main` / v1.0.0) |
| **Incremental** | Session state for EL/RL/RDFS mutations |
| **Explain** | Proof graphs (EL full; RL/RDFS asserted-only) |
| **Interop** | JSON snapshot v2, bridge adapters, Python wheels |

Full construct matrix: [Supported constructs](https://ontologos.readthedocs.io/en/latest/reference/supported-constructs.html).

---

## Quick start

### Rust (crates.io, no clone)

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
cargo new ontologos-demo && cd ontologos-demo
```

`Cargo.toml`:

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
ontologos-rdfs = "0.9.0"
```

`src/main.rs`:

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ontology = load_ontology(std::path::Path::new("family.owl"))?;
    let report = RdfsEngine::new().materialize(&mut ontology)?;
    println!(
        "mapped {} → {} axioms (inferred {})",
        report.initial_axiom_count,
        report.final_axiom_count,
        report.inferred_total()
    );
    Ok(())
}
```

```bash
cargo run
```

OWL RL, profile detection, and more: [Getting started](https://ontologos.readthedocs.io/en/latest/getting-started/).

### Python

```bash
pip install ontologos
```

```python
from ontologos import Reasoner

report = Reasoner(path="family.owl", profile="rl").classify()
print(report)
```

Download `family.owl` first (see [Quick start](#rust-cratesio-no-clone)). See [Python guide](https://ontologos.readthedocs.io/en/latest/guides/python.html).

### Repository clone (CLI, benchmarks, tests)

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh
cargo build -p ontologos-cli --release
./target/release/ontologos classify --profile auto benchmarks/data/family.owl
```

---

## How it works

```
  Load (parser)          Profile detect          Reason
  ─────────────          ──────────────          ──────
  OWL / RDF file    →    EL / RL / RDFS / DL  →  Engine crate
  OntologyBuilder        ontologos-profile       (rdfs / rl / el / dl)
  JSON snapshot v2                             →  Taxonomy + reports
```

```
  Author (Rust / Python / CLI)
           │
           ▼
    load_ontology / Reasoner
           │
           ▼
    detect_profile (optional)
           │
           ▼
    classify / materialize / explain
           │
           ▼
    Taxonomy, proofs, JSON output

```

At runtime, **`ontologos-facade`** routes `classify` by profile. Use **`materialize`** for explicit RDFS (same engine as `classify --profile rdfs`).

---

## Example

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_facade::ClassifyOutcome;
use ontologos_parser::load_ontology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ontology = load_ontology("family.owl".as_ref())?;
    let reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(ontology)?;

    // Production consistency: always inspect `complete`
    let consistency = ontologos_facade::check_consistency(&reasoner)?;
    if consistency.complete && consistency.consistent {
        let mut reasoner = reasoner;
        match ontologos_facade::classify(&mut reasoner)? {
            ClassifyOutcome::Taxonomy(t) => {
                println!("EL/DL taxonomy: {} subsumptions", t.subsumption_count());
            }
            ClassifyOutcome::Rdfs(r) => {
                println!("RDFS inferred: {}", r.inferred_total());
            }
            ClassifyOutcome::Rl(r) => {
                println!("RL inferred: {}", r.inferred_total());
            }
        }
    }
    Ok(())
}
```

`Cargo.toml` dependencies:

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
ontologos-facade = "0.9.0"
```

On crates.io the latest **tagged** release is **0.9.0**. Build from `main` with `"1.0.0"` pins for DL and the full **12-crate** library set — see [Release status](docs/project/release-status.md).

Do **not** call `ontologos_core::Reasoner::classify()` or `Reasoner::is_consistent()` — use `ontologos_facade::check_consistency` and `ontologos_facade::classify`. See [Facade API](https://ontologos.readthedocs.io/en/latest/guides/facade-api.html).

---

## Upgrading from an older version

See [Migration hub](https://ontologos.readthedocs.io/en/latest/migration/) for guides by version (including [v0.8 → v1.0](https://ontologos.readthedocs.io/en/latest/migration/v0.8.x-to-v1.0.0.html)).

---

## Packages

| Crate | crates.io | Description |
|-------|-----------|-------------|
| `ontologos-core` | [yes](https://crates.io/crates/ontologos-core) | Data model, builder, JSON v3 (v2 read) |
| `ontologos-parser` | [yes](https://crates.io/crates/ontologos-parser) | OWL/RDF loading |
| `ontologos-profile` | [yes](https://crates.io/crates/ontologos-profile) | Profile detection |
| `ontologos-rl` | [yes](https://crates.io/crates/ontologos-rl) | OWL RL + RDFS + ABox (reasonable) |
| `ontologos-el` | [yes](https://crates.io/crates/ontologos-el) | OWL EL completion |
| `ontologos-explain` | [yes](https://crates.io/crates/ontologos-explain) | Proof graphs |
| `ontologos-facade` | [yes](https://crates.io/crates/ontologos-facade) | Unified classify routing |
| `ontologos-bridge` | [yes](https://crates.io/crates/ontologos-bridge) | horned-owl / reasonable adapters |
| `ontologos-alc` | [yes](https://crates.io/crates/ontologos-alc) | ALC tableau-lite (preview) |
| `ontologos-dl` | [yes](https://crates.io/crates/ontologos-dl) | OWL 2 DL |
| `ontologos-swrl` | [yes](https://crates.io/crates/ontologos-swrl) | DLSafe SWRL + DL |
| `ontologos-ql` | [yes](https://crates.io/crates/ontologos-ql) | OWL QL queries + taxonomy hierarchy |
| `ontologos-cli` | source only | CLI binary — `cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli` |
| `ontologos-py` | [PyPI](https://pypi.org/project/ontologos/) | Python bindings |

---

## Documentation

Full site: **[ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/)**

| Topic | Link |
|-------|------|
| Start here | [Persona paths](https://ontologos.readthedocs.io/en/latest/guides/start-here.html) |
| Examples | [Examples gallery](https://ontologos.readthedocs.io/en/latest/examples/) |
| Prerequisites | [Rust / Python / clone](https://ontologos.readthedocs.io/en/latest/guides/prerequisites.html) |
| Choosing an API | [Crate picker](https://ontologos.readthedocs.io/en/latest/guides/choosing-an-api.html) |
| CLI | [CLI reference](https://ontologos.readthedocs.io/en/latest/reference/cli.html) |
| Python | [Python guide](https://ontologos.readthedocs.io/en/latest/guides/python.html) |
| Errors | [Error reference](https://ontologos.readthedocs.io/en/latest/reference/errors.html) |
| Conformance | [HermiT-ported tests](https://ontologos.readthedocs.io/en/latest/reference/conformance.html) · [Burndown guide](https://ontologos.readthedocs.io/en/latest/guides/hermit-burndown.html) (contributors) |
| Migration | [Upgrade hub](https://ontologos.readthedocs.io/en/latest/migration/) |

Source markdown: `docs/` · Changelog: [CHANGELOG.md](CHANGELOG.md) · Security: [docs/project/security-policy.md](docs/project/security-policy.md)

---

## Development

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh
cargo test --workspace --exclude ontologos-conformance
cargo clippy --workspace --all-targets -- -D warnings
```

**Working on HermiT parity (v1.0)?** Start with the [HermiT burndown guide](docs/guides/hermit-burndown.md) and `bash benchmarks/scripts/hermit-burndown.sh status`.

| Change type | Usually enough |
|-------------|----------------|
| `docs/` only | `./docs/build-site.sh` |
| Single crate | `cargo test -p ontologos-el` |
| Full CI parity | `cargo test --workspace` + conformance release build |

Contributors: [CONTRIBUTING.md](CONTRIBUTING.md) · [Architecture](docs/architecture.md) · [Roadmap summary](docs/project/roadmap-summary.md)

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
