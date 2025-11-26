"""Distribution data filtering services (deprecated - use impossible_filters)."""

from __future__ import annotations

import warnings
from typing import cast

import pandas as pd

# Import from consolidated module
from tsi.services.impossible_filters import (
    filter_impossible_observations as _filter_impossible_observations,
)

# Deprecated: kept for backward compatibility
warnings.warn(
    "distributions_filters is deprecated. Use impossible_filters instead.",
    DeprecationWarning,
    stacklevel=2,
)


def filter_impossible_observations(df: pd.DataFrame, filter_mode: str) -> pd.DataFrame:
    """
    Filter out impossible observations based on visibility constraints.

    **DEPRECATED**: Use `impossible_filters.filter_impossible_observations` instead.

    Args:
        df: Source DataFrame
        filter_mode: One of "all" or "exclude_impossible"

    Returns:
        Filtered DataFrame (view if no filtering, copy if filtered)
    """
    return cast(pd.DataFrame, _filter_impossible_observations(df, filter_mode))
