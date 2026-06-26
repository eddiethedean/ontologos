# ontologos-conformance

HermiT- and OWL WG-ported integration tests for OntoLogos.

## New contributors

Read **[HermiT burndown guide](../../docs/guides/hermit-burndown.md)** before changing this crate.

```bash
bash benchmarks/scripts/hermit-burndown.sh status
bash benchmarks/scripts/hermit-burndown.sh loop
```

## Layout

| Path | Role |
|------|------|
| `src/catalog.rs` | Catalog loader, semantic checks, scan/promote helpers |
| `src/bin/` | CLI tools (`parity_status`, `wg_failures`, `promote_catalog`, …) |
| `tests/hermit_*_generated.rs` | Auto-generated from `tests/hermit/generate_catalog.py` |
| `tests/hermit_{rl,rdfs,el}.rs` | Hand-written ports |
| `tests/wg_phase4_check.rs` | WG regression gate |

Catalog data: `benchmarks/data/hermit/catalog/`.
