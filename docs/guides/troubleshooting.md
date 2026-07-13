# Troubleshooting

## `rustc` version too old

**Symptom:** `error: edition 2024 is unstable` or `package requires Rust 1.88`.

**Fix:**

```bash
rustup update stable
rustc --version   # must be >= 1.88.0
```

See [Prerequisites](prerequisites.md).

## `command not found: ontologos`

The CLI is **not on crates.io**. See [CLI installation](../getting-started/cli-install.md).

```bash
cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.3 ontologos-cli
```

Or build from a clone: `cargo build -p ontologos-cli --release` → `./target/release/ontologos`.

## `cargo test` fails on Pizza / Family tests

**Symptom:** `missing benchmark corpus pizza at ...`

**Fix:** Download corpora before testing:

```bash
./benchmarks/scripts/download.sh
cargo test --workspace
```

CI runs `download.sh` automatically. See [benchmarks](../project/benchmarks.md).

## `ontologos profile` fails: file not found

Pizza is not committed to the repo (gitignored). Fetch it:

--8<-- "snippets/pizza-owl-download.md"

Or use `benchmarks/data/family.owl` (vendored).

## Axiom count does not match Protégé

See the dedicated walkthrough: [Protégé vs OntoLogos axiom counts](protege-axiom-counts.md).

`ontology.axiom_count()` is the number of axioms **stored in core** after mapping, not the raw OWL logical axiom count in the source file.

The parser skips complex class expressions, many data-property axioms, and some property axiom shapes. Named ABox axioms (`ClassAssertion`, `ObjectPropertyAssertion`, `SameIndividual`, `DifferentIndividuals`) are mapped in v0.4. Skipped items appear in `parse_meta.skipped_axiom_count` and `parse_meta.warnings`.

Benchmark manifest values (e.g. Pizza `658`) are mapper output targets, documented in [benchmarks](../project/benchmarks.md).

## Pizza is DL with mapped-construct diagnostics

Expected for the current Pizza corpus: mapped axioms include both EL shapes (existentials) and constructs outside EL (e.g. inverse properties), so detection reports **DL** with diagnostics explaining which mapped constructs rule out EL/RL. Source-only constructs may also appear under the [hybrid profile contract](profile-detection.md).

## `Ontology::from_file` returns `ParseNotAvailable`

Use `ontologos_parser::load_ontology`. See [FAQ](../project/faq.md).

## JSON `from_json` fails

Common causes: `format_version: 1`, invalid IRI, unknown entity in axiom, size limit. See [errors.md](../reference/errors.md) and [json-snapshot-v3.md](../json-snapshot-v3.md).

## Unsupported file extension

`load_ontology` returns `Error::UnsupportedFormat` for unrecognized extensions. Supported: `.owl`, `.rdf`, `.xml`, `.ttl`, `.turtle`, `.ofn`, `.func`.

## Parser warnings after load

Inspect `ontology.parse_meta().warnings`. Warnings are non-fatal; the ontology loads with whatever axioms could be mapped.

## Missing axioms from imported ontologies

Import behavior is **format-dependent**. RDF/XML merges local `owl:imports` by default; Turtle and OWL Functional do not. Remote import URLs are never fetched.

See [OWL imports reference](../reference/owl-imports.md). For remote or multi-format bundles, merge upstream with ROBOT or OWL API.

## `classify` / `explain` behavior

CLI **`classify --profile auto|el|rl|rdfs|alc|dl|dl-preview|swrl`** routes via `ontologos-facade`. Use **`materialize`** for explicit RDFS. **`explain`** is available on v1.0.0 (EL full traces; RL/RDFS asserted-only).

Library users: call **`ontologos_facade::classify`** or profile crate helpers (`ontologos_el::classify_reasoner`, `ontologos_rl::rdfs::classify_reasoner`, `ontologos_rl::classify_reasoner`, `ontologos_dl::classify`). Classification is **not** on `ontologos_core::Reasoner`. CLI and Python route via the facade. See [Facade API](facade-api.md), [CLI reference](../reference/cli.md), [errors.md](../reference/errors.md), and [Choosing an API](../guides/choosing-an-api.md).

## DL preview errors

| Error | Symptom | Fix |
|-------|---------|-----|
| `PreviewLimit` | Construct not in preview scope | Use stable `dl` profile or simplify ontology |
| `ResourceLimit` | Tableau expansion budget exhausted (4096) | Reduce ontology size or retry with smaller corpus |
| Wrong profile on DL ontology | Unexpected taxonomy shape | Run `ontologos profile file.owl`; use `--profile dl` |

See [Preview profiles](preview-profiles.md).

## RDFS does not expand `EquivalentClasses`

RDFS materializes primarily `subClassOf` transitive closure via reasonable. Some RDFS rules (`subPropertyOf` transitivity, domain/range inheritance) have [upstream gaps](../reference/reasonable-limits.md). Named `EquivalentClasses` axioms are expanded into mutual subsumption by `ontologos-rl` during saturation. ABox `rdf:type` propagation is handled by RL rules when using `ontologos-rl` or Python `profile="rl"`.

## What version am I running?

| Surface | Command |
|---------|---------|
| **Python** | `python -c "import ontologos; print(ontologos.__version__)"` |
| **CLI** | `ontologos --version` |
| **Rust dependency** | `cargo tree -p ontologos-core \| head -1` |

Expected from registries today: **1.0.0**. See [Release status](../project/release-status.md).

## Binding build failures

| Symptom | Fix |
|---------|-----|
| Java: `UnsatisfiedLinkError` / cannot load native library | Build JNI first: `cargo build -p ontologos-jni --release`. Set `java.library.path` or `-Dontologos.native.path=…` — see [Java guide](java.md) |
| Node: `Cannot find module 'ontologos'` or addon load error | Run `npm run build` in `crates/ontologos-node` after any Rust change |
| WASM: bundler cannot resolve `.wasm` | Follow wasm-bindgen docs for your bundler — see [WASM guide](wasm.md) |
| .NET: `DllNotFoundException` | `cargo build -p ontologos-dotnet --release` before `dotnet test` |
| C/C++: linker error for `libontologos_c` | `cargo build -p ontologos-c --release`; add `-L target/release` |

Full binding matrix: [Bindings overview](bindings-overview.md).
