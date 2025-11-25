"""Insights page data filtering services (deprecated - use impossible_filters)."""

from __future__ import annotations

import warnings

# Import from consolidated module

# Deprecated: kept for backward compatibility
warnings.warn(
    "insights_filtering is deprecated. Use impossible_filters instead.",
    DeprecationWarning,
    stacklevel=2,
)
