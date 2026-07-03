# Datatype manager waiver (HermiT Phase B4)

**Status:** Accepted — permanent catalog exclusion with targeted parser coverage.

## Scope

Twenty-five HermiT JVM tests under `AnyURITest`, `BinaryDataTest`, `RDFPlainLiteralTest`, and `DateTimeTest` exercise HermiT's internal `DatatypeRegistry` / `ValueSpaceSubset` APIs. They validate lexical facets, enumeration, pattern/length facets, and XSD dateTime parsing inside the JVM datatype manager — not OWL DL/TBox reasoning.

OntoLogos does not ship a JVM-compatible datatype manager. These cases remain in `EXCLUDED_IDS` in [tests/hermit/generate_catalog.py](https://github.com/eddiethedean/ontologos/blob/main/tests/hermit/generate_catalog.py) and are **not** promoted to `run_hermit_case`.

## Excluded HermiT IDs

| Suite | Methods | JVM concern |
|-------|---------|-------------|
| `AnyURITest` | `testInvalidAnyURILiterals`, `testPatternAndLength2/3`, `testComplement2/3/4` | `anyURI` facet algebra, cardinality of value spaces |
| `BinaryDataTest` | `testExplicitSize`, `testEnumerate1/2`, `testBase64Parsing` | `xsd:base64Binary` / hex enumeration |
| `RDFPlainLiteralTest` | `testInvalidStringLiterals`, `testExplicitSize`, `testEnumerate`, `testPatternAndLength2/3`, `testComplement2/3/4`, `testLangRange1/2` | RDF plain literal + language-tag facets |
| `DateTimeTest` | `testParsing`, `testExactIntervalsWithoutTZ1/2`, `testExactIntervalsWithTZ1/2/3` | XSD dateTime timezone / interval equality |

## What OntoLogos covers instead

| Layer | Coverage |
|-------|----------|
| **Parser** | OFN/RDF literal lexicals, `DatatypeDefinition`, known XSD aliases — see `crates/ontologos-parser/tests/datatype_manager_waiver.rs` |
| **Reasoner** | OWL datatype restrictions in DL (`DatatypesTest`, WG datatype cases) via `ontologos-dl` — separate from JVM facet manager |
| **Conformance guards** | Datatype-range / sameAs literal entailment guards in `ontologos-conformance/src/catalog.rs` |

## Re-inclusion criteria

Promote a datatype-manager case only when:

1. The Java test asserts an **OWL entailment** (subsumption, satisfiability, or ABox) mappable to OFN/RDF, **and**
2. OntoLogos passes via `run_hermit_case` or a hand-written Rust port with the same logical claim.

Facet enumeration / `ValueSpaceSubset.hasCardinalityAtLeast` checks stay waived.

## Related OWLLink exclusions (not datatype manager)

`OWLLinkTest.testBobTestC` and `testBobTests` remain in `EXCLUDED_IDS` — DL ABox property-value retrieval and multi-fixture entailment on the Liebig corpus need inverse/transitive role materialization beyond current RL ABox saturation. See [parity-roadmap](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/parity-roadmap.md).

## Related

- [alc-boundary.md](alc-boundary.md) — ALC vs DL construct routing
- [parity-roadmap](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/parity-roadmap.md) — Tier B4 burn-down
- `tests/hermit/internal_ports.toml` — structural/clausification datatype goldens (B3)
