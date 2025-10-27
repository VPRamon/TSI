"""Low-level data cleaning helpers."""

from __future__ import annotations

from collections.abc import Iterable
from typing import Literal

import pandas as pd


def remove_duplicates(
    df: pd.DataFrame,
    subset: Iterable[str],
    *,
    keep: Literal["first", "last"] = "first",
) -> pd.DataFrame:
    """Return a copy of *df* without duplicate rows on *subset* columns."""

    return df.drop_duplicates(subset=list(subset), keep=keep).reset_index(drop=True)


def remove_missing_coordinates(df: pd.DataFrame) -> pd.DataFrame:
    """Remove rows with NaN RA/Dec values."""

    return df.dropna(subset=["raInDeg", "decInDeg"]).reset_index(drop=True)


def impute_missing(
    series: pd.Series,
    *,
    strategy: Literal["mean", "median", "mode", "constant"],
    fill_value: float | None = None,
) -> pd.Series:
    """Fill NaNs in *series* using the requested strategy without mutating the input."""

    result = series.copy()

    if strategy == "mean":
        fill = result.mean()
    elif strategy == "median":
        fill = result.median()
    elif strategy == "mode":
        fill = result.mode().iloc[0] if not result.mode().empty else result.iloc[0]
    else:  # constant
        if fill_value is None:
            raise ValueError("fill_value is required when strategy='constant'")
        fill = fill_value

    return result.fillna(fill)


def validate_schema(
    df: pd.DataFrame,
    *,
    required_columns: set[str],
    expected_dtypes: dict[str, str] | None = None,
) -> tuple[bool, list[str]]:
    """Validate that *df* contains the requested columns and dtypes."""

    errors: list[str] = []
    missing = required_columns - set(df.columns)
    if missing:
        errors.append(f"Missing columns: {sorted(missing)}")

    if expected_dtypes:
        for column, expected in expected_dtypes.items():
            if column not in df.columns:
                continue
            actual = str(df[column].dtype)
            if actual != expected:
                errors.append(f"Column '{column}' has dtype {actual}, expected {expected}")

    return not errors, errors
