"""
Schedule Loader - Unified data loading interface

This module provides functions to load scheduling data from various sources
(JSON files, CSV files, iteration directories) into standardized DataFrames.

It centralizes the preprocessing logic previously scattered across notebooks
and the Streamlit app, ensuring consistent data processing.
"""

import io
import json
import logging
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import pandas as pd

try:
    import tsi_rust
except ImportError:
    raise ImportError(
        "tsi_rust module not found. Please compile the Rust backend with: "
        "maturin develop --release"
    )

logger = logging.getLogger(__name__)


@dataclass
class ValidationResult:
    """Result of data validation checks."""

    is_valid: bool
    errors: list[str]
    warnings: list[str]
    stats: dict


@dataclass
class ScheduleLoadResult:
    """Result of loading schedule data."""

    dataframe: pd.DataFrame
    validation: ValidationResult
    source_type: str  # 'json', 'csv', 'iteration'
    source_path: str | None = None


def load_schedule_from_json(
    schedule_json: str | Path | io.IOBase | dict,
    visibility_json: str | Path | io.IOBase | dict | None = None,
    validate: bool = True,
) -> ScheduleLoadResult:
    """
    Load scheduling data directly from JSON file(s) or file-like objects.

    This function processes JSON data in memory without creating intermediate CSV files.
    Ideal for Streamlit file uploads and programmatic use in notebooks.

    Args:
        schedule_json: Path to schedule.json, file-like object, or parsed dict
        visibility_json: Optional path to possible_periods.json, file-like object, or dict
        validate: Whether to validate the resulting DataFrame

    Returns:
        ScheduleLoadResult with DataFrame and validation info

    Examples:
        >>> # From file paths
        >>> result = load_schedule_from_json('data/schedule.json', 'data/possible_periods.json')
        >>> df = result.dataframe

        >>> # From Streamlit uploaded files
        >>> schedule_file = st.file_uploader("Upload schedule.json")
        >>> result = load_schedule_from_json(schedule_file)

        >>> # From parsed JSON
        >>> with open('schedule.json') as f:
        ...     data = json.load(f)
        >>> result = load_schedule_from_json(data)
    """
    logger.info("Loading schedule from JSON...")

    # Handle different input types for schedule
    schedule_data = None
    schedule_path_str = None
    source_path_display = None
    
    if isinstance(schedule_json, dict):
        # Already parsed JSON
        schedule_data = schedule_json
        source_path_display = "parsed_dict"
    elif isinstance(schedule_json, (str, Path)):
        # File path - can use directly with Rust
        schedule_path = Path(schedule_json)
        if not schedule_path.exists():
            raise FileNotFoundError(f"Schedule file not found: {schedule_path}")
        schedule_path_str = str(schedule_path)
        source_path_display = schedule_path_str
    elif hasattr(schedule_json, "read"):
        # File-like object (e.g., Streamlit UploadedFile) - need to load into memory
        content = schedule_json.read()
        if isinstance(content, bytes):
            content = content.decode("utf-8")
        schedule_data = json.loads(content)
        source_path_display = getattr(schedule_json, "name", "uploaded_file")
        # Reset file pointer if possible
        if hasattr(schedule_json, "seek"):
            schedule_json.seek(0)
    else:
        raise TypeError(f"Unsupported schedule_json type: {type(schedule_json)}")

    # Handle visibility data
    visibility_data = None
    visibility_path_str = None
    if visibility_json is not None:
        if isinstance(visibility_json, dict):
            visibility_data = visibility_json
        elif isinstance(visibility_json, (str, Path)):
            visibility_path = Path(visibility_json)
            if visibility_path.exists():
                visibility_path_str = str(visibility_path)
        elif hasattr(visibility_json, "read"):
            content = visibility_json.read()
            if isinstance(content, bytes):
                content = content.decode("utf-8")
            visibility_data = json.loads(content)
            if hasattr(visibility_json, "seek"):
                visibility_json.seek(0)

    # Use Rust backend for preprocessing
    num_blocks = len(schedule_data.get('SchedulingBlock', [])) if schedule_data else "schedule"
    logger.info(f"Processing {num_blocks} blocks using Rust backend...")
    
    # Call Rust preprocessing - returns (DataFrame, ValidationResult)
    # If we have file paths, use them directly; otherwise use temp files
    if schedule_path_str is not None:
        # We have a real file path - use it directly
        df_polars, rust_validation = tsi_rust.py_preprocess_schedule(schedule_path_str, visibility_path_str, validate)
    else:
        # For dict/file-like objects, we need to save to temp files
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as schedule_tmp:
            json.dump(schedule_data, schedule_tmp)
            schedule_tmp_path = schedule_tmp.name
        
        visibility_tmp_path = None
        if visibility_data:
            with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as vis_tmp:
                json.dump(visibility_data, vis_tmp)
                visibility_tmp_path = vis_tmp.name
        
        try:
            df_polars, rust_validation = tsi_rust.py_preprocess_schedule(schedule_tmp_path, visibility_tmp_path, validate)
        finally:
            Path(schedule_tmp_path).unlink()
            if visibility_tmp_path:
                Path(visibility_tmp_path).unlink()

    # Convert Polars DataFrame to pandas
    df = df_polars.to_pandas()

    # Convert Rust validation result to Python ValidationResult
    if validate:
        stats = rust_validation.get_stats() if hasattr(rust_validation, 'get_stats') else {}
        validation = ValidationResult(
            is_valid=rust_validation.is_valid,
            errors=rust_validation.errors,
            warnings=rust_validation.warnings,
            stats=stats
        )
    else:
        validation = ValidationResult(is_valid=True, errors=[], warnings=[], stats={})

    if validate and not validation.is_valid:
        logger.warning(f"Validation found {len(validation.errors)} errors")
        for error in validation.errors:
            logger.error(f"  - {error}")

    if validate and validation.warnings:
        logger.info(f"Validation found {len(validation.warnings)} warnings")
        for warning in validation.warnings[:5]:  # Show first 5
            logger.warning(f"  - {warning}")

    logger.info(f"Successfully loaded {len(df)} scheduling blocks")

    return ScheduleLoadResult(
        dataframe=df,
        validation=validation,
        source_type="json",
        source_path=source_path_display,
    )


