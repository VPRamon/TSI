"""
TSI Rust Backend - Python Integration Layer (Compatibility Shim)

This module provides backward compatibility for code that imports from tsi_rust_api.
All functionality has been moved to the tsi.backend package.

For new code, please import from tsi.backend directly:
    >>> from tsi.backend import TSIBackend

Or using the functional API:
    >>> from tsi.backend import load_schedule_file, compute_metrics

Example:
    >>> from tsi_rust_api import TSIBackend
    >>>
    >>> # Initialize backend
    >>> backend = TSIBackend()
    >>>
    >>> # Load schedule data
    >>> df = backend.load_schedule("data/schedule.json")
    >>> print(f"Loaded {len(df)} observations")
    >>>
    >>> # Compute analytics
    >>> metrics = backend.compute_metrics(df)
    >>> print(f"Scheduled: {metrics['scheduled_count']}/{metrics['total_observations']}")
    >>>
    >>> # Filter data
    >>> high_priority = backend.filter_by_priority(df, min_priority=15.0)
    >>> print(f"High priority observations: {len(high_priority)}")
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, cast

import pandas as pd

from tsi.backend import (
    compute_metrics as _compute_metrics,
)
from tsi.backend import (
    filter_by_priority as _filter_by_priority,
)
from tsi.backend import (
    load_dark_periods as _load_dark_periods,
)
from tsi.backend import (
    load_schedule_file,
)

# Re-export from new location
from tsi.backend.core import TSIBackend

# Also expose Rust module availability check
try:
    import tsi_rust  # noqa: F401 - re-export for backwards compatibility
except ImportError:
    from tsi.exceptions import ServerError

    raise ServerError(
        "tsi_rust module not found. Please compile the Rust backend with: maturin develop --release",
        details={"install_command": "maturin develop --release"},
    )


# Convenience functions (preserved for backward compatibility)


def load_schedule(path: str | Path, **kwargs: Any) -> pd.DataFrame:
    """
    Quick function to load schedule data. Returns pandas DataFrame.

    Note: For new code, use tsi.backend.load_schedule_file directly.
    """
    return cast(pd.DataFrame, load_schedule_file(path, use_pandas=True, **kwargs))


def load_dark_periods(path: str | Path) -> pd.DataFrame:
    """
    Quick function to load dark periods data.

    Args:
        path: Path to dark_periods.json file

    Returns:
        pandas DataFrame with columns: start_dt, stop_dt, start_mjd, stop_mjd,
        duration_hours, months

    Example:
        >>> from tsi_rust_api import load_dark_periods
        >>> df = load_dark_periods("data/dark_periods.json")
        >>> print(f"Loaded {len(df)} dark periods")
    """
    return _load_dark_periods(path)  # type: ignore[no-any-return]


def compute_metrics(df: pd.DataFrame) -> dict[str, Any]:
    """
    Quick function to compute scheduling metrics.

    Note: For new code, use tsi.backend.compute_metrics directly.
    """
    return _compute_metrics(df)  # type: ignore[no-any-return]


def filter_by_priority(
    df: pd.DataFrame, min_priority: float = 0.0, max_priority: float = 100.0
) -> pd.DataFrame:
    """
    Quick function to filter by priority range.

    Note: For new code, use tsi.backend.filter_by_priority directly.
    """
    return cast(pd.DataFrame, _filter_by_priority(df, min_priority, max_priority, use_pandas=True))


# Version info
__version__ = "0.1.0"
__all__ = [
    "TSIBackend",
    "load_schedule",
    "load_dark_periods",
    "compute_metrics",
    "filter_by_priority",
]
