"""Priority range utilities now that schedule preprocessing happens in Rust."""

from __future__ import annotations

import pandas as pd


def get_priority_range(df: pd.DataFrame) -> tuple[float, float]:
    """
    Calculate the priority range from a DataFrame.

    Extracts min and max priority values, handling edge cases:
    - Missing priority column → returns (0.0, 10.0)
    - Empty priority values → returns (0.0, 10.0)
    - Single priority value → returns (value, value + 1.0)
    """

    if "priority" not in df.columns:
        return 0.0, 10.0

    priority_values = df["priority"].dropna()

    if priority_values.empty:
        return 0.0, 10.0

    priority_min = float(priority_values.min())
    priority_max = float(priority_values.max())

    if priority_min == priority_max:
        priority_max = priority_min + 1.0

    return priority_min, priority_max
