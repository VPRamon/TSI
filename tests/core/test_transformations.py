"""Unit tests for :mod:`core.transformations`."""

from __future__ import annotations

import pandas as pd
import pytest

from core.transformations import prepare_dataframe, validate_dataframe
from core.transformations.data_cleaning import impute_missing, remove_duplicates, validate_schema

pytestmark = pytest.mark.unit


def test_remove_duplicates__with_duplicate_keys__keeps_first_occurrence() -> None:
    """Duplicate rows should collapse to the first entry for each key."""

    # Given: a dataframe with a repeated identifier
    df = pd.DataFrame({"id": [1, 2, 2], "value": ["a", "b", "c"]})

    # When: removing duplicates by key
    result = remove_duplicates(df, subset=["id"])

    # Then: two rows remain and the first duplicate is preserved
    assert len(result) == 2
    assert result.loc[result["id"] == 2, "value"].iloc[0] == "b"


def test_impute_missing__with_constant_strategy_without_value__raises_error() -> None:
    """Constant strategy requires an explicit fill value."""

    # Given: a series containing missing values
    series = pd.Series([1.0, None, 3.0])

    # When / Then: attempting constant imputation without ``fill_value`` fails
    with pytest.raises(ValueError):
        impute_missing(series, strategy="constant")


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


def test_validate_dataframe__with_invalid_ra__returns_error_messages() -> None:
    """Validation should reject out-of-range right ascension values."""

    # Given: a dataframe containing an impossible RA value
    df = pd.DataFrame(
        {
            "schedulingBlockId": [1],
            "priority": [5],
            "raInDeg": [720],
            "decInDeg": [0],
        }
    )

    # When: validating the dataframe
    is_valid, issues = validate_dataframe(df)

    # Then: the validator should flag the right ascension column
    assert not is_valid
    assert any("right ascension" in issue.lower() for issue in issues)


def test_validate_schema__with_missing_columns__reports_missing_keys() -> None:
    """Schema validator must report missing required columns."""

    # Given: a dataframe lacking required columns
    df = pd.DataFrame({"a": [1]})

    # When: running schema validation
    ok, errors = validate_schema(df, required_columns={"a", "b"})

    # Then: validation fails with an explicit missing-columns message
    assert not ok
    assert any("missing columns" in error.lower() for error in errors)
