"""Shared Rust backend instance and thin helpers."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Literal, cast

import pandas as pd

import tsi_rust_api

# Single shared backend for the app
BACKEND = tsi_rust_api.TSIBackend(use_pandas=True)


def load_schedule_from_any(
    source: str | Path | Any, format: Literal["auto", "json"] = "auto"
) -> pd.DataFrame:
    """
    Load schedule data from a path or file-like object via the Rust backend.

    Note: CSV format is no longer supported - use JSON only.
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
                tsi_rust_api.load_schedule_from_string(content, format="json", use_pandas=True),
            )
        raise ValueError(f"Unsupported format: {format}")

    return cast(pd.DataFrame, tsi_rust_api.load_schedule_file(Path(source), format=format, use_pandas=True))


__all__ = ["BACKEND", "load_schedule_from_any"]
