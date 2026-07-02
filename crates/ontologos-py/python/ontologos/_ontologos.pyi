from typing import Any, Generic, Literal, TypeVar, overload

from ontologos.types import (
    ConsistencyResult,
    ExplainResult,
    MaterializeResult,
    ParseMeta,
    TaxonomyResult,
)

MaterializeProfile = Literal["rdfs", "rl"]
TaxonomyProfile = Literal["el", "dl", "dl-preview", "alc", "swrl"]
AutoProfile = Literal["auto"]
ProfileT = TypeVar(
    "ProfileT",
    MaterializeProfile,
    TaxonomyProfile,
    AutoProfile,
    None,
    default=None,
)

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

class Reasoner(Generic[ProfileT]):
    @overload
    def __init__(
        self: Reasoner[Literal["rdfs"]],
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: Literal["rdfs"],
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @overload
    def __init__(
        self: Reasoner[Literal["rl"]],
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: Literal["rl"],
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @overload
    def __init__(
        self: Reasoner[Literal["el"]],
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: Literal["el"],
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @overload
    def __init__(
        self: Reasoner[Literal["dl"]],
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: Literal["dl"],
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @overload
    def __init__(
        self: Reasoner[Literal["dl-preview"]],
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: Literal["dl-preview"],
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @overload
    def __init__(
        self: Reasoner[Literal["alc"]],
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: Literal["alc"],
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @overload
    def __init__(
        self: Reasoner[Literal["swrl"]],
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: Literal["swrl"],
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @overload
    def __init__(
        self: Reasoner[Literal["auto"]],
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: Literal["auto"] = "auto",
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @overload
    def __init__(
        self: Reasoner[None],
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: None = None,
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @overload
    def __init__(
        self,
        path: str | None = None,
        ontology: Ontology | None = None,
        *,
        profile: str | None = None,
        incremental: bool = False,
        budget_secs: int | None = None,
    ) -> None: ...
    @property
    def parse_meta(self) -> ParseMeta: ...
    @property
    def taxonomy(self) -> TaxonomyResult | None: ...
    @overload
    def classify(self: Reasoner[MaterializeProfile]) -> MaterializeResult: ...
    @overload
    def classify(self: Reasoner[TaxonomyProfile]) -> TaxonomyResult: ...
    @overload
    def classify(self: Reasoner[AutoProfile]) -> TaxonomyResult | MaterializeResult: ...
    @overload
    def classify(self: Reasoner[None]) -> TaxonomyResult | MaterializeResult: ...
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
