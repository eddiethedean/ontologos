# When not to use OntoLogos

Honest guidance for evaluators and architects. See also [Comparison](../comparison.md) and [Evaluator scope](evaluator-scope.md).

## Use OntoLogos when

- You need OWL reasoning **embedded in Rust or Python** without a JVM
- You want one library/CLI for **EL, RL, RDFS, DL, or SWRL** routing
- You load ontologies programmatically, from files, or JSON snapshots
- You run **batch classification or materialization** in services or pipelines
- Your ontology fits the [supported construct subset](../reference/supported-constructs.md) (or you validate DL on your corpus)

## Do not use OntoLogos when

| Situation | Better alternative |
|-----------|-------------------|
| **Interactive OWL editing** with plugins and visual axioms | [Protégé](https://protege.stanford.edu/) + ELK/HermiT |
| **EL-only, maximum EL performance** | [ELK](https://github.com/owlcs/elk), [whelk-rs](https://github.com/INCATools/whelk-rs) |
| **RL-only, triple-store materialization** without core model | [reasonable](https://crates.io/crates/reasonable) directly |
| **Maximum DL performance on very large ontologies** | [Konclude](https://www.cs.ox.ac.uk/isg/tools/Konclude/) |
| **OWL QL query answering at scale** | ELK or a dedicated QL reasoner (`ql` detection only in OntoLogos) |
| **Full OWL axiom coverage** (every Protégé axiom mapped) | Java reasoners; OntoLogos maps a **subset** — see [Known limitations](known-limitations.md) |
| **Remote `owl:imports` fetched automatically** | Merge with [ROBOT](http://robot.obolibrary.org/) first |
| **OWL export after reasoning** | Retain source OWL + JSON snapshot; no built-in serializer |
| **Plugin / extension API** | Not available — OntoLogos is a library, not a host platform |
| **Guaranteed HermiT-identical output on any ontology** | Validate yourself — parity is on **889 gated cases**, not universal |

## OWL DL specifically

OntoLogos DL passes blocking CI on the gated HermiT catalog @ 30s. That does **not** guarantee:

- Every real-world ontology classifies correctly
- Identical taxonomy to HermiT (Tier C allows sound superset)
- Performance on multi-million-axiom corpora without tuning

**Before production DL cutover:** run the [Evaluator playbook](evaluator-playbook.md) on **your** ontologies and compare against HermiT/Konclude.

## Python with JVM backends

If your team already standardizes on **owlready2 + HermiT/ELK**, switching to OntoLogos is a trade-off: native wheels and no JVM vs. mature desktop/research ecosystem integration.

## Related

- [Evaluator scope](evaluator-scope.md) — what HermiT parity metrics mean
- [Profile stability](profile-stability.md) — stable vs preview profiles
- [Choosing an API](choosing-an-api.md) — when to use upstream crates directly
