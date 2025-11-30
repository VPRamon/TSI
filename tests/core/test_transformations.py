"""Unit tests for :mod:`tsi.services.data.preparation`."""

from __future__ import annotations

import pandas as pd
import pytest

from tsi.services.data.preparation import prepare_dataframe, validate_schema

pytestmark = pytest.mark.unit


def test_prepare_dataframe__with_minimal_columns__returns_enriched_snapshot() -> None:
    """Preparation should enrich the dataframe and emit warnings list."""

    # Given: a minimal but valid raw dataframe
    df = pd.DataFrame(
        {
            "schedulingBlockId": [1],
            "priority": [5],
            "requestedDurationSec": [3600],
            "minObservationTimeInSec": [1800],
            "decInDeg": [10],
            "raInDeg": [20],
            "minAzimuthAngleInDeg": [0],
            "maxAzimuthAngleInDeg": [20],
            "minElevationAngleInDeg": [0],
            "maxElevationAngleInDeg": [90],
            "num_visibility_periods": [1],
            "total_visibility_hours": [2.0],
            "visibility": ["[(59000.0, 59000.1)]"],
            "priority_bin": ["Medium"],
            "scheduled_period.start": [None],
            "scheduled_period.stop": [None],
            "fixedStartTime": [None],
            "fixedStopTime": [None],
        }
    )

    # When: preparing the dataframe for analytics
    result = prepare_dataframe(df)

    # Then: additional columns and warnings should be available
    assert "visibility_periods_parsed" in result.dataframe.columns
    assert isinstance(result.warnings, list)


def test_validate_schema__with_missing_columns__reports_missing_keys() -> None:
    """Schema validator must report missing required columns."""

    # Given: a dataframe lacking required columns
    df = pd.DataFrame({"a": [1]})

    # When: running schema validation
    ok, errors = validate_schema(df, required_columns={"a", "b"})

    # Then: validation fails with an explicit missing-columns message
    assert not ok
    assert any("missing columns" in error.lower() for error in errors)
