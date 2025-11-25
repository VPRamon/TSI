"""Insights page data filtering services (deprecated - use impossible_filters)."""

from __future__ import annotations

import warnings

import pandas as pd

# Import from consolidated module
from tsi.services.impossible_filters import (
    apply_insights_filter,
    check_filter_support,
    compute_impossible_mask,
)

# Deprecated: kept for backward compatibility
warnings.warn(
    "insights_filtering is deprecated. Use impossible_filters instead.",
    DeprecationWarning,
    stacklevel=2,
)

