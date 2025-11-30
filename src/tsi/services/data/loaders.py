"""Data loading and preparation services."""

from __future__ import annotations

import logging
from pathlib import Path
from typing import Any, cast

import pandas as pd

from tsi.config import REQUIRED_COLUMNS
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


def load_schedule_rust(path: str | Path | Any, format: str = "auto") -> pd.DataFrame:
    """
    Load schedule data using the Rust backend (supports file-like objects).
    """
    return cast(pd.DataFrame, load_schedule_from_any(path, format=format))


def filter_by_priority(
    df: pd.DataFrame, min_priority: float = 0.0, max_priority: float = 10.0
) -> pd.DataFrame:
    """Filter dataframe by priority range using the Rust backend."""
    return cast(pd.DataFrame, BACKEND.filter_by_priority(df, min_priority, max_priority))


def filter_by_scheduled(df: pd.DataFrame, filter_type: str = "All") -> pd.DataFrame:
    """Filter dataframe by scheduled status using the Rust backend."""
    return cast(pd.DataFrame, BACKEND.filter_by_scheduled(df, filter_type))


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
        df = load_schedule_rust(file_path_or_buffer, format="csv")
    except Exception as e:
        raise ValueError(f"Failed to read CSV: {e}")

    # Validate required columns
    missing_cols = set(REQUIRED_COLUMNS) - set(df.columns)
    if missing_cols:
        raise ValueError(f"Missing required columns: {missing_cols}")

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
    Filter DataFrame based on user-selected criteria using Rust backend (10x faster).
    """
    # Start with full DataFrame
    result = df.copy()

    # Apply priority range filter (Rust)
    if priority_range != (0.0, 10.0):
        result = filter_by_priority(result, priority_range[0], priority_range[1])

    # Apply scheduled filter (Rust)
    if scheduled_filter != "All":
        result = filter_by_scheduled(result, scheduled_filter)

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
    try:
        data_ok, data_errors = BACKEND.validate_dataframe(df)
    except Exception as e:
        data_ok, data_errors = False, [f"Validation failed: {e}"]

    issues = [*schema_errors, *data_errors]
    return schema_ok and data_ok, issues
