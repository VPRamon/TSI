"""Data loading and preparation services."""

from __future__ import annotations

import logging
from pathlib import Path
from typing import Any, cast

import pandas as pd

from tsi.config import REQUIRED_COLUMNS
from tsi.exceptions import DataLoadError, SchemaError, DataValidationError
from tsi.error_handling import with_retry, log_error
from tsi.services.data.preparation import PreparationResult
from tsi.services.data.preparation import prepare_dataframe as core_prepare_dataframe
from tsi.services.data.preparation import validate_schema as core_validate_schema
from tsi.services.rust_backend import BACKEND, load_schedule_from_any

logger = logging.getLogger(__name__)


def emit_warning(message: str) -> None:
    """Display warnings in Streamlit when available, otherwise log."""
    try:
        import streamlit as st

        st.warning(message)
    except Exception:
        logger.warning(message)


@with_retry(max_attempts=2, backoff_factor=1.5)
def load_schedule_rust(path: str | Path | Any, format: str = "auto") -> pd.DataFrame:
    """
    Load schedule data using the Rust backend (supports file-like objects).
    
    Args:
        path: Path to schedule file or file-like object
        format: File format ('auto', 'csv', or 'json')
        
    Returns:
        DataFrame with schedule data
        
    Raises:
        DataLoadError: If loading fails
    """
    try:
        return cast(pd.DataFrame, load_schedule_from_any(path, format=format))
    except Exception as e:
        raise DataLoadError(
            f"Failed to load schedule from {path}",
            details={"path": str(path), "format": format, "error": str(e)}
        ) from e


def filter_by_priority(
    df: pd.DataFrame, min_priority: float = 0.0, max_priority: float = 10.0
) -> pd.DataFrame:
    """
    Filter dataframe by priority range.

    Note:
        This is a thin wrapper around BACKEND.filter_by_priority.
        For full filtering with multiple criteria, use get_filtered_dataframe().
    """
    return cast(pd.DataFrame, BACKEND.filter_by_priority(df, min_priority, max_priority))


def filter_by_scheduled(df: pd.DataFrame, filter_type: str = "All") -> pd.DataFrame:
    """
    Filter dataframe by scheduled status.

    Note:
        This is a thin wrapper around BACKEND.filter_by_scheduled.
        For full filtering with multiple criteria, use get_filtered_dataframe().
    """
    return cast(pd.DataFrame, BACKEND.filter_by_scheduled(df, filter_type))  # type: ignore[arg-type]


def _load_csv_core(file_path_or_buffer: str | Path | Any) -> pd.DataFrame:
    """
    Load CSV file using Rust backend (10x faster than pandas).

    Args:
        file_path_or_buffer: Path to CSV file or file-like buffer

    Returns:
        Raw DataFrame from CSV

    Raises:
        DataLoadError: If file loading fails
        SchemaError: If required columns are missing
    """
    try:
        df = load_schedule_rust(file_path_or_buffer, format="csv")
    except DataLoadError:
        # Re-raise DataLoadError as-is
        raise
    except Exception as e:
        raise DataLoadError(
            "Failed to read CSV",
            details={"error": str(e)}
        ) from e

    # Validate required columns
    missing_cols = set(REQUIRED_COLUMNS) - set(df.columns)
    if missing_cols:
        raise SchemaError(
            f"Missing required columns",
            details={"missing_columns": sorted(missing_cols), "found_columns": sorted(df.columns)}
        )

    return cast(pd.DataFrame, df)


def load_csv(file_path_or_buffer: str | Path | Any) -> pd.DataFrame:  # type: ignore[no-any-return]
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
    return cast(pd.DataFrame, _load_csv_core(file_path_or_buffer))  # type: ignore[no-any-return]


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


def prepare_dataframe(df: pd.DataFrame) -> pd.DataFrame:
    """
    Prepare and enrich pre-processed DataFrame.

    Assumes the CSV has been pre-processed with all derived columns.
    Only performs lightweight operations like type conversion and datetime parsing.
    """
    prepared_df, warnings = _prepare_dataframe_core(df)
    for warning in warnings:
        emit_warning(f"⚠️ {warning}")
    return cast(pd.DataFrame, prepared_df)  # type: ignore[no-any-return]


def get_filtered_dataframe(
    df: pd.DataFrame,
    priority_range: tuple[float, float] = (0.0, 10.0),
    scheduled_filter: str = "All",
    priority_bins: list[str] | None = None,
    block_ids: list[str | int] | None = None,
) -> pd.DataFrame:
    """
    Filter DataFrame based on user-selected criteria using Rust backend.

    This is the canonical filtering function - all filtering should go through
    this function which delegates to the Rust backend for performance.

    Args:
        df: DataFrame to filter
        priority_range: Tuple of (min_priority, max_priority)
        scheduled_filter: 'All', 'Scheduled', or 'Unscheduled'
        priority_bins: Optional list of priority bin labels to include
        block_ids: Optional list of scheduling block IDs to include

    Returns:
        Filtered DataFrame

    Note:
        Uses Rust backend (TSIBackend.filter_dataframe) for 10x faster filtering.
    """
    # Convert block_ids to strings for Rust backend
    block_ids_str = [str(bid) for bid in block_ids] if block_ids else None

    return cast(
        pd.DataFrame,
        BACKEND.filter_dataframe(
            df,
            priority_min=priority_range[0],
            priority_max=priority_range[1],
            scheduled_filter=scheduled_filter,  # type: ignore[arg-type]
            priority_bins=priority_bins,
            block_ids=block_ids_str,
        ),
    )


def validate_dataframe(df: pd.DataFrame) -> tuple[bool, list[str]]:
    """
    Canonical DataFrame validation combining schema and data quality checks.

    This is the main validation entry point. It performs:
    1. Schema validation (Python): Checks required columns exist
    2. Data validation (Rust): Checks coordinate ranges, priority validity, etc.

    Args:
        df: DataFrame to validate

    Returns:
        Tuple of (is_valid, list of error/warning messages)

    Note:
        Uses Rust backend for data validation (5x faster than pure Python).
        For schema-only validation, use `preparation.validate_schema()` directly.
    """
    # Schema validation (Python - needed for custom checks)
    schema_ok, schema_errors = core_validate_schema(
        df,
        required_columns=set(REQUIRED_COLUMNS),
        expected_dtypes=None,
    )

    # Data validation (Rust - 5x speedup)
    try:
        data_ok, data_errors = BACKEND.validate_dataframe(df)
    except Exception as e:
        data_ok, data_errors = False, [f"Validation failed: {e}"]

    issues = [*schema_errors, *data_errors]
    return schema_ok and data_ok, issues
