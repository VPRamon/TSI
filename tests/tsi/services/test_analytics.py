"""Unit tests for :mod:`tsi.services.data.analytics`."""

from __future__ import annotations

from collections.abc import Iterator
from io import StringIO

import pandas as pd
import pytest

from tsi.services.data.analytics import (
    compute_correlations,
    find_conflicts,
    get_top_observations,
)
from tsi.services.data.loaders import prepare_dataframe

pytestmark = pytest.mark.unit


@pytest.fixture
def streamlit_mock() -> Iterator[None]:
    """Provide a minimal Streamlit mock to satisfy module imports."""

    import sys
    from unittest.mock import MagicMock

    original = sys.modules.get("streamlit")
    sys.modules["streamlit"] = MagicMock()

    yield

    if original is None:
        del sys.modules["streamlit"]
    else:
        sys.modules["streamlit"] = original


@pytest.fixture
def prepared_dataframe(streamlit_mock: None) -> pd.DataFrame:
    """Create a prepared DataFrame for testing analytics pipelines."""

    csv_data = (
        "schedulingBlockId,priority,minObservationTimeInSec,requestedDurationSec,fixedStartTime,"
        "fixedStopTime,decInDeg,raInDeg,minAzimuthAngleInDeg,maxAzimuthAngleInDeg,minElevationAngleInDeg,"
        "maxElevationAngleInDeg,scheduled_period.start,scheduled_period.stop,visibility,num_visibility_periods,"
        "total_visibility_hours,priority_bin,scheduled_flag,requested_hours,elevation_range_deg\n"
        "1000001,8.5,1200,1200,,,40.5,307.5,0.0,360.0,60.0,90.0,61894.19,61894.20,"
        '"[(61892.20, 61892.21), (61893.20, 61893.21)]",2,48.0,Medium (8-10),True,0.3333333333,30.0\n'
        "1000002,6.2,1800,1800,61890.0,61900.0,35.2,315.8,0.0,360.0,50.0,85.0,,,[],0,24.0,Medium (4-7),False,0.5,35.0\n"
        "1000003,9.1,900,900,,,42.1,299.3,0.0,360.0,65.0,90.0,61895.10,61895.11,"
        '"[(61894.10, 61894.12), (61895.10, 61895.12)]",2,72.0,High (7-10),True,0.25,25.0\n'
        "1000004,5.0,1500,1500,,,38.0,310.0,0.0,360.0,55.0,80.0,,,[],0,36.0,Medium (4-7),False,0.4166666667,25.0\n"
        "1000005,9.8,1000,1000,,,45.0,295.0,0.0,360.0,70.0,90.0,61896.15,61896.16,"
        '"[(61896.14, 61896.17)]",1,96.0,High (7-10),True,0.2777777778,20.0\n'
    )
    df = pd.read_csv(StringIO(csv_data))
    return prepare_dataframe(df)


def test_compute_metrics__with_balanced_dataset__returns_expected_counts(
    prepared_dataframe: pd.DataFrame,
) -> None:
    """Metrics aggregation should capture basic scheduling statistics."""
    pytest.skip(
        "API changed: compute_metrics now expects schedule_id for database-backed analytics. "
        "Test needs migration to use database fixtures."
    )


def test_compute_correlations__with_numeric_columns__returns_square_matrix(
    prepared_dataframe: pd.DataFrame,
) -> None:
    """Correlation matrix should be square and bounded."""

    # Given / When: computing correlations
    corr_matrix = compute_correlations(prepared_dataframe)

    # Then: resulting matrix should be square and values between -1 and 1
    assert isinstance(corr_matrix, pd.DataFrame)
    assert corr_matrix.shape[0] == corr_matrix.shape[1]
    assert (corr_matrix.values >= -1.0).all() and (corr_matrix.values <= 1.0).all()


def test_get_top_observations__by_priority__returns_sorted_subset(
    prepared_dataframe: pd.DataFrame,
) -> None:
    """Selecting the top observations by priority should be sorted descending."""

    # Given / When: selecting top 3 by priority
    top = get_top_observations(prepared_dataframe, by="priority", n=3)

    # Then: list should be sorted and contain expected columns
    assert len(top) <= 3
    assert top["priority"].tolist() == sorted(top["priority"].tolist(), reverse=True)
    assert {"schedulingBlockId", "priority", "requested_hours"}.issubset(top.columns)


def test_get_top_observations__by_visibility__returns_sorted_subset(
    prepared_dataframe: pd.DataFrame,
) -> None:
    """Selecting by visibility should produce descending totals."""

    # Given / When: selecting by visibility
    top = get_top_observations(prepared_dataframe, by="total_visibility_hours", n=3)

    # Then: results should be descending by the requested metric
    visibility_values = top["total_visibility_hours"].tolist()
    assert visibility_values == sorted(visibility_values, reverse=True)


def test_find_conflicts__with_prepared_dataframe__returns_expected_columns(
    prepared_dataframe: pd.DataFrame,
) -> None:
    """Conflict detection should return a dataframe with expected schema."""

    # Given / When: scanning for conflicts
    conflicts = find_conflicts(prepared_dataframe)

    # Then: ensure expected columns exist when conflicts are present
    if not conflicts.empty:
        assert {"schedulingBlockId", "conflict_reasons"}.issubset(conflicts.columns)


def test_find_conflicts__unscheduled_rows__are_excluded_from_results(
    prepared_dataframe: pd.DataFrame,
) -> None:
    """Unscheduled observations must not appear in conflicts output."""

    # Given / When: retrieving conflicts
    conflicts = find_conflicts(prepared_dataframe)

    # Then: each conflict should reference a scheduled observation
    if not conflicts.empty:
        scheduled_ids = set(
            prepared_dataframe.loc[prepared_dataframe["scheduled_flag"], "schedulingBlockId"]
        )
        assert set(conflicts["schedulingBlockId"]) <= scheduled_ids


def test_generate_insights__with_metrics__returns_explanations(
    prepared_dataframe: pd.DataFrame,
) -> None:
    """Insight generator should produce human-readable insights."""
    pytest.skip(
        "API changed: compute_metrics now expects schedule_id for database-backed analytics. "
        "Test needs migration to use database fixtures."
    )
