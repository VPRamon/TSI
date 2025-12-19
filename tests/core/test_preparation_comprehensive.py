"""Comprehensive unit tests for :mod:`tsi.services.data.preparation`."""

from __future__ import annotations

import pandas as pd
import pytest

from tsi.services.data.preparation import (
    PreparationResult,
    parse_visibility_for_rows,
    prepare_dataframe,
)

pytestmark = pytest.mark.unit


class TestParseVisibilityForRows:
    """Test parse_visibility_for_rows function."""

    def test_with_missing_visibility_column__returns_none_series(self) -> None:
        """Return None series when visibility column missing."""
        df = pd.DataFrame({"schedulingBlockId": ["SB001", "SB002"]})
        result = parse_visibility_for_rows(df)
        assert len(result) == 2
        assert all(pd.isna(result))

    def test_with_nan_values__returns_none(self) -> None:
        """Return None for NaN visibility values."""
        df = pd.DataFrame({"visibility": [float("nan"), None]})
        result = parse_visibility_for_rows(df)
        assert all(pd.isna(result))

    def test_with_valid_visibility__parses_periods(self) -> None:
        """Parse visibility periods from valid data."""
        df = pd.DataFrame({"visibility": ["[[58000.0, 58001.0]]"]})
        result = parse_visibility_for_rows(df)
        # Should return parsed periods (specific format depends on parse_visibility_periods)
        assert len(result) == 1

    def test_with_custom_column_name__uses_custom_column(self) -> None:
        """Use custom visibility column name."""
        df = pd.DataFrame({"custom_vis": ["[[58000.0, 58001.0]]"]})
        result = parse_visibility_for_rows(df, visibility_column="custom_vis")
        assert len(result) == 1


class TestPrepareDataframe:
    """Test prepare_dataframe function."""

    def test_with_minimal_dataframe__returns_preparation_result(self) -> None:
        """Prepare minimal dataframe."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB001"],
                "priority": [5.0],
                "fixedStartTime": [None],
                "fixedStopTime": [None],
                "scheduled_period.start": [None],
                "scheduled_period.stop": [None],
            }
        )
        result = prepare_dataframe(df)
        assert isinstance(result, PreparationResult)
        assert isinstance(result.dataframe, pd.DataFrame)
        assert isinstance(result.warnings, list)

    def test_with_numeric_columns__coerces_to_numeric(self) -> None:
        """Coerce numeric columns to proper types."""
        df = pd.DataFrame(
            {
                "priority": ["5.0", "6.0"],
                "requestedDurationSec": ["3600", "7200"],
                "raInDeg": ["123.45", "234.56"],
                "fixedStartTime": [None, None],
                "fixedStopTime": [None, None],
                "scheduled_period.start": [None, None],
                "scheduled_period.stop": [None, None],
            }
        )
        result = prepare_dataframe(df)
        assert pd.api.types.is_numeric_dtype(result.dataframe["priority"])
        assert pd.api.types.is_numeric_dtype(result.dataframe["requestedDurationSec"])

    def test_with_invalid_numeric_values__coerces_to_nan(self) -> None:
        """Convert invalid numeric values to NaN."""
        df = pd.DataFrame(
            {
                "priority": ["invalid", "5.0", "6.0"],
                "fixedStartTime": [None, None, None],
                "fixedStopTime": [None, None, None],
                "scheduled_period.start": [None, None, None],
                "scheduled_period.stop": [None, None, None],
            }
        )
        result = prepare_dataframe(df)
        assert pd.isna(result.dataframe["priority"].iloc[0])
        assert result.dataframe["priority"].iloc[1] == 5.0

    def test_with_string_scheduled_flag__converts_to_boolean(self) -> None:
        """Convert string scheduled_flag to boolean."""
        df = pd.DataFrame({"scheduled_flag": ["True", "False", "true", "false"]})
        result = prepare_dataframe(df)
        assert result.dataframe["scheduled_flag"].dtype == bool
        assert result.dataframe["scheduled_flag"].iloc[0]
        assert not result.dataframe["scheduled_flag"].iloc[1]
        assert result.dataframe["scheduled_flag"].iloc[2]
        assert not result.dataframe["scheduled_flag"].iloc[3]

    def test_with_nan_scheduled_flag__converts_to_false(self) -> None:
        """Convert NaN scheduled_flag to False."""
        df = pd.DataFrame({"scheduled_flag": [float("nan"), None, "True"]})
        result = prepare_dataframe(df)
        assert not result.dataframe["scheduled_flag"].iloc[0]
        assert not result.dataframe["scheduled_flag"].iloc[1]
        assert result.dataframe["scheduled_flag"].iloc[2]

    def test_with_mjd_columns__converts_to_numeric(self) -> None:
        """Convert MJD columns to numeric."""
        df = pd.DataFrame(
            {
                "fixedStartTime": ["58000.5", "58100.5"],
                "fixedStopTime": ["58001.5", "58101.5"],
                "scheduled_period.start": ["58010.5", "58110.5"],
                "scheduled_period.stop": ["58011.5", "58111.5"],
            }
        )
        result = prepare_dataframe(df)
        assert result.dataframe["fixedStartTime"].dtype in [float, "float64"]
        assert result.dataframe["scheduled_period.start"].dtype in [float, "float64"]

    def test_with_visibility_column__sets_parsed_to_none(self) -> None:
        """Set visibility_periods_parsed to None for lazy parsing."""
        df = pd.DataFrame({"visibility": ["[[58000.0, 58001.0]]"]})
        result = prepare_dataframe(df)
        assert "visibility_periods_parsed" in result.dataframe.columns
        # Should be None for lazy parsing
        assert result.dataframe["visibility_periods_parsed"].iloc[0] is None

    def test_with_empty_dataframe__returns_empty_result(self) -> None:
        """Handle empty dataframe."""
        df = pd.DataFrame()
        result = prepare_dataframe(df)
        assert len(result.dataframe) == 0
        assert isinstance(result.warnings, list)

    def test_does_not_modify_original_dataframe(self) -> None:
        """Ensure original dataframe is not modified."""
        df = pd.DataFrame({"priority": ["5.0", "6.0"]})
        original_priority = df["priority"].copy()
        prepare_dataframe(df)
        pd.testing.assert_series_equal(df["priority"], original_priority)


class TestPreparationResult:
    """Test PreparationResult dataclass."""

    def test_is_frozen(self) -> None:
        """Ensure PreparationResult is immutable."""
        df = pd.DataFrame({"col": [1, 2, 3]})
        result = PreparationResult(dataframe=df, warnings=[])
        with pytest.raises(Exception):  # FrozenInstanceError
            result.dataframe = pd.DataFrame()  # type: ignore[misc]
