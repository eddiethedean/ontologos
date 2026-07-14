# OntoLogos

[![CI](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml)
[![Documentation](https://readthedocs.org/projects/ontologos/badge/?version=latest)](https://ontologos.readthedocs.io/en/latest/)
[![crates.io](https://img.shields.io/crates/v/ontologos-core.svg)](https://crates.io/crates/ontologos-core)
[![Wasmer](https://img.shields.io/badge/Wasmer-1.1.4-4946E5?logo=wasmer&logoColor=white)](https://wasmer.io/eddiethedean/ontologos)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)

**Embed OWL reasoning in Rust and Python today — no JVM.** WASM ships on [Wasmer](https://wasmer.io/eddiethedean/ontologos); Node.js, Java, .NET, and C/C++ via [source build](https://ontologos.readthedocs.io/en/latest/guides/bindings-overview.html).

Load `.owl` files, detect EL/RL/DL profiles, and classify or materialize in-process.
Built for services and data pipelines, not as a Protégé replacement.

| Try now | Command |
|---------|---------|
| **Python** | `pip install ontologos` |
| **Rust** | Add `ontologos-parser`, `ontologos-facade` @ `1.1.4` to `Cargo.toml` |
| **CLI** | `cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.4 ontologos-cli` |
| **WASM** | [wasmer.io/eddiethedean/ontologos](https://wasmer.io/eddiethedean/ontologos) ([WASM guide](https://ontologos.readthedocs.io/en/latest/guides/wasm.html)) |
| **Node.js** | Build `crates/ontologos-node` ([Bindings overview](https://ontologos.readthedocs.io/en/latest/guides/bindings-overview.html)) |
| **Java / .NET / C/C++** | Source-build ([Bindings overview](https://ontologos.readthedocs.io/en/latest/guides/bindings-overview.html)) |
| **Evaluate** | [30-minute playbook](https://ontologos.readthedocs.io/en/latest/guides/evaluator-playbook.html) |

**Requires Rust 1.88+** for library users · **Full docs:** [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/)

**v1.1.4** on [crates.io](https://crates.io/crates/ontologos-core), [PyPI](https://pypi.org/project/ontologos/), and [Wasmer](https://wasmer.io/eddiethedean/ontologos). Install channels: [guide](https://ontologos.readthedocs.io/en/latest/guides/install-channels.html).

> **OWL DL:** passes the gated HermiT conformance catalog (889 in-scope cases) — not a guarantee for every ontology. [Validate on your corpus](https://ontologos.readthedocs.io/en/latest/guides/evaluator-scope.html) before production.

> **Known limitations:** Remote `owl:imports` are never fetched; OntoLogos maps a subset of OWL axioms. See [Known limitations](https://ontologos.readthedocs.io/en/latest/guides/known-limitations.html).

---

## Choose your path

| Path | Start here |
|------|------------|
| **Not sure?** | [Start here](https://ontologos.readthedocs.io/en/latest/guides/start-here.html) |
| **Rust (no clone)** | [5-minute guide](https://ontologos.readthedocs.io/en/latest/getting-started/#cratesio-only-no-clone) |
| **Python** | [Python guide](https://ontologos.readthedocs.io/en/latest/guides/python.html) |
| **Bindings (Node, Java, .NET, C)** | [Bindings overview](https://ontologos.readthedocs.io/en/latest/guides/bindings-overview.html) |
| **WASM** | [Wasmer package](https://wasmer.io/eddiethedean/ontologos) · [WASM guide](https://ontologos.readthedocs.io/en/latest/guides/wasm.html) |
| **CLI** | [CLI installation](https://ontologos.readthedocs.io/en/latest/getting-started/cli-install.html) |
| **Evaluate vs HermiT/ELK** | [Evaluator playbook](https://ontologos.readthedocs.io/en/latest/guides/evaluator-playbook.html) · [Evaluator scope](https://ontologos.readthedocs.io/en/latest/guides/evaluator-scope.html) |
| **Contribute** | [CONTRIBUTING](CONTRIBUTING.md) |

You do **not** need to clone this repo to use OntoLogos from crates.io, PyPI, or [Wasmer](https://wasmer.io/eddiethedean/ontologos) (browser JS glue still builds from source).

---

## Quick start (Python)

```bash
pip install ontologos
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
```

```python
from ontologos import Reasoner

report = Reasoner(path="family.owl", profile="rl").classify()
print(report)
```

Rust, CLI, and profile-specific guides: [Getting started](https://ontologos.readthedocs.io/en/latest/getting-started/).

---

## Capabilities (v1.1.4)

| Profile | Use case | Status |
|---------|----------|--------|
| **RDFS** | TBox materialization | Stable |
| **OWL RL** | Forward-chaining saturation | Stable |
| **OWL EL** | Taxonomy / subsumption | Stable |
| **OWL 2 DL** | Full DL classification | Stable — [scope limits](https://ontologos.readthedocs.io/en/latest/guides/evaluator-scope.html) |
| **SWRL** | DLSafe rules + DL | Stable |

Preview only: `alc`, `dl-preview`. See [Profile stability](https://ontologos.readthedocs.io/en/latest/guides/profile-stability.html).

---

## Rust integration

Load with `ontologos_parser::load_ontology`. Classify with `ontologos_facade::classify` — not `Reasoner::classify()` on core.

See the [Rust integration contract](https://ontologos.readthedocs.io/en/latest/guides/rust-integration-contract.html) and [Choosing an API](https://ontologos.readthedocs.io/en/latest/guides/choosing-an-api.html).

---

## Documentation

| Topic | Link |
|-------|------|
| **Full site** | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) |
| Crate picker | [Choosing an API](https://ontologos.readthedocs.io/en/latest/guides/choosing-an-api.html) |
| Packages (12 library crates + bindings) | [Reference index](https://ontologos.readthedocs.io/en/latest/reference/) |
| Migration | [Upgrade hub](https://ontologos.readthedocs.io/en/latest/migration/) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) |

---

## Development

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh
cargo test --workspace --exclude ontologos-conformance
```

Contributors: [CONTRIBUTING.md](CONTRIBUTING.md) · [Architecture](docs/architecture.md) · [HermiT burndown](docs/guides/hermit-burndown.md)

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
