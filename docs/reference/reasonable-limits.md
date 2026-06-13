# Reasonable Adapter Limits

Public summary of upstream gaps when `ontologos-rdfs` and `ontologos-rl` delegate to [**reasonable**](https://crates.io/crates/reasonable). Full rationale: [dependency-first ADR](../internal/design/dependency-first.md).

OntoLogos intentionally does **not** reimplement these rules in-tree. Gaps are tracked as upstream issues; HermiT-ported tests that depend on missing rules are marked `ignored` in CI.

## RDFS gaps

| Rule / semantics | Status in reasonable | OntoLogos impact |
|------------------|------------------------|------------------|
| Transitive `subClassOf` | Implemented | Works |
| Transitive `subPropertyOf` (RDFS 5) | **Not implemented** | `subPropertyOf` closure may be incomplete |
| Domain inheritance along `subPropertyOf` (RDFS 6) | **Not implemented** | Domain axioms may not propagate |
| Range inheritance along `subPropertyOf` (RDFS 8) | **Not implemented** | Range axioms may not propagate |
| Mutual `subPropertyOf` from `equivalentProperty` | **Not implemented** | Equivalence expansion incomplete for properties |

## OWL RL gaps (via reasonable)

| Rule / semantics | Status | OntoLogos impact |
|------------------|--------|------------------|
| Property-characteristic propagation along `subPropertyOf` | **Not implemented** | Some RL TBox rules may not fire |
| Existential TBox subsumption between named classes | **Not implemented** | Use in-house EL for taxonomy; RL tests ignored |
| Domain on subproperty typing superproperty assertions | **Not implemented** | ABox typing may differ from HermiT |
| Rule-level explanation traces | **Not implemented** | Proof graphs seed asserted axioms only |

## What still works

- **OWL EL classification** — in-house completion in `ontologos-el` (Pizza golden in CI)
- **RL saturation** — reasonable implements core OWL RL forward-chaining for mapped axioms
- **RDFS `subClassOf` closure** — primary TBox materialization path
- **Clash detection** — partial (see [supported constructs](supported-constructs.md))

## Evaluator guidance

When comparing OntoLogos RDFS/RL output to HermiT or RDFox:

1. Check [Conformance](conformance.md) for tier coverage and ignored tests.
2. Prefer **EL classification** (`--profile el`) for taxonomy benchmarks on EL corpora.
3. Use `inferred_total()` and axiom counts — not `inferred_by_rule` — for RL/RDFS metrics until reasonable exposes diagnostics.

## Related

- [Supported constructs](supported-constructs.md)
- [Comparison](../comparison.md)
- [RL rules reference](rl-rules.md)
- [Explain API](explain.md)
