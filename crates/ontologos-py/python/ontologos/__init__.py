"""OntoLogos — Python bindings for the OntoLogos OWL reasoner (pre-release)."""

from __future__ import annotations

__version__ = "0.6.1"

try:
    from ontologos._ontologos import Reasoner
except ImportError:  # pragma: no cover - platform wheel not installed
    Reasoner = None  # type: ignore[assignment,misc]

__all__ = ["Reasoner", "__version__"]
