# WebAssembly Guide

[![Wasmer](https://img.shields.io/badge/Wasmer-1.1.4-4946E5?logo=wasmer&logoColor=white)](https://wasmer.io/eddiethedean/ontologos)

Browser bindings ship in **`@ontologos/wasm`** (`crates/ontologos-wasm`). Shared logic lives in `ontologos-js`; the WASM crate is a thin wasm-bindgen wrapper.

The `.wasm` module is published to the Wasmer Registry as **[`eddiethedean/ontologos`](https://wasmer.io/eddiethedean/ontologos)**. Browser apps still need this crate’s wasm-bindgen JS glue (`npm run build` locally, or consume the package sources).

## Install

| Channel | How |
|---------|-----|
| **Wasmer** | [wasmer.io/eddiethedean/ontologos](https://wasmer.io/eddiethedean/ontologos) — published package **1.1.4** |
| **Source** | Build with wasm-pack (below) for JS glue + local `.wasm` |

## Build

```bash
# Install wasm-pack: https://rustwasm.github.io/wasm-pack/installer/
cd crates/ontologos-wasm
npm install
npm run build   # wasm-pack build --target web --out-dir pkg
npm test
```

The release artifact is large when DL support is included — expect multi‑MB `.wasm` files. EL/RL-only browser builds may be slimmed in a future release.

## Wasmer Registry

On each `v*.*.*` tag (or via the manual **Publish Wasmer** workflow), CI builds with wasm-pack and runs `wasmer publish` for **`eddiethedean/ontologos`** (see [`wasmer.toml`](https://github.com/eddiethedean/ontologos/blob/main/crates/ontologos-wasm/wasmer.toml)).

Package page: [wasmer.io/eddiethedean/ontologos](https://wasmer.io/eddiethedean/ontologos)

This is a wasm-bindgen module (`abi = "none"`), not a WASI CLI — use it with the JS glue from this crate, not `wasmer run` alone.

## Browser usage

```javascript
import init, { OntologyBuilder, Reasoner } from "@ontologos/wasm";

// Fetch or import the .wasm bytes, then initialize once:
await init({ module_or_path: "/path/to/ontologos_wasm_bg.wasm" });

const builder = new OntologyBuilder();
builder.addClass("http://example.org/Pizza");
builder.addClass("http://example.org/Food");
builder.subclassOf("http://example.org/Pizza", "http://example.org/Food");
const ontology = builder.build();

const reasoner = new Reasoner(ontology, "el");
const report = reasoner.classify();
console.log(report.status, report.subsumption_count);
```

Bundlers (Vite, webpack, etc.) usually resolve the wasm asset — follow your bundler's wasm-bindgen integration docs.

## No filesystem

WASM has no file API. Load ontologies from:

- **`Ontology.fromBytes(Uint8Array)`** — strict parse (default for uploads)
- **`Ontology.fromBytesLenient`** — trusted corpora only
- **`Ontology.fromText(string)`** / **`fromTextLenient`**
- **`Ontology.fromJson`**, **`fromObject`**, or **`fromJsonWithLimits`** / **`fromObjectWithLimits`**

## Resource limits

```javascript
import { Ontology } from "@ontologos/wasm";

const ontology = Ontology.fromJsonWithLimits(json, 1_048_576);
```

Recommended browser upload caps: **1–4 MiB** JSON or OWL text unless you control memory budgets.

## Errors

WASM throws `Error` subclasses with `.name` and `.code` set to `ParseError`, `ResourceLimitError`, or `IncompleteReasoningError`:

```javascript
try {
  Ontology.fromJson("{");
} catch (err) {
  console.log(err.name); // "ParseError"
}
```

## Threading and performance

- Bindings are **single-threaded** (`Rc<RefCell>`). One WASM instance per tab/worker; do not share handles across workers.
- **DL classification** can block the browser main thread. Pass `budgetSecs` to `Reasoner` and run heavy DL jobs in a Web Worker with a dedicated WASM instance.
- Remote `owl:imports` are not fetched in WASM (horned-owl `remote` feature disabled for `wasm32`).

## Security

See [Security — JavaScript bindings](../security.md#javascript-bindings-node-wasm). Default byte/text loaders use **strict** parse limits.

## API surface

Mirrors Node/Python: `Ontology`, `OntologyBuilder`, `Reasoner` with `classify()`, `checkConsistency()`, `isEntailed()`, `query()`, and incremental edits.
