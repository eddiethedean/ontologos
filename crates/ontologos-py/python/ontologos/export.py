"""Optional taxonomy export helpers for pandas and polars."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import pandas as pd
    import polars as pl


def _subsumption_rows(taxonomy: dict[str, Any]) -> list[tuple[str, str]]:
    subs = taxonomy.get("subsumptions", [])
    return [(str(sub), str(sup)) for sub, sup in subs]


def subsumptions_to_pandas(taxonomy: dict[str, Any]) -> pd.DataFrame:
    """Return a DataFrame with columns ``subclass`` and ``superclass``."""
    import pandas as pd

    rows = _subsumption_rows(taxonomy)
    return pd.DataFrame(rows, columns=["subclass", "superclass"])


def subsumptions_to_polars(taxonomy: dict[str, Any]) -> pl.DataFrame:
    """Return a Polars DataFrame with columns ``subclass`` and ``superclass``."""
    import polars as pl

    rows = _subsumption_rows(taxonomy)
    return pl.DataFrame(rows, schema=["subclass", "superclass"], orient="row")
