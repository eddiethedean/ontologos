# Rust Modernization Report

OntoLogos workspace modernization pass (Rust 2024, MSRV 1.85). This document records baseline results, changes by phase, and validation status.

## Workspace overview

| Item | Value |
|------|-------|
| Members | 19 crates (17 libs, `ontologos-cli`, `ontologos-py`, `ontologos-conformance`) |
| Edition / MSRV | Rust 2024, `rust-version = "1.85"` |
| Workspace lints | `unsafe_code = deny`; clippy `all` warn; pedantic allow (documented in root `Cargo.toml`) |
| Async | None (sync + threads in conformance/parser) |
| Benches | 7 criterion benches |

## Anti-patterns audit (corrected)

Raw `unwrap`/`expect` counts in `src/` were inflated by large inline `#[cfg(test)]` modules (e.g. former `facade/src/lib.rs`, `rdf_preprocess.rs`). Production panic debt is much smaller.

| Area | Before | After |
|------|--------|-------|
| ALC tableau production `panic!` | 4 | 0 (`TupleIndexError`, `merge_error` paths) |
| Conformance `Result<_, String>` | pervasive | `CatalogError` introduced; `run_dl_bounded` migrated; public APIs keep `String` wrappers |
| Parser I/O errors | `Parse(String)` | `Error::Io(#[from])` |
| Profile errors in EL/DL/facade | `Profile(String)` | `Profile(#[from] ontologos_profile::Error)` |

## Changes by phase

### Phase 0 — Baseline

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Pass (after fixes) |
| `cargo test --workspace --all-features --exclude ontologos-conformance --locked` | Pass |

### Phase 1 — Workspace hygiene

- Added `[lints] workspace = true` to all 19 member `Cargo.toml` files.
- Root `Cargo.toml`: `unsafe_code = "deny"`; pedantic allowed workspace-wide with comment.
- **Inline tests moved:**
  - `ontologos-facade`: `tests/facade_routing.rs` (integration tests via public API).
  - `ontologos-parser`: `src/rdf_preprocess_tests.rs` via `#[path = ...]`.

### Phase 2 — Safety and error chaining

- **ALC tableau:** `TupleIndexExhausted`, `TupleIndexError`, `add_tuple`/`remove_tuple` return `Result`; extension manager merge errors propagate without `panic!`.
- **Parser:** `ontologos_parser::Error::Io(#[from] std::io::Error)`.
- **EL / DL / explain / facade:** `Profile(#[from] ontologos_profile::Error)`; facade uses `Message` for ad-hoc strings.
- **Parser mutex docs:** `ONTOLOGY_LOAD_LOCK` and `HORNED_OWL_READ_LOCK` documented (horned-owl thread safety).

### Phase 3 — Conformance harness

- **`catalog/` split (partial, move-only):**
  - `catalog/types.rs` — `HermitCase`, `WgCase`, expectation structs.
  - `catalog/mod.rs` — remainder of former `catalog.rs` (~10k lines).
  - Full submodule split (`loader.rs`, `checks.rs`, `wg.rs`, …) deferred: mechanical line-range split broke at helper boundaries; types extraction is stable.
- **`CatalogError`** (`catalog_error.rs`): `CaseFailed`, `Dl`, `Parser`, `Message`; `From<String>`; `run_dl_bounded_inner` returns `CatalogError`; public `check_axiom_case*` still `Result<(), String>`.

### Phase 4 — Module organization

- **`ontologos-facade` split:**
  - `error.rs`, `classify.rs`, `entailment.rs`, `lookup.rs`, `query.rs`; `lib.rs` re-exports only (no public API change).
- **`rdf_preprocess`:** tests externalized; **file split deferred** — normalize/rewrite boundary crosses shared private helpers (`parse_entity_decl`, `parse_xml_base`, etc.); monolithic module retained.

### Phase 5 — Performance

No clone reductions without criterion evidence (per plan).

## API / display changes

- `el::Error::Profile` now nests `profile::Error` display text (may differ slightly from former `format!("profile detection failed: {e}")`).
- Parser I/O failures surface as `Io` variant instead of `Parse("...")` where migrated.

## Final validation

| Command | Status |
|---------|--------|
| `cargo fmt --all` | Pass |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Pass |
| `cargo test --workspace --all-features --exclude ontologos-conformance --locked` | Pass |
| `bash benchmarks/scripts/run-ci-local.sh` | Pass (2026-07-01T00:55:56Z) |

## Remaining recommendations

1. Complete `catalog/` split (`loader`, `checks`, `wg`, `scan`, `guards`) with dependency-aware boundaries, not fixed line ranges.
2. Migrate `check_axiom_case_with_opts` internals to `CatalogError` (public `String` wrappers already in place).
3. Split `rdf_preprocess` into `normalize` / `rewrite` / `entities` / `literals` using a shared `helpers` module or `pub(crate)` dependency graph.
4. Criterion regression gates for tableau hot paths.
5. `datatype/consistency.rs` and CLI `main.rs` subcommand splits (out of scope this pass).

## Risk notes

| Change | Risk | Mitigation |
|--------|------|------------|
| Catalog types extraction | Low | Build + conformance tests |
| `CatalogError` / profile `From` | Error message churn | Documented; grep tests for exact strings |
| Tableau `panic!` → `Err` | New DL error paths | ALC unit + HermiT suite |
| Facade module split | Low | Same `pub use` surface |
