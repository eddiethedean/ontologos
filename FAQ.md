# Frequently Asked Questions

## Why does `Ontology::from_file` fail?

`Ontology::from_file` on `ontologos-core` intentionally returns `Error::ParseNotAvailable` to keep the core crate free of parser dependencies.

**Load OWL/RDF files with `ontologos-parser`:**

```rust
use ontologos_parser::load_ontology;

let ontology = load_ontology(path::Path::new("ontology.owl"))?;
```

Or use the CLI: `ontologos profile ontology.owl`

See [Load an OWL file](https://ontologos.readthedocs.io/en/latest/getting-started/load-owl-file/).

## Which crate should I depend on?

Pin all `ontologos-*` crates to the **same version**. Check [Release status](https://ontologos.readthedocs.io/en/latest/project/release-status/) for the current crates.io version (**1.1.4**).

```toml
[dependencies]
ontologos-core = "1.1.4"
ontologos-parser = "1.1.4"   # OWL/RDF file loading
ontologos-profile = "1.1.4"  # EL / RL / QL / DL detection
ontologos-rl = "1.1.4"       # OWL RL saturation + RDFS (`ontologos_rl::rdfs`)
ontologos-el = "1.1.4"       # OWL EL classification
ontologos-explain = "1.1.4"  # Proof graphs
ontologos-ql = "1.1.4"        # Taxonomy queries and OWL QL
ontologos-facade = "1.1.4"   # Unified classify routing
ontologos-bridge = "1.1.4"   # Engine adapters (usually transitive)
```

Depend on **`ontologos-core` only** if you build ontologies programmatically or from JSON snapshots.

There is no umbrella `ontologos` crate on crates.io. The CLI binary is built from `ontologos-cli` in this repository.

## Can I use OntoLogos instead of Protégé + HermiT today?

**Published v1.1.4** — `ontologos-dl` passes the HermiT Tier A catalog (**450** runnable Java + **428** OWL WG cases) and Tier B/C classification gates at a 30s per-operation budget. That is **HermiT functional parity on the gated conformance corpora** (`parity_pct = 100%` on **889 in-scope cases**), not a guarantee for every real-world ontology. Composite **`true_parity_pct`** is **100%** (blocking CI). See [Evaluator scope](https://ontologos.readthedocs.io/en/latest/guides/evaluator-scope/) for what each metric measures. For ontologies within the [supported construct](https://ontologos.readthedocs.io/en/latest/reference/supported-constructs/) subset, use `classify --profile dl` (or `profile="dl"` in Python). Set `ONTOLOGOS_DL_BUDGET_SECS` if you need longer wall-clock limits. Outside the gated suite, validate results against HermiT/Konclude until you trust the engine on your corpus.

OntoLogos is for adopters who want to embed the Rust data model, load ontologies natively, run RL saturation, or follow the [roadmap](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md).

## What is the difference between `parity_pct` and `true_parity_pct`?

Two metrics — do not conflate them:

| Metric | Meaning | Current on `main` |
|--------|---------|-------------------|
| **`parity_pct`** | In-scope HermiT catalog harness complete (zero `planned` cases) | **100%** (**889** cases) |
| **`true_parity_pct`** | Composite everyday HermiT equivalence: minimum of literal catalog green, strict taxonomy, perf, internal ports, and SWRL rules | **100%** |

Check live values: `bash benchmarks/scripts/hermit-burndown.sh status`

- **`parity_pct = 100%`** is the **v1.0 engineering gate** — blocking in CI via `check-hermit-parity-phases.sh`.
- **`true_parity_pct`** reached **100%** on the composite burndown metric. CI runs `check-true-parity-gate.sh` **blocking** @ 100%. Details: [Evaluator scope](https://ontologos.readthedocs.io/en/latest/guides/evaluator-scope/).

See [Evaluator scope](https://ontologos.readthedocs.io/en/latest/guides/evaluator-scope/) for the full framing.

## Why was my JSON rejected?

Common causes:

- **`format_version: 1`** — v1 is rejected for untrusted input; use [JSON v2](https://ontologos.readthedocs.io/en/latest/json-snapshot-v2/)
- **Invalid IRI** — only `http`, `https`, and `urn` schemes; no control characters
- **Unknown entity IRI in axioms** — declare all entities before referencing them in axioms
- **Size limits** — default max JSON size is 16 MiB; see [Security](https://ontologos.readthedocs.io/en/latest/security/)

## Why does Pizza detect as DL?

Profile **classification** uses mapped TBox shapes (`parse_meta.profile_constructs`). The Pizza corpus mixes EL shapes (existentials) with constructs that rule out EL and RL (e.g. inverse and functional object properties), so detection reports **DL**. **Diagnostics** explain which mapped constructs violate EL/RL profile rules and may also list constructs seen in the source but not stored in core (e.g. `ObjectAllValuesFrom`).

See [Profile detection](https://ontologos.readthedocs.io/en/latest/guides/profile-detection/).

## Why doesn't `ontology.axiom_count()` match Protégé's axiom count?

The parser maps a subset of OWL constructs into the core model. Complex class expressions, many data-property axioms, and some property axioms are scanned for profile detection but **skipped** during mapping. Named ABox axioms (`ClassAssertion`, `ObjectPropertyAssertion`, `SameIndividual`, `DifferentIndividuals`) are mapped. `axiom_count()` is **mapper output**, not raw OWL logical axiom count.

See [Protégé vs OntoLogos counts](https://ontologos.readthedocs.io/en/latest/guides/protege-axiom-counts/), [Troubleshooting](https://ontologos.readthedocs.io/en/latest/guides/troubleshooting/), and [Supported constructs](https://ontologos.readthedocs.io/en/latest/reference/supported-constructs/).

## What is the difference between ROADMAP and PLAN.md?

**[docs/internal/roadmap.md](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/roadmap.md)** is the canonical semver release plan. **[PLAN.md](https://github.com/eddiethedean/ontologos/blob/main/PLAN.md)** is historical background and ecosystem vision; prefer the internal roadmap for current status. The root [ROADMAP.md](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) is a short pointer.

## Is OntoLogos the same as Ontologos?

**Display name:** OntoLogos. **Crate and command names:** `ontologos-*` and `ontologos` (lowercase).

## How do I load the pizza test fixture?

**From JSON (no download):**

```rust
let json = include_str!("../tests/fixtures/pizza_minimal.json");
let ontology = Ontology::from_json(json)?;
```

**From OWL (benchmark corpus):**

```bash
./benchmarks/scripts/download.sh
```

```rust
let ontology = ontologos_parser::load_ontology(path::Path::new("benchmarks/data/pizza.owl"))?;
```

Or run `cargo run -p ontologos-core --example pizza_builder`.

## Where is the API reference?

- Hosted: [docs.rs/ontologos-core](https://docs.rs/ontologos-core/1.1.4), [docs.rs/ontologos-parser](https://docs.rs/ontologos-parser/1.1.4), [docs.rs/ontologos-profile](https://docs.rs/ontologos-profile/1.1.4), [docs.rs/ontologos-rl](https://docs.rs/ontologos-rl/1.1.4), [docs.rs/ontologos-el](https://docs.rs/ontologos-el/1.1.4), [docs.rs/ontologos-explain](https://docs.rs/ontologos-explain/1.1.4), [docs.rs/ontologos-ql](https://docs.rs/ontologos-ql/1.1.4), [docs.rs/ontologos-facade](https://docs.rs/ontologos-facade/1.1.4)
- Site reference: [Explain API](https://ontologos.readthedocs.io/en/latest/reference/explain/) · [Query API](https://ontologos.readthedocs.io/en/latest/reference/query/) · [CLI](https://ontologos.readthedocs.io/en/latest/reference/cli/)
- Guides: [Choosing an API](https://ontologos.readthedocs.io/en/latest/guides/choosing-an-api/) · [Architecture](https://ontologos.readthedocs.io/en/latest/architecture/)
- Local: `cargo doc -p ontologos-core --open`
- Error catalog: [Error reference](https://ontologos.readthedocs.io/en/latest/reference/errors/)

## Does `pip install ontologos` work?

Yes. The PyPI package is **v1.1.4**. It supports file and in-memory ontologies, incremental mutations, `explain()`, and optional pandas/polars export.

**Profiles on PyPI:** `"auto"`, `"el"`, `"rl"`, `"rdfs"`, `"dl"`, and `"swrl"`.

**Preview only:** `"alc"` and `"dl-preview"`. See [Install and channels](https://ontologos.readthedocs.io/en/latest/guides/install-channels/).

```python
from ontologos import Reasoner, OntologyBuilder

Reasoner(path="family.owl", profile="auto").classify()
Reasoner(path="family.owl", profile="dl").classify()
```

Optional extras: `pip install 'ontologos[pandas]'` or `'ontologos[polars]'`.

See [Python guide](https://ontologos.readthedocs.io/en/latest/guides/python/) and [v0.8→v0.9 migration](https://ontologos.readthedocs.io/en/latest/migration/v0.8.x-to-v0.9.0/).

## When should I use `OntologyBuilder` vs loading a file?

Use **`Reasoner(path=...)`** when you have an OWL/RDF file on disk.

Use **`OntologyBuilder`** or **`Ontology.from_dict`** when constructing ontologies in memory (tests, pipelines, incremental edit workflows) without a parser round-trip. Pass the result to `Reasoner(ontology=..., profile=...)`.

## What does `explain()` cover?

| Profile | Coverage |
|---------|----------|
| **EL** | Full inference traces → proof graph with IRI-resolved conclusions |
| **RL / RDFS** | Proof graph seeds **asserted** axioms; inferred steps lack per-rule premises until reasonable exposes a trace API |
| **auto** | Routes like `classify`; DL-detected ontologies use DL preview |

See [Explain API reference](https://ontologos.readthedocs.io/en/latest/reference/explain/).

## How does incremental reasoning work in Python?

Pass `incremental=True` to `Reasoner`, call `classify()`, then mutate the ontology (`add_subclass_of`, `remove_subclass_of`, `add_axiom_json`), and call `classify()` again. Each pass reuses the session when the delta is small.

See [Incremental reasoning guide](https://ontologos.readthedocs.io/en/latest/guides/incremental-reasoning/).

## Is the Python `Reasoner` thread-safe?

No. Each `Reasoner` instance should be used from one thread at a time. Create separate instances per worker or guard access with your own synchronization.

## Why does `Reasoner::classify()` on core return `NotImplemented`?

`ontologos_core::Reasoner::classify()` is a facade stub: it returns delegate hints for RDFS/RL and `NotImplemented` for EL. Use **CLI** (`ontologos classify`), **Python** (`Reasoner.classify()`), **`ontologos_facade::classify`**, or profile crates directly (`ElClassifier`, `RlEngine`, `RdfsEngine`).

## Why are axioms missing after I load an OWL file?

**Remote `owl:imports` are never fetched.** RDF/XML (`.owl`, `.rdf`, `.xml`) merges **local** `owl:imports` when using `load_ontology()`. Turtle and OWL Functional load only the file you specify.

**Workaround for remote or multi-format bundles:** merge with [ROBOT](http://robot.obolibrary.org/) (`robot merge --input ontology.owl --output merged.owl`) or OWL API, then load the merged file. See [OWL imports](https://ontologos.readthedocs.io/en/latest/reference/owl-imports.html) and [Load an OWL file](https://ontologos.readthedocs.io/en/latest/getting-started/load-owl-file/).

## Which version should I `cargo add` or `pip install`?

| Channel | Version | When |
|---------|---------|------|
| crates.io / PyPI (production) | **1.1.4** | Default for `cargo add` and `pip install ontologos` |
| Prior release | **1.0.0** | See [v1.0.x → v1.1.0 migration](https://ontologos.readthedocs.io/en/latest/migration/v1.0.x-to-v1.1.0/) |

See [Release status](https://ontologos.readthedocs.io/en/latest/project/release-status/).

## Where do I ask questions?

Open a [GitHub issue](https://github.com/eddiethedean/ontologos/issues) for bugs, feature requests, or design questions. Check this FAQ and [Troubleshooting](https://ontologos.readthedocs.io/en/latest/guides/troubleshooting/) first.
