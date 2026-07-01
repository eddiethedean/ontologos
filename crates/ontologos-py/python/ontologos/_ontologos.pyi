from typing import Any, TypedDict

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

class ExplainResult(TypedDict, total=False):
    node_count: int
    nodes: list[ProofNode]
    parse_meta: ParseMeta

class TaxonomyResult(TypedDict):
    status: str
    subsumption_count: int
    subsumptions: list[tuple[str, str]]
    equivalences: list[list[str]]
    unsatisfiable: list[str]

class MaterializeResult(TypedDict, total=False):
    status: str
    initial_axiom_count: int
    final_axiom_count: int
    inferred_axioms: int
    inferred_by_rule: dict[str, int]
    clash_count: int
    clashes: list[str]

class ConsistencyResult(TypedDict):
    consistent: bool
    complete: bool

class Ontology:
    @classmethod
    def from_json(cls, json: str) -> Ontology: ...
    @classmethod
    def from_json_with_limits(
        cls,
        json: str,
        *,
        max_json_bytes: int | None = None,
        max_entities: int | None = None,
        max_axioms: int | None = None,
        max_iri_len: int | None = None,
    ) -> Ontology: ...
    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Ontology: ...
    @classmethod
    def load_in(cls, base: str, path: str) -> Ontology: ...
    def to_json(self) -> str: ...
    def to_dict(self) -> dict[str, Any]: ...
    @property
    def axiom_count(self) -> int: ...
    @property
    def entity_count(self) -> int: ...

class OntologyBuilder:
    def __init__(self) -> None: ...
    def add_class(self, iri: str) -> None: ...
    def individual(self, iri: str) -> None: ...
    def object_property(self, iri: str) -> None: ...
    def subclass_of(self, subclass: str, superclass: str) -> None: ...
    def subproperty_of(self, sub: str, sup: str) -> None: ...
    def property_domain(self, property: str, domain: str) -> None: ...
    def property_range(self, property: str, range: str) -> None: ...
    def class_assertion(self, individual: str, class_: str) -> None: ...
    def object_property_assertion(
        self, subject: str, property: str, object: str
    ) -> None: ...
    def build(self) -> Ontology: ...

class Reasoner:
    def __init__(
        self,
        path: str | None = None,
        ontology: Ontology | None = None,
        profile: str | None = None,
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @property
    def parse_meta(self) -> ParseMeta: ...
    @property
    def taxonomy(self) -> TaxonomyResult | None: ...
    def classify(self) -> TaxonomyResult | MaterializeResult: ...
    def explain(self) -> ExplainResult: ...
    def check_consistency(self) -> ConsistencyResult: ...
    def is_consistent(self) -> bool: ...
    def is_entailed(
        self,
        sub: str | None = None,
        sup: str | None = None,
        *,
        individual: str | None = None,
        class_: str | None = None,
        subject: str | None = None,
        property: str | None = None,
        object: str | None = None,
    ) -> bool: ...
    def query(self, query: str) -> list[dict[str, str]]: ...
    def add_subclass_of(self, subclass: str, superclass: str) -> None: ...
    def remove_subclass_of(self, subclass: str, superclass: str) -> None: ...
    def add_axiom_json(self, axiom: dict[str, Any]) -> None: ...

class ParseError(Exception): ...
class ResourceLimitError(Exception): ...
class IncompleteReasoningError(Exception): ...
