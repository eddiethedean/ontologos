# Node.js API Reference

[![npm](https://img.shields.io/npm/v/ontologos.svg)](https://www.npmjs.com/package/ontologos)

Native Node.js bindings via npm package **`ontologos`** (`crates/ontologos-node`) — see [Bindings overview](../guides/bindings-overview.md).

Tutorial: [Node.js guide](../guides/node.md).

## Install

```bash
npm install ontologos
```

Prebuilt binaries for macOS (arm64/x64), Linux gnu (arm64/x64), and Windows x64. Requires **Node.js 18+**.

### Build from source

```bash
cd crates/ontologos-node
npm install && npm run build
```

Requires a Rust toolchain (napi-rs).

## Profiles

Same strings as Python: `"auto"`, `"rdfs"`, `"rl"`, `"el"`, `"dl"`, `"swrl"`, `"alc"`, `"dl-preview"`. See [Profile stability](../guides/profile-stability.md).

## `Reasoner`

```javascript
const { Reasoner } = require("ontologos");
```

### Constructors

| Factory | Description |
|---------|-------------|
| `Reasoner.fromPath(path, profile)` | Load file (lenient, no sandbox) |
| `Reasoner.loadIn(base, path, profile)` | Sandboxed load under `base` |
| `new Reasoner(ontology, profile, options?)` | In-memory ontology |

Options: `{ incremental?: boolean, budgetSecs?: number }`.

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `classify()` | Object | Taxonomy or materialization report |
| `checkConsistency()` | `{ consistent, complete }` | Prefer for DL |
| `isConsistent()` | `boolean` | Throws on incomplete |
| `isEntailed(...)` | `boolean` | Subsumption, class, or property assertion |
| `query(query)` | `Array` | OWL QL conjunctive query |
| `explain()` | Object | Proof graph |
| `addSubclassOf(sub, sup)` | `void` | Incremental mutation |
| `removeSubclassOf(sub, sup)` | `void` | Incremental mutation |

### `classify()` return shapes

Same as [Python reference](python.md#classify-return-shapes).

## `OntologyBuilder`

| Method | Description |
|--------|-------------|
| `addClass(iri)` | Declare class |
| `subclassOf(sub, sup)` | Add `SubClassOf` |
| `build()` | Returns `Ontology` |

## `Ontology`

| Method | Description |
|--------|-------------|
| `load(path)` | Lenient file load |
| `loadIn(base, path)` | Sandboxed file load |
| `fromJson(json)` / `fromJsonWithLimits(json, maxBytes)` | JSON snapshot |
| `fromBytes(bytes)` / `fromBytesLenient(bytes)` | OWL/RDF bytes |
| `axiomCount` | Mapped axiom count (property) |

## Errors

Message prefixes: `ParseError:`, `ResourceLimitError:`, `IncompleteReasoningError:`. Use `errorCodeFromMessage(err.message)` for branching.

## Threading

Single-threaded only — uses `Rc<RefCell<…>>` internally. Do not share across `worker_threads`.

## Related

- [Security — Node](../security.md#javascript-bindings-node-wasm)
- [Python API](python.md) — canonical parity reference
