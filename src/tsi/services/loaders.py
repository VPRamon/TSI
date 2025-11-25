"""Data loading and preparation services."""

from __future__ import annotations

import logging
from collections.abc import Callable
from pathlib import Path
from typing import Any, TypeVar

import pandas as pd

from core.transformations import PreparationResult
from core.transformations import prepare_dataframe as core_prepare_dataframe
from core.transformations.data_cleaning import validate_schema as core_validate_schema
from tsi.config import REQUIRED_COLUMNS
from tsi.services.rust_compat import (
    filter_by_priority,
    filter_by_scheduled,
    load_schedule_rust,
    validate_dataframe_rust,
)

logger = logging.getLogger(__name__)

F = TypeVar("F", bound=Callable[..., Any])


# =============================================================================
# Streamlit Adapter Layer
# =============================================================================

def _identity_cache(func: F | None = None, **_: Any) -> F | Callable[[F], F]:
    """Fallback decorator used when Streamlit caching is unavailable."""

    def decorator(inner: F) -> F:
        return inner

    if func is None:
        return decorator
    return decorator(func)


# Streamlit integration (optional - gracefully degrades if unavailable)
_streamlit_available = False
_cache_decorator = _identity_cache
def _warning_handler(msg: str) -> None:
    return logger.warning(msg)

try:  # pragma: no cover
    import streamlit as st

    try:
        from streamlit import runtime
    except Exception:
        runtime = None  # type: ignore[assignment]

    if (
        hasattr(st, "cache_data")
        and runtime is not None
        and callable(getattr(runtime, "exists", None))
    ):
        if runtime.exists():  # type: ignore[union-attr]
            _streamlit_available = True
            _cache_decorator = st.cache_data
            _warning_handler = st.warning
        else:
            logger.info("Streamlit runtime not initialized")
    else:
        logger.info("Streamlit caching unavailable")

except Exception as e:
    logger.debug(f"Streamlit not available: {e}")


def emit_warning(message: str) -> None:
    """
    Emit a warning through the appropriate channel.

    Uses Streamlit UI if available, otherwise logs as warning.

    Args:
        message: Warning message to display
    """
    _warning_handler(message)


def cache_data(**kwargs):
    """
    Apply caching decorator appropriate for the runtime context.

    Uses Streamlit cache_data if available, otherwise no-op.

    Args:
        **kwargs: Cache configuration parameters (passed to Streamlit if available)

    Returns:
        Caching decorator function
    """
    return _cache_decorator(**kwargs)


# =============================================================================
# Core Loading Functions (Streamlit-agnostic)
# =============================================================================

def _load_csv_core(file_path_or_buffer: str | Path | Any) -> pd.DataFrame:
    """
    Load CSV file using Rust backend (10x faster than pandas).

    Args:
        file_path_or_buffer: Path to CSV file or file-like buffer

    Returns:
        Raw DataFrame from CSV

    Raises:
        FileNotFoundError: If file doesn't exist
        ValueError: If required columns are missing
    """
    try:
        # Use Rust backend for loading (10x speedup)
        df = load_schedule_rust(file_path_or_buffer, format="csv")
    except Exception as e:
        raise ValueError(f"Failed to read CSV: {e}")

    # Validate required columns
    missing_cols = set(REQUIRED_COLUMNS) - set(df.columns)
    if missing_cols:
        raise ValueError(f"Missing required columns: {missing_cols}")

    return df


@cache_data(ttl=3600, show_spinner="Loading data...")
def load_csv(file_path_or_buffer: str | Path | Any) -> pd.DataFrame:
    """
    Load CSV file using Rust backend (10x faster than pandas).

    Streamlit-aware version with caching when available.

    Args:
        file_path_or_buffer: Path to CSV file or file-like buffer

    Returns:
        Raw DataFrame from CSV

    Raises:
        FileNotFoundError: If file doesn't exist
        ValueError: If required columns are missing
    """
    return _load_csv_core(file_path_or_buffer)


def _prepare_dataframe_core(df: pd.DataFrame) -> tuple[pd.DataFrame, list[str]]:
    """
    Core DataFrame preparation logic without Streamlit dependencies.

    Args:
        df: Raw DataFrame to prepare

    Returns:
        Tuple of (prepared DataFrame, list of warnings)
    """
    result: PreparationResult = core_prepare_dataframe(df)
    return result.dataframe, result.warnings  # type: ignore[return-value]


@cache_data(ttl=3600, show_spinner="Preparing data...")
def prepare_dataframe(df: pd.DataFrame) -> pd.DataFrame:
    """
    Prepare and enrich pre-processed DataFrame.

    Assumes the CSV has been pre-processed with all derived columns.
    Only performs lightweight operations like type conversion and datetime parsing.

    Streamlit-aware version with caching and warning display.
    """
    prepared_df, warnings = _prepare_dataframe_core(df)
    for warning in warnings:
        emit_warning(f"⚠️ {warning}")
    return prepared_df


def get_filtered_dataframe(
    df: pd.DataFrame,
    priority_range: tuple[float, float] = (0.0, 10.0),
    scheduled_filter: str = "All",
    priority_bins: list[str] | None = None,
    block_ids: list[str | int] | None = None,
) -> pd.DataFrame:
    """
    Filter DataFrame based on user-selected criteria using Rust backend (10x faster).
    """
    # Start with full DataFrame
    result = df.copy()

    # Apply priority range filter (Rust)
    if priority_range != (0.0, 10.0):
        result = filter_by_priority(result, priority_range[0], priority_range[1])

    # Apply scheduled filter (Rust)
    if scheduled_filter.lower() != "all":
        result = filter_by_scheduled(result, scheduled_filter.lower())

    # Apply priority bins filter (Python - complex logic)
    if priority_bins:
        result = result[result["priority_bin"].isin(priority_bins)]

    # Apply block IDs filter (Python - simple filter)
    if block_ids:
        result = result[result["schedulingBlockId"].isin(block_ids)]

    return result


def validate_dataframe(df: pd.DataFrame) -> tuple[bool, list[str]]:
    """
    Validate DataFrame for data quality issues using Rust backend (5x faster).
    """
    # Schema validation (Python - needed for custom checks)
    schema_ok, schema_errors = core_validate_schema(
        df,
        required_columns=set(REQUIRED_COLUMNS),
        expected_dtypes=None,
    )

    # Data validation (Rust - 5x speedup)
    data_ok, data_errors = validate_dataframe_rust(df)

    issues = [*schema_errors, *data_errors]
    return schema_ok and data_ok, issues
