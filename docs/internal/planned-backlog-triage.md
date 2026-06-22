# Planned backlog triage

**Generated:** 2026-06-22 (UTC) via `benchmarks/scripts/audit-planned-backlog.sh`

Do not edit by hand — regenerate after catalog or engine changes.

## Summary

| Catalog | Planned |
|---------|--------:|
| HermiT Java (`cases.json`) | 330 |
| OWL WG (`wg_cases.json`) | 67 |

### Java by category

| Category | Count |
|----------|------:|
| `engine_gap` | 8 |
| `manual_port` | 235 |
| `missing_assertions` | 87 |

### WG by category

| Category | Count |
|----------|------:|
| `missing_expectations` | 15 |
| `missing_premise` | 52 |

## Promotion candidates (Java)

_None — run `promote_catalog` after engine fixes._

## Engine gaps (sample Java)

- `reasoner.OWLReasonerTest.testIncrementalAddition2` — reasoner.OWLReasonerTest.testIncrementalAddition2: consistency expected false, got true
- `reasoner.ReasonerTest.testChains2` — reasoner.ReasonerTest.testChains2: consistency expected false, got true
- `reasoner.ReasonerTest.testChains` — reasoner.ReasonerTest.testChains: consistency expected false, got true
- `reasoner.ReasonerTest.testNegativeDataPropertyAssertion` — reasoner.ReasonerTest.testNegativeDataPropertyAssertion: consistency expected false, got true
- `reasoner.ReasonerTest.testNegProperties` — reasoner.ReasonerTest.testNegProperties: consistency expected false, got true
- `reasoner.ReasonerTest.testRoleDisjointness_1` — reasoner.ReasonerTest.testRoleDisjointness_1: consistency expected false, got true
- `reasoner.ReasonerTest.testRoleDisjointness_2` — reasoner.ReasonerTest.testRoleDisjointness_2: consistency expected false, got true
- `reasoner.ReasonerTest.testInverses2` — reasoner.ReasonerTest.testInverses2: consistency expected false, got true
