# RDFox Evaluation

## Overview

RDFox is a high-performance rule-based RDF triple store with Datalog reasoning. It excels at OWL RL materialization over large RDF datasets through forward chaining.

## Architecture

| Component | Role |
|-----------|------|
| **RDF store** | Column-oriented triple storage |
| **Rule compiler** | Compiles OWL RL rules to Datalog |
| **Forward chainer** | Applies rules until saturation |
| **Parallel execution** | Multi-threaded rule firing |
| **Explanation** | Derivation trees for inferred triples |

## OWL RL approach

OWL RL rules map OWL axioms to RDF triple patterns:

- `equivalentClass` → bidirectional `rdfs:subClassOf`
- `inverseOf` → symmetric inference on property assertions
- `TransitiveProperty` → transitive closure on property chains
- Domain/range → `rdf:type` propagation

Rules are indexed by head predicate for efficient matching.

## Performance

- Sub-second materialization on million-triple datasets
- Commercial license; not embeddable in open-source Rust projects
- Validates that rule indexing + parallel forward chaining is the right RL strategy

## Implications for OntoLogos

1. **`ontologos-rl` (v0.4)** should use forward chaining with rule indexing, not tableau.
2. **`TripleIndex`** (`HashMap<EntityId, Vec<TripleId>>`) indexes inferred triples by subject for O(1) rule matching — aligns with RDFox's head-predicate indexing.
3. **Parallel rule execution** in `RlEngine::saturate()` should partition work by rule batch; `ReasonerConfig::parallelism` controls thread count.
4. **Materialization output** is a set of derived axioms/triples layered on the TBox/ABox; RDFS engine (v0.3) shares the same materialization pattern.
5. **Do not build a triple store** — OntoLogos reasons over in-memory ontologies; OntoIndex handles query/index at scale.
6. **Open RL reference** — [reasonable](https://github.com/gtfierro/reasonable) (Rust, DataFrog) is the practical open benchmark for RL materialization; compare v0.4 output against reasonable + OWLRL, not only RDFox.
7. **MORe integration** — RDFox is used inside MORe for RL modules; validates MORe-style hybrid routing in v1.5. See [more.md](more.md).
