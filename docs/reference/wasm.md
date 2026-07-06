# WebAssembly API Reference

Browser bindings via **`@ontologos/wasm`** (`crates/ontologos-wasm`). Source-build only — see [Bindings overview](../guides/bindings-overview.md).

Tutorial: [WASM guide](../guides/wasm.md).

## Build

```bash
cd crates/ontologos-wasm
npm install && npm run build   # wasm-pack --target web
```

Requires **wasm-pack** and Rust `wasm32-unknown-unknown` target.

## Profiles

Same strings as Python: `"auto"`, `"rdfs"`, `"rl"`, `"el"`, `"dl"`, `"swrl"`, `"alc"`, `"dl-preview"`. DL builds produce large `.wasm` files (multi‑MB).

## Initialization

```javascript
import init, { Reasoner, OntologyBuilder } from "@ontologos/wasm";
await init({ module_or_path: wasmUrl });
```

## `Reasoner`

| Constructor | Description |
|-------------|-------------|
| `new Reasoner(ontology, profile)` | In-memory only (no filesystem) |
| `new Reasoner(ontology, profile, incremental, budgetSecs)` | Full config |

No file-path constructors — WASM has no filesystem API.

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `classify()` | Object | Taxonomy or materialization report |
| `checkConsistency()` | Object | `{ consistent, complete }` |
| `isConsistent()` | `boolean` | Throws `IncompleteReasoningError` when incomplete |
| `explain()` | Object | Proof graph |
| `addSubclassOf(sub, sup)` | `void` | Incremental mutation |

## `Ontology`

| Method | Description |
|--------|-------------|
| `fromBytes(Uint8Array)` | Strict OWL/RDF parse |
| `fromBytesLenient(Uint8Array)` | Trusted corpora only |
| `fromText(string)` / `fromTextLenient(string)` | Text parse |
| `fromJson(json)` / `fromJsonWithLimits(json, maxBytes)` | JSON snapshot |
| `fromObject(obj)` / `fromObjectWithLimits(obj, …)` | Parsed JSON object |

Recommended browser upload caps: **1–4 MiB** unless you control memory budgets.

## Errors

Throws `Error` subclasses with `.name` / `.code`: `ParseError`, `ResourceLimitError`, `IncompleteReasoningError`.

## Related

- [Python API](python.md) — canonical parity reference
- [Security — WASM](../security.md#javascript-bindings-node-wasm)
