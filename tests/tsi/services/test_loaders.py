"""Unit tests for :mod:`tsi.services.data.loaders`."""

from __future__ import annotations

from collections.abc import Iterator
from io import StringIO

import pandas as pd
import pytest

from tsi.exceptions import SchemaError
from tsi.services.data.loaders import (
    get_filtered_dataframe,
    prepare_dataframe,
    validate_dataframe,
)

# Required columns that were checked by the now-removed load_csv_with_validation function
REQUIRED_COLUMNS = {
    "schedulingBlockId",
    "priority",
    "minObservationTimeInSec",
    "requestedDurationSec",
    "decInDeg",
    "raInDeg",
}


def load_csv_with_validation(buffer: StringIO) -> pd.DataFrame:
    """Load CSV data and validate required columns.

    This is a test helper that mimics the behavior of the removed load_csv_with_validation function.
    """
    df = pd.read_csv(buffer)
    missing = REQUIRED_COLUMNS - set(df.columns)
    if missing:
        raise SchemaError(f"Missing required columns: {sorted(missing)}")
    return df


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
def sample_csv_data() -> str:
    """Create sample CSV data for testing."""

    return (
        "schedulingBlockId,priority,minObservationTimeInSec,requestedDurationSec,"
        "fixedStartTime,fixedStopTime,decInDeg,raInDeg,minAzimuthAngleInDeg,"
        "maxAzimuthAngleInDeg,minElevationAngleInDeg,maxElevationAngleInDeg,"
        "scheduled_period.start,scheduled_period.stop,visibility,num_visibility_periods,"
        "total_visibility_hours,priority_bin,scheduled_flag,requested_hours,"
        "elevation_range_deg\n"
        "1000001,8.5,1200,1200,,,40.5,307.5,0.0,360.0,60.0,90.0,61894.19,61894.20,"
        '"[(61892.20, 61892.21), (61893.20, 61893.21)]",2,48.0,Medium (8-10),True,0.3333333333,30.0\n'
        "1000002,6.2,1800,1800,61890.0,61900.0,35.2,315.8,0.0,360.0,50.0,85.0,,,[] ,0,0.0,Medium (4-7),False,0.5,35.0\n"
        "1000003,9.1,900,900,,,42.1,299.3,0.0,360.0,65.0,90.0,61895.10,61895.11,"
        '"[(61894.10, 61894.12), (61895.10, 61895.12)]",2,72.0,High (7-10),True,0.25,25.0\n'
    )


@pytest.fixture
def sample_dataframe(sample_csv_data: str) -> pd.DataFrame:
    """Load sample CSV into DataFrame."""

    buffer = StringIO(sample_csv_data)
    return load_csv_with_validation(buffer)


def test_load_csv_with_validation__with_valid_buffer__returns_dataframe(
    sample_csv_data: str,
) -> None:
    """Loading a valid CSV should produce a dataframe with expected columns."""

    # Given: an in-memory CSV buffer
    buffer = StringIO(sample_csv_data)

    # When: loading the CSV through the helper
    df = load_csv_with_validation(buffer)

    # Then: the dataframe should have the correct shape and columns
    assert isinstance(df, pd.DataFrame)
    assert len(df) == 3
    assert {"schedulingBlockId", "priority"}.issubset(df.columns)


def test_load_csv_with_validation__with_missing_columns__raises_value_error() -> None:
    """Missing required headers should trigger a helpful error."""

    # Given: a CSV lacking the required schema
    buffer = StringIO("col1,col2\n1,2\n")

    # When / Then: the loader should raise a validation error
    with pytest.raises(SchemaError, match="Missing required columns"):
        load_csv_with_validation(buffer)


def test_prepare_dataframe__adds_enriched_columns(
    streamlit_mock: None, sample_dataframe: pd.DataFrame
) -> None:
    """Prepare step should enrich the dataframe with derived features."""

    # Given: a parsed dataframe and mocked Streamlit environment

    # When: preparing the dataframe for analytics
    prepared = prepare_dataframe(sample_dataframe)

    # Then: enrichment columns should be present
    for column in {
        "scheduled_flag",
        "elevation_range_deg",
        "requested_hours",
        "visibility_periods_parsed",
    }:
        assert column in prepared.columns


