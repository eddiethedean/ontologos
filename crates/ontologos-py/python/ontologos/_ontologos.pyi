from typing import TypedDict

class ParseMeta(TypedDict):
    warnings: list[str]
    mapped_axiom_count: int
    skipped_axiom_count: int
    logical_axiom_count: int

class Reasoner:
    def __init__(self, path: str, profile: str | None = None) -> None: ...
    @property
    def parse_meta(self) -> ParseMeta: ...
    def classify(self) -> None: ...
