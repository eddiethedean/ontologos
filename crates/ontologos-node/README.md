# ontologos (Node.js)

Native Node.js bindings for [OntoLogos](https://github.com/eddiethedean/ontologos).

## Build

```bash
cd crates/ontologos-node
npm install
npm run build
npm test
```

## Usage

```javascript
const { Reasoner } = require("ontologos");

const reasoner = Reasoner.fromPath("family.owl", "rl");
const report = reasoner.classify();
console.log(report);
```

Or construct from an in-memory ontology:

```javascript
const { Ontology, Reasoner } = require("ontologos");

const ontology = Ontology.fromJson(jsonSnapshot);
const reasoner = new Reasoner(ontology, "el");
```

## Security

| API | When to use |
|-----|-------------|
| `Ontology.loadIn(base, path)` | User uploads (strict, sandboxed) |
| `Ontology.load(path)` | Trusted local files only (lenient) |
| `Ontology.fromBytes` / `fromText` | Untrusted in-memory input (strict) |
| `fromJsonWithLimits` / `fromObjectWithLimits` | Untrusted JSON snapshots |

OWL file loads serialize on a process-wide parser mutex — avoid concurrent loads in cluster workers without a queue.

Full guide: [Node.js bindings](https://ontologos.readthedocs.io/en/latest/guides/node.html)

## API

Mirrors the Python bindings: `Ontology`, `OntologyBuilder`, `Reasoner` with file loading, `classify()`, `checkConsistency()`, `isEntailed()`, `query()`, and incremental axiom edits.
