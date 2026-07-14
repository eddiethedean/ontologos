# Node.js Guide

[![npm](https://img.shields.io/npm/v/ontologos.svg)](https://www.npmjs.com/package/ontologos)

Native Node.js bindings ship as the npm package **[`ontologos`](https://www.npmjs.com/package/ontologos)** (`crates/ontologos-node`). They mirror the Python API via shared logic in `ontologos-js`.

## Install

```bash
npm install ontologos
```

Published **v1.1.4** with prebuilt native addons for macOS (arm64/x64), Linux gnu (arm64/x64), and Windows x64. Requires **Node.js 18+**.
### Build from source

```bash
cd crates/ontologos-node
npm install
npm run build
```

Requires a Rust toolchain (napi-rs compiles the native addon).

## Quick start

```javascript
const { Reasoner } = require("ontologos");

const reasoner = Reasoner.fromPath("family.owl", "rl");
const report = reasoner.classify();
console.log(report.status, report.subsumption_count ?? report.inferred_axioms);
```

Build in memory:

```javascript
const { OntologyBuilder, Reasoner } = require("ontologos");

const builder = new OntologyBuilder();
builder.addClass("http://example.org/Pizza");
builder.addClass("http://example.org/Food");
builder.subclassOf("http://example.org/Pizza", "http://example.org/Food");
const ontology = builder.build();

const reasoner = new Reasoner(ontology, "el");
console.log(reasoner.classify());
```

## File loading

| Method | Parse mode | Sandbox | Use when |
|--------|------------|---------|----------|
| `Ontology.load(path)` | Lenient | No | Trusted local corpora only |
| `Ontology.loadIn(base, path)` | Strict | Yes | User uploads; path resolved under `base` |
| `Reasoner.fromPath(path)` | Lenient | No | Trusted paths |
| `Reasoner.loadIn(base, path)` | Strict | Yes | Sandboxed server uploads |

For untrusted uploads, prefer **`loadIn`** with a dedicated sandbox directory. Relative `path` values are resolved under `base`.

In-memory untrusted input should use **`Ontology.fromBytes`** / **`fromText`** (strict). Use **`fromBytesLenient`** / **`fromTextLenient`** only for trusted corpora.

## Resource limits

For JSON snapshots from users:

```javascript
const { Ontology } = require("ontologos");

const ontology = Ontology.fromJsonWithLimits(json, 1_048_576); // 1 MiB cap
// or
const ontology = Ontology.fromObjectWithLimits(obj, 1_048_576, 100_000, 1_000_000, 8192);
```

Defaults match [`Limits`](../reference/core.md) in `ontologos-core` (16 MiB JSON, 1M entities, etc.).

## Shared ontology and incremental edits

`Reasoner` and `Ontology` share the same in-memory graph when constructed together. Incremental axiom edits on the reasoner are visible on the ontology handle:

```javascript
const ontology = builder.build();
const reasoner = new Reasoner(ontology, "el");
reasoner.addSubclassOf("http://example.org/A", "http://example.org/B");
console.log(ontology.axiomCount); // reflects the new axiom
```

The handle uses `Rc<RefCell<…>>` internally — **single-threaded only**. Do not share across worker threads or `worker_threads` without separate instances.

## Errors

Node surfaces parse and limit failures with message prefixes such as `ParseError:`, `ResourceLimitError:`, and `IncompleteReasoningError:`. Use `errorCodeFromMessage(err.message)` to extract the code for branching.

## Server deployment notes

- OWL file parsing uses a **process-wide horned-owl mutex** — concurrent `load` / `loadIn` calls serialize. Use a queue or worker pool with one load at a time per process.
- Set `budgetSecs` on `Reasoner` for DL workloads to avoid unbounded tableau work.
- See [Security](../security.md#javascript-bindings-node-wasm) and [Production integration](production-integration.md).

## API surface

`Ontology`, `OntologyBuilder`, and `Reasoner` expose `classify()`, `checkConsistency()`, `isEntailed()`, `query()`, `explain()`, and incremental axiom helpers — aligned with the [Python guide](python.md).