def load_schedule_from_csv(
    csv_path: str | Path | io.IOBase,
    validate: bool = True,
) -> ScheduleLoadResult:
    """
    Load scheduling data from a preprocessed CSV file.

    Args:
        csv_path: Path to CSV file or file-like object
        validate: Whether to validate the DataFrame

    Returns:
        ScheduleLoadResult with DataFrame and validation info

    Examples:
        >>> result = load_schedule_from_csv('data/schedule_ap_iter_1.csv')
        >>> df = result.dataframe

        >>> # From Streamlit uploaded file
        >>> csv_file = st.file_uploader("Upload CSV")
        >>> result = load_schedule_from_csv(csv_file)
    """
    logger.info("Loading schedule from CSV...")

    # Load CSV
    if isinstance(csv_path, (str, Path)):
        csv_path_obj = Path(csv_path)
        if not csv_path_obj.exists():
            raise FileNotFoundError(f"CSV file not found: {csv_path_obj}")
        df = pd.read_csv(csv_path_obj)
        path_str: str = str(csv_path_obj)
    elif hasattr(csv_path, "read"):
        df = pd.read_csv(csv_path)  # type: ignore[arg-type,call-overload]
        path_str = str(getattr(csv_path, "name", "uploaded_file"))
        if hasattr(csv_path, "seek"):
            csv_path.seek(0)  # type: ignore[union-attr]
    else:
        raise TypeError(f"Unsupported csv_path type: {type(csv_path)}")

    # Parse visibility column if it's a string representation of a list
    if "visibility" in df.columns:

        def parse_visibility(val: Any) -> list[Any]:
            if pd.isna(val):
                return []
            if isinstance(val, str):
                try:
                    import ast

                    parsed = ast.literal_eval(val)
                    return parsed  # type: ignore[no-any-return]
                except (ValueError, SyntaxError):
                    return []
            return val if isinstance(val, list) else []

        df["visibility"] = df["visibility"].apply(parse_visibility)

    # Basic validation
    validation = ValidationResult(
        is_valid=True,
        errors=[],
        warnings=[],
        stats={
            "total_blocks": len(df),
            "scheduled_blocks": df["scheduled_flag"].sum() if "scheduled_flag" in df.columns else 0,
        },
    )

    if validate:
        # Check for required columns
        required_cols = [
            "schedulingBlockId",
            "priority",
            "requestedDurationSec",
            "scheduled_period.start",
            "scheduled_period.stop",
        ]
        missing_cols = [col for col in required_cols if col not in df.columns]
        if missing_cols:
            validation.errors.append(f"Missing required columns: {missing_cols}")
            validation.is_valid = False

    validation.stats["unscheduled_blocks"] = (
        validation.stats["total_blocks"] - validation.stats["scheduled_blocks"]
    )

    logger.info(f"Successfully loaded {len(df)} scheduling blocks from CSV")

    return ScheduleLoadResult(
        dataframe=df,
        validation=validation,
        source_type="csv",
        source_path=path_str,
    )


def load_schedule_from_iteration(
    iteration_dir: str | Path,
    validate: bool = True,
) -> ScheduleLoadResult:
    """
    Load scheduling data from a data directory structure (legacy support).

    Expected directory structure:
    - data_dir/schedule.json
    - data_dir/possible_periods.json (optional)

    Args:
        iteration_dir: Path to data directory
        validate: Whether to validate the DataFrame

    Returns:
        ScheduleLoadResult with DataFrame and validation info

    Examples:
        >>> result = load_schedule_from_iteration('data/')
        >>> df = result.dataframe
    """
    iteration_path = Path(iteration_dir)

    if not iteration_path.exists():
        raise FileNotFoundError(f"Data directory not found: {iteration_path}")

    # Locate schedule.json (try new structure first, then legacy)
    schedule_path = iteration_path / "schedule.json"
    if not schedule_path.exists():
        # Try legacy structure: iteration_dir/schedule/schedule.json
        schedule_path = iteration_path / "schedule" / "schedule.json"
        if not schedule_path.exists():
            raise FileNotFoundError(f"Schedule file not found: {schedule_path}")

    # Locate possible_periods.json (optional)
    visibility_path = iteration_path / "possible_periods.json"
    if not visibility_path.exists():
        # Try legacy structure
        visibility_path = iteration_path / "possible periods" / "possible_periods.json"
    vis_path = visibility_path if visibility_path.exists() else None

    logger.info(f"Loading data from {iteration_path.name}...")

    # Use JSON loader
    result = load_schedule_from_json(schedule_path, vis_path, validate=validate)
    result.source_type = "data_directory"
    result.source_path = str(iteration_path)

    return result
