# @ontologos/wasm

[![npm](https://img.shields.io/npm/v/@ontologos/wasm.svg)](https://www.npmjs.com/package/@ontologos/wasm)
[![Wasmer](https://img.shields.io/badge/Wasmer-1.1.4-4946E5?logo=wasmer&logoColor=white)](https://wasmer.io/eddiethedean/ontologos)
[![Documentation](https://readthedocs.org/projects/ontologos/badge/?version=latest)](https://ontologos.readthedocs.io/en/latest/guides/wasm/)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

WebAssembly bindings for [OntoLogos](https://github.com/eddiethedean/ontologos).

## Install

```bash
npm install @ontologos/wasm
```

| Channel | How |
|---------|-----|
| **npm** | [`@ontologos/wasm`](https://www.npmjs.com/package/@ontologos/wasm) |
| **Wasmer** | [`eddiethedean/ontologos`](https://wasmer.io/eddiethedean/ontologos) (`.wasm` module) |
| **Source** | Build this crate with wasm-pack (below) — preferred until the corrected npm tarball ships |

> First `@ontologos/wasm@1.1.4` omitted `pkg/`; use source build or Wasmer for the `.wasm` binary for now.

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
