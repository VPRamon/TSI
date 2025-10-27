"""Tests for loading schedule data from JSON sources."""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest

from core.loaders import load_schedule_from_json

PROJECT_ROOT = Path(__file__).resolve().parents[2]


@pytest.fixture(scope="module")
def schedule_paths() -> tuple[Path, Path | None]:
    """Provide paths to the JSON schedule fixtures bundled with the repo."""

    schedule_path = PROJECT_ROOT / "data" / "schedule.json"
    visibility_path = PROJECT_ROOT / "data" / "possible_periods.json"

    if not schedule_path.exists():
        pytest.skip("Repository fixture data/schedule.json is not available.")

    return schedule_path, visibility_path if visibility_path.exists() else None


def test_load_schedule_from_json_provides_expected_schema(
    schedule_paths: tuple[Path, Path | None],
) -> None:
    """Ensure the JSON loader returns data with the expected columns and validation."""

    schedule_path, visibility_path = schedule_paths

    result = load_schedule_from_json(schedule_path, visibility_path)
    df = result.dataframe

    assert not df.empty, "Loader should return at least one scheduling block."
    assert result.source_type == "json"
    assert result.validation.is_valid, "Validation should succeed for bundled dataset."

    required_columns = {
        "schedulingBlockId",
        "visibility",
        "scheduled_flag",
        "requested_hours",
        "elevation_range_deg",
        "priority_bin",
        "num_visibility_periods",
        "total_visibility_hours",
    }

    missing_columns = required_columns.difference(df.columns)
    assert not missing_columns, f"Missing expected derived columns: {sorted(missing_columns)}"

    scheduled_rows = df[df["scheduled_flag"].fillna(False)]
    assert not scheduled_rows.empty, "Dataset should include scheduled blocks."
    assert scheduled_rows["scheduled_period.start"].notna().all()
    assert scheduled_rows["scheduled_period.stop"].notna().all()

    expected_requested_hours = df["requestedDurationSec"] / 3600.0
    assert np.allclose(
        df["requested_hours"], expected_requested_hours
    ), "Requested hours should match seconds conversion."

    priority_bins = df["priority_bin"].dropna().unique()
    assert len(priority_bins) > 0, "Priority bins should be populated."


def test_load_schedule_from_json_without_visibility(
    schedule_paths: tuple[Path, Path | None],
) -> None:
    """Verify that loading without a visibility file still returns consistent results."""

    schedule_path, visibility_path = schedule_paths

    with_visibility = load_schedule_from_json(schedule_path, visibility_path)
    without_visibility = load_schedule_from_json(schedule_path, None)

    df_with_visibility = with_visibility.dataframe
    df_without_visibility = without_visibility.dataframe

    assert len(df_without_visibility) == len(
        df_with_visibility
    ), "Row count should remain unchanged."

    if "num_visibility_periods" in df_without_visibility.columns:
        assert (
            df_without_visibility["num_visibility_periods"] == 0
        ).all(), "Visibility periods should default to zero."
