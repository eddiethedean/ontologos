# @ontologos/wasm

WebAssembly bindings for [OntoLogos](https://github.com/eddiethedean/ontologos).

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
