"""OntoLogos — Python bindings for the OntoLogos OWL reasoner."""

from __future__ import annotations

__version__ = "1.1.4"

try:
    from ontologos._ontologos import (
        IncompleteReasoningError,
        Ontology,
        OntologyBuilder,
        ParseError,
        Reasoner,
        ResourceLimitError,
    )
except ImportError as exc:  # pragma: no cover - platform wheel not installed
    raise ImportError(
        "ontologos native extension is not installed; "
        "run `maturin develop --release` from crates/ontologos-py"
    ) from exc

from ontologos.export import subsumptions_to_pandas, subsumptions_to_polars
from ontologos.types import (
    ClassifyResult,
    ConsistencyResult,
    ExplainResult,
    MaterializeResult,
    ParseMeta,
    ProofNode,
    TaxonomyResult,
)

__all__ = [
    "Ontology",
    "OntologyBuilder",
    "Reasoner",
    "ParseError",
    "ResourceLimitError",
    "IncompleteReasoningError",
    "ClassifyResult",
    "ConsistencyResult",
    "ExplainResult",
    "MaterializeResult",
    "ParseMeta",
    "ProofNode",
    "TaxonomyResult",
    "__version__",
    "subsumptions_to_pandas",
    "subsumptions_to_polars",
]
