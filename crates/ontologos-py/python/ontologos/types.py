"""Public TypedDict shapes for OntoLogos Python bindings."""

from __future__ import annotations

from typing import NotRequired, TypedDict


class ParseMeta(TypedDict):
    warnings: list[str]
    mapped_axiom_count: int
    skipped_axiom_count: int
    logical_axiom_count: int


class ProofNode(TypedDict, total=False):
    rule: str
    premises: list[int]
    conclusion_axiom: int
    conclusion_sub: tuple[str, str]
    conclusion_existential: tuple[str, str, str]
    conclusion_subproperty: tuple[str, str]


class ExplainResult(TypedDict):
    node_count: int
    nodes: list[ProofNode]
    parse_meta: NotRequired[ParseMeta]


class TaxonomyResult(TypedDict):
    status: str
    subsumption_count: int
    subsumptions: list[tuple[str, str]]
    equivalences: list[list[str]]
    unsatisfiable: list[str]
    parse_meta: NotRequired[ParseMeta]


class MaterializeResult(TypedDict):
    status: str
    initial_axiom_count: int
    final_axiom_count: int
    inferred_axioms: int
    inferred_by_rule: dict[str, int]
    clash_count: int
    clashes: NotRequired[list[str]]
    parse_meta: NotRequired[ParseMeta]


class ConsistencyResult(TypedDict):
    consistent: bool
    complete: bool


ClassifyResult = TaxonomyResult | MaterializeResult

__all__ = [
    "ClassifyResult",
    "ConsistencyResult",
    "ExplainResult",
    "MaterializeResult",
    "ParseMeta",
    "ProofNode",
    "TaxonomyResult",
]
