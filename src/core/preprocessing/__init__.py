"""Data preprocessing module for converting raw scheduling data to app-ready CSVs.

Note: The core preprocessing logic has been migrated to the Rust backend (tsi_rust module).
This module now only re-exports data structure definitions for backward compatibility.
"""

from dataclasses import dataclass


@dataclass
class ValidationResult:
    """Result of data validation checks."""

    is_valid: bool
    errors: list[str]
    warnings: list[str]
    stats: dict


@dataclass
class PreprocessMetadata:
    """Summary information produced by the preprocessing helpers."""

    total_blocks: int
    scheduled_blocks: int
    unscheduled_blocks: int
    csv_path: "Path | None" = None  # type: ignore
    parquet_path: "Path | None" = None  # type: ignore
    validation: ValidationResult | None = None


@dataclass
class PreprocessResult:
    """Container returned by the high-level preprocessing helpers."""

    dataframe: "pd.DataFrame"  # type: ignore
    metadata: PreprocessMetadata


__all__ = [
    "PreprocessMetadata",
    "PreprocessResult",
    "ValidationResult",
]
