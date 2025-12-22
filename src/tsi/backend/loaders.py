"""
TSI Backend Loaders - Data loading utilities.

This module contains functions for loading schedule data from JSON files
and strings via the Rust backend or fallback to pandas.

Note: The Rust backend is primarily designed to work with a database.
      File loading functions fall back to pandas when direct Rust file
      loading is not available.

      CSV format is no longer supported - use JSON only.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal, cast

import pandas as pd

if TYPE_CHECKING:
    pass

# Import Rust module (available after core module validates it)


def load_schedule_file(
    path: str | Path,
    format: Literal["auto", "json"] = "auto",
    use_pandas: bool = True,
) -> pd.DataFrame:
    """
    Load schedule data from JSON file.

    Args:
        path: Path to the schedule file
        format: File format ('auto' or 'json'). Auto-detects from extension.
        use_pandas: If True, return pandas DataFrame. If False, return Polars DataFrame.

    Returns:
        DataFrame with scheduling blocks and derived columns

    Example:
        >>> df = load_schedule_file("data/schedule.json")
        >>> print(df.columns)
    """
    path = Path(path)

    if format == "auto":
        if path.suffix != ".json":
            raise ValueError(f"Only JSON files are supported. Got: {path.suffix}")
        format = "json"

    if format == "json":
        # Try Rust's JSON string loading via reading file first
        content = path.read_text()
        return load_schedule_from_string(content, format="json", use_pandas=use_pandas)
    else:
        raise ValueError(f"Unknown format: {format}")


def load_schedule_from_string(
    content: str,
    format: Literal["json"] = "json",
    use_pandas: bool = True,
) -> pd.DataFrame:
    """
    Load schedule data from JSON string content.

    Args:
        content: JSON string content
        format: Format of the content ('json' only)
        use_pandas: If True, return pandas DataFrame. If False, return Polars DataFrame.

    Returns:
        DataFrame with scheduling blocks

    Example:
        >>> json_str = '{"SchedulingBlock": [...]}'
        >>> df = load_schedule_from_string(json_str, format="json")
    """
    import json as json_module

    if format != "json":
        raise ValueError(f"Only JSON format is supported. Got: {format}")

    # Parse JSON and extract scheduling blocks
    data = json_module.loads(content)
    # Handle different JSON structures
    if "SchedulingBlock" in data:
        blocks = data["SchedulingBlock"]
    elif "schedulingBlocks" in data:
        blocks = data["schedulingBlocks"]
    else:
        blocks = data if isinstance(data, list) else [data]

    df_pandas = pd.DataFrame(blocks)
    return df_pandas


def load_dark_periods(path: str | Path) -> pd.DataFrame:
    """
    Load dark periods data from JSON file.

    Args:
        path: Path to dark_periods.json file

    Returns:
        pandas DataFrame with columns: start_dt, stop_dt, start_mjd, stop_mjd,
        duration_hours, months

    Example:
        >>> df = load_dark_periods("data/dark_periods.json")
        >>> print(f"Loaded {len(df)} dark periods")
    """
    import json as json_module

    path = Path(path)
    with open(path) as f:
        data = json_module.load(f)

    # Handle different JSON structures
    if "dark_periods" in data:
        periods = data["dark_periods"]
    elif isinstance(data, list):
        periods = data
    else:
        periods = [data]

    return pd.DataFrame(periods)


def load_schedule_from_any(
    source: str | Path | Any,
    format: Literal["auto", "json"] = "auto",
    use_pandas: bool = True,
) -> pd.DataFrame:
    """
    Load schedule data from a path or file-like object via the Rust backend.

    This is a convenience function that handles both file paths and file-like
    objects (e.g., uploaded files in Streamlit).

    Args:
        source: File path (str/Path) or file-like object with read() method
        format: File format ('auto' or 'json'). Must be specified for buffers.
        use_pandas: If True, return pandas DataFrame.

    Returns:
        DataFrame with schedule data
    """
    if hasattr(source, "read"):
        content = source.read()
        if isinstance(content, bytes):
            content = content.decode("utf-8")
        if hasattr(source, "seek"):
            source.seek(0)

        if format == "auto":
            raise ValueError("Format must be specified when reading from a buffer")
        if format == "json":
            return cast(
                pd.DataFrame,
                load_schedule_from_string(content, format="json", use_pandas=use_pandas),
            )
        raise ValueError(f"Unsupported format: {format}")

    return cast(
        pd.DataFrame, load_schedule_file(Path(source), format=format, use_pandas=use_pandas)
    )
