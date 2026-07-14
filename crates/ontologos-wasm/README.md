# @ontologos/wasm

[![Wasmer](https://img.shields.io/badge/Wasmer-1.1.4-4946E5?logo=wasmer&logoColor=white)](https://wasmer.io/eddiethedean/ontologos)
[![Documentation](https://readthedocs.org/projects/ontologos/badge/?version=latest)](https://ontologos.readthedocs.io/en/latest/guides/wasm/)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

WebAssembly bindings for [OntoLogos](https://github.com/eddiethedean/ontologos).

The `.wasm` module is published to the Wasmer Registry as **[`eddiethedean/ontologos`](https://wasmer.io/eddiethedean/ontologos)** (see `wasmer.toml`). Browser usage still requires this package’s wasm-bindgen JS glue.

## Install

| Channel | How |
|---------|-----|
| **Wasmer** | Package page: [wasmer.io/eddiethedean/ontologos](https://wasmer.io/eddiethedean/ontologos) |
| **Source** | Build this crate with wasm-pack (below) |

## Build

```bash
# Install wasm-pack: https://rustwasm.github.io/wasm-pack/installer/
cd crates/ontologos-wasm
npm install
npm run build
npm test
```

## Browser usage

```javascript
import init, { OntologyBuilder, Reasoner } from "@ontologos/wasm";

await init({ module_or_path: wasmUrl });

const builder = new OntologyBuilder();
builder.addClass("http://example.org/Pizza");
builder.addClass("http://example.org/Food");
builder.subclassOf("http://example.org/Pizza", "http://example.org/Food");
const ontology = builder.build();

const reasoner = new Reasoner(ontology, "el");
const report = reasoner.classify();
console.log(report);
```

Load OWL from bytes or text (no filesystem):

```javascript
const ontology = Ontology.fromBytes(new TextEncoder().encode(owlString));
const reasoner = new Reasoner(ontology, "auto");
```

## Security

- **`fromBytes` / `fromText`** use strict parse defaults for untrusted uploads.
- Use **`fromBytesLenient` / `fromTextLenient`** only for trusted corpora.
- Use **`fromJsonWithLimits`** for user JSON with tightened `max_json_bytes`.
- DL reasoning can block the main thread — set `budgetSecs` and prefer Web Workers.

Full guide: [WebAssembly bindings](https://ontologos.readthedocs.io/en/latest/guides/wasm.html)

## API

Mirrors the Python bindings: `Ontology`, `OntologyBuilder`, `Reasoner` with `classify()`, `checkConsistency()`, `isEntailed()`, `query()`, and incremental axiom edits.
