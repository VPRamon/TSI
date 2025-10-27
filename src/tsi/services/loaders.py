"""Data loading and preparation services."""

from __future__ import annotations

from collections.abc import Callable
from pathlib import Path
from typing import Any, TypeVar

import pandas as pd

from core.transformations import PreparationResult
from core.transformations import filter_dataframe as core_filter_dataframe
from core.transformations import prepare_dataframe as core_prepare_dataframe
from core.transformations import validate_dataframe as core_validate_dataframe
from core.transformations.data_cleaning import validate_schema as core_validate_schema
from tsi.config import REQUIRED_COLUMNS

F = TypeVar("F", bound=Callable[..., Any])


def _identity_cache(func: F | None = None, **_: Any) -> F | Callable[[F], F]:
    """Fallback decorator used when Streamlit caching is unavailable."""

    def decorator(inner: F) -> F:
        return inner

    if func is None:
        return decorator
    return decorator(func)


try:  # pragma: no cover - exercised indirectly in tests
    import streamlit as st

    try:
        from streamlit import runtime
    except Exception:  # runtime module missing (older versions)
        runtime = None  # type: ignore[assignment]

    if (
        hasattr(st, "cache_data")
        and runtime is not None
        and callable(getattr(runtime, "exists", None))
    ):
        if runtime.exists():  # type: ignore[union-attr]
            cache_data = st.cache_data

            def emit_warning(message: str) -> None:
                st.warning(message)

        else:
            raise RuntimeError("Streamlit runtime not initialized")
    else:
        raise RuntimeError("Streamlit caching unavailable")

except Exception:  # pragma: no cover - triggered in test environment
    st = None  # type: ignore[assignment]
    cache_data = _identity_cache  # type: ignore[assignment]

    def emit_warning(message: str) -> None:
        return None


@cache_data(ttl=3600, show_spinner="Loading data...")
def load_csv(file_path_or_buffer: str | Path | Any) -> pd.DataFrame:
    """
    Load CSV file into a pandas DataFrame with basic validation.

    Args:
        file_path_or_buffer: Path to CSV file or file-like buffer

    Returns:
        Raw DataFrame from CSV

    Raises:
        FileNotFoundError: If file doesn't exist
        ValueError: If required columns are missing
    """
    try:
        df = pd.read_csv(file_path_or_buffer)
    except Exception as e:
        raise ValueError(f"Failed to read CSV: {e}")

    # Validate required columns
    missing_cols = set(REQUIRED_COLUMNS) - set(df.columns)
    if missing_cols:
        raise ValueError(f"Missing required columns: {missing_cols}")

    return df


@cache_data(ttl=3600, show_spinner="Preparing data...")
def prepare_dataframe(df: pd.DataFrame) -> pd.DataFrame:
    """
    Prepare and enrich pre-processed DataFrame.

    Assumes the CSV has been pre-processed with all derived columns.
    Only performs lightweight operations like type conversion and datetime parsing.
    """
    result: PreparationResult = core_prepare_dataframe(df)
    for warning in result.warnings:
        emit_warning(f"⚠️ {warning}")
    return result.dataframe  # type: ignore[return-value,no-any-return]


def get_filtered_dataframe(
    df: pd.DataFrame,
    priority_range: tuple[float, float] = (0.0, 10.0),
    scheduled_filter: str = "All",
    priority_bins: list[str] | None = None,
) -> pd.DataFrame:
    """
    Filter DataFrame based on user-selected criteria.
    """
    result = core_filter_dataframe(
        df,
        priority_range=priority_range,
        scheduled_filter=scheduled_filter,  # type: ignore[arg-type]
        priority_bins=priority_bins or [],
    )
    return result  # type: ignore[return-value,no-any-return]


def validate_dataframe(df: pd.DataFrame) -> tuple[bool, list[str]]:
    """
    Validate DataFrame for data quality issues.
    """
    schema_ok, schema_errors = core_validate_schema(
        df,
        required_columns=set(REQUIRED_COLUMNS),
        expected_dtypes=None,
    )
    data_ok, data_errors = core_validate_dataframe(df)
    issues = [*schema_errors, *data_errors]
    return schema_ok and data_ok, issues