def test_prepare_dataframe__computes_scheduled_flag(
    streamlit_mock: None, sample_dataframe: pd.DataFrame
) -> None:
    """Scheduled observations should be marked appropriately."""

    # Given: the prepared dataframe
    prepared = prepare_dataframe(sample_dataframe)

    # When: checking scheduled indicator columns
    # Then: first row is scheduled, second is not
    assert bool(prepared.loc[0, "scheduled_flag"]) is True
    assert bool(prepared.loc[1, "scheduled_flag"]) is False


def test_get_filtered_dataframe__with_priority_range__returns_subset(
    streamlit_mock: None, sample_dataframe: pd.DataFrame
) -> None:
    """Filtering by priority should restrict rows to the desired range."""

    # Given: a prepared dataframe
    prepared = prepare_dataframe(sample_dataframe)

    # When: applying a priority filter
    filtered = get_filtered_dataframe(prepared, priority_range=(8.0, 10.0))

    # Then: only high-priority rows remain
    assert len(filtered) == 2
    assert (filtered["priority"] >= 8.0).all() and (filtered["priority"] <= 10.0).all()


def test_get_filtered_dataframe__with_scheduled_filter__splits_data(
    streamlit_mock: None, sample_dataframe: pd.DataFrame
) -> None:
    """Filtering by scheduled status should separate scheduled/unscheduled rows."""

    # Given: a prepared dataframe
    prepared = prepare_dataframe(sample_dataframe)

    # When: filtering by scheduled and unscheduled status
    scheduled = get_filtered_dataframe(prepared, scheduled_filter="Scheduled")
    unscheduled = get_filtered_dataframe(prepared, scheduled_filter="Unscheduled")

    # Then: the flags should be homogeneous within each subset
    assert scheduled["scheduled_flag"].all()
    assert (~unscheduled["scheduled_flag"]).all()


def test_validate_dataframe__with_valid_input__returns_no_issues(
    sample_dataframe: pd.DataFrame,
) -> None:
    """Validation should succeed when the schema is correct."""

    # Given: a dataframe matching the expected schema

    # When: running the validator
    is_valid, issues = validate_dataframe(sample_dataframe)

    # Then: the dataframe should be accepted without issues
    assert is_valid is True
    assert issues == []


def test_validate_dataframe__with_invalid_priority__reports_issue() -> None:
    """Invalid priority values must be surfaced to the caller."""

    # Given: a CSV with a malformed priority column
    csv_data = (
        "schedulingBlockId,priority,minObservationTimeInSec,requestedDurationSec,fixedStartTime,"
        "fixedStopTime,decInDeg,raInDeg,minAzimuthAngleInDeg,maxAzimuthAngleInDeg,minElevationAngleInDeg,"
        "maxElevationAngleInDeg,scheduled_period.start,scheduled_period.stop,visibility,num_visibility_periods,"
        "total_visibility_hours,priority_bin,scheduled_flag,requested_hours,elevation_range_deg\n"
        '1000001,not-a-number,1200,1200,,,40.5,307.5,0.0,360.0,60.0,90.0,61894.19,61894.20,"[]",0,0.0,Medium,True,0.3333,30.0\n'
    )
    df = load_csv_with_validation(StringIO(csv_data))

    # When: validating the dataframe
    is_valid, issues = validate_dataframe(df)

    # Then: validation should fail with a priority message
    assert is_valid is False
    assert any("priority" in issue.lower() for issue in issues)


def test_validate_dataframe__with_invalid_declination__reports_issue() -> None:
    """Out-of-range declination angles should be caught."""

    # Given: a CSV with an invalid declination value
    csv_data = (
        "schedulingBlockId,priority,minObservationTimeInSec,requestedDurationSec,fixedStartTime,"
        "fixedStopTime,decInDeg,raInDeg,minAzimuthAngleInDeg,maxAzimuthAngleInDeg,minElevationAngleInDeg,"
        "maxElevationAngleInDeg,scheduled_period.start,scheduled_period.stop,visibility,num_visibility_periods,"
        "total_visibility_hours,priority_bin,scheduled_flag,requested_hours,elevation_range_deg\n"
        '1000001,8.5,1200,1200,,,95.0,307.5,0.0,360.0,60.0,90.0,61894.19,61894.20,"[]",0,0.0,Medium,True,0.3333,30.0\n'
    )
    df = load_csv_with_validation(StringIO(csv_data))

    # When: validating the dataframe
    is_valid, issues = validate_dataframe(df)

    # Then: an informative message about declination is returned
    assert is_valid is False
    assert any("declination" in issue.lower() for issue in issues)
