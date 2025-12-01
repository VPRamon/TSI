"""
TSI Backend Package.

This package provides a clean Python interface to the Rust-based
TSI scheduling backend. It is organized into modules by concern:

- core: The main TSIBackend class
- loaders: Data loading utilities (load_schedule, load_dark_periods)
- analytics: Metrics and scheduling algorithms
- transformations: Filtering and data transformations

Example:
    >>> from tsi.backend import TSIBackend
    >>> backend = TSIBackend()
    >>> df = backend.load_schedule("data/schedule.json")
    >>> metrics = backend.compute_metrics(df)

Or using the functional API:
    >>> from tsi.backend import load_schedule_file, compute_metrics
    >>> df = load_schedule_file("data/schedule.json")
    >>> metrics = compute_metrics(df)
"""

from __future__ import annotations

# Re-export from submodules for convenience
from tsi.backend.core import TSIBackend
from tsi.backend.loaders import (
    load_dark_periods,
    load_schedule_file,
    load_schedule_from_any,
    load_schedule_from_string,
)
from tsi.backend.analytics import (
    compute_metrics,
    compute_priority_range,
    find_conflicts,
    get_top_observations,
    greedy_schedule,
)
from tsi.backend.transformations import (
    datetime_to_mjd,
    filter_by_priority,
    filter_by_scheduled,
    filter_dataframe,
    mjd_to_datetime,
    parse_visibility_periods,
    remove_duplicates,
    remove_missing_coordinates,
    validate_dataframe,
)

__all__ = [
    # Main class
    "TSIBackend",
    # Loaders
    "load_schedule_file",
    "load_schedule_from_string",
    "load_schedule_from_any",
    "load_dark_periods",
    # Analytics
    "compute_metrics",
    "get_top_observations",
    "find_conflicts",
    "greedy_schedule",
    "compute_priority_range",
    # Transformations
    "filter_by_priority",
    "filter_by_scheduled",
    "filter_dataframe",
    "remove_duplicates",
    "remove_missing_coordinates",
    "validate_dataframe",
    # Time utilities
    "mjd_to_datetime",
    "datetime_to_mjd",
    "parse_visibility_periods",
]
