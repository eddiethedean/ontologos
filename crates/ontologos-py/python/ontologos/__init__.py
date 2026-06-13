"""OntoLogos — Python bindings for the OntoLogos OWL reasoner."""

from __future__ import annotations

__version__ = "0.9.0"

try:
    from ontologos._ontologos import Ontology, OntologyBuilder, Reasoner
except ImportError:  # pragma: no cover - platform wheel not installed
    Ontology = None  # type: ignore[assignment,misc]
    OntologyBuilder = None  # type: ignore[assignment,misc]
    Reasoner = None  # type: ignore[assignment,misc]

from ontologos.export import subsumptions_to_pandas, subsumptions_to_polars

__all__ = [
    "Ontology",
    "OntologyBuilder",
    "Reasoner",
    "__version__",
    "subsumptions_to_pandas",
    "subsumptions_to_polars",
]
