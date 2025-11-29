"""Shared Rust backend instance and thin helpers."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Literal, cast

import pandas as pd

from tsi_rust_api import TSIBackend

# Single shared backend for the app
BACKEND = TSIBackend(use_pandas=True)


def load_schedule_from_any(
    source: str | Path | Any, format: Literal["auto", "csv", "json"] = "auto"
) -> pd.DataFrame:
    """
    Load schedule data from a path or file-like object via the Rust backend.
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
            return cast(pd.DataFrame, BACKEND.load_schedule_from_string(content, format="json"))
        if format == "csv":
            import io

            return pd.read_csv(io.StringIO(content))
        raise ValueError(f"Unsupported format: {format}")

    return cast(pd.DataFrame, BACKEND.load_schedule(Path(source), format=format))


__all__ = ["BACKEND", "load_schedule_from_any"]
