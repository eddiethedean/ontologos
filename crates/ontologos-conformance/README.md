# ontologos-conformance

**Internal HermiT parity harness** — engine burndown, catalog promotion, and phase closure gates.

PR CI uses [`ontologos-contract`](../ontologos-contract) for user-facing API checks routed through `ontologos_facade`. Run this crate locally when working on HermiT parity or DL engine internals.

## New contributors

Read **[HermiT burndown guide](../../docs/guides/hermit-burndown.md)** before changing this crate.

```bash
bash benchmarks/scripts/hermit-burndown.sh status
bash benchmarks/scripts/hermit-burndown.sh loop
```

## CI tiers

| Target | When | Command |
|--------|------|---------|
| User contract | Every PR | `cargo test -p ontologos-contract --release` |
| Parity (Tier A) | Nightly / release | `cargo test -p ontologos-conformance --release` |

## Layout

| Path | Role |
|------|------|
| `src/catalog/` | Catalog loader, internal + `check_user_axiom_case` semantic checks |
| `src/bin/` | CLI tools (`parity_status`, `wg_failures`, `promote_catalog`, …) |
| `tests/hermit_*_generated.rs` | Auto-generated from `tests/hermit/generate_catalog.py` |
| `tests/hermit_{rl,rdfs,el}.rs` | Hand-written ports |
| `tests/phase*_closure.rs` | Burndown phase gates (nightly) |

Catalog data: `benchmarks/data/hermit/catalog/`.
