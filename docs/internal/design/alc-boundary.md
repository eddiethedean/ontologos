# ALC vs DL boundary (v1.7)

**Status:** Phase 8 / v1.7 exit documentation.

## What `ontologos-alc` covers

| Construct | ALC (`Profile::Alc`) | Full DL (`Profile::Dl`) |
|-----------|----------------------|-------------------------|
| ⊤ / ⊥ | Yes | Yes |
| Atomic classes | Yes | Yes |
| Intersection ⊓ | Yes | Yes |
| Union ⊔ | Yes | Yes |
| Complement ¬ | Yes | Yes |
| Existential ∃R.C | Yes | Yes |
| Universal ∀R.C | Yes | Yes |
| Role hierarchy | Yes (H) | Yes |
| Nominals {a} | No | Yes |
| Cardinality ≥/≤/= n | No | Yes |
| Datatype restrictions | No | Yes |
| Property chains | No | Yes |
| Keys / complex ABox | No | Partial |

## Routing

- `ontologos-facade::classify` with `Profile::Alc` delegates to `ontologos_alc::classify`.
- `Profile::Dl` delegates to `ontologos-dl`, which uses ALC tableau internally plus DL extensions.
- `Profile::Auto` on DL-detected ontologies uses MORe-style hybrid routing (v1.5), not ALC alone.

## Exit tests

See `crates/ontologos-alc/tests/alc_exit.rs`:

- Synthetic ALC unsat (equivalence + disjointness)
- Universal/existential subsumption
- Pizza corpus consistency + basic subsumption
- ALC extension unsat (Pizza + disjoint spicy/not-spicy pattern)

## When to use ALC vs DL

Use **ALC** for TBox-only ontologies without nominals, datatypes, or cardinality — faster tableau with smaller search space.

Use **DL** for HermiT parity, OWL WG fixtures, and any ontology with datatype or cardinality restrictions.
