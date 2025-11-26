"""Comprehensive unit tests for :mod:`core.transformations.preparation`."""

from __future__ import annotations

import pandas as pd
import pytest

from core.transformations.preparation import (
    PreparationResult,
    filter_dataframe,
    parse_visibility_for_rows,
    prepare_dataframe,
    validate_dataframe,
)

pytestmark = pytest.mark.unit


@pytest.fixture
def required_mjd_columns() -> dict:
    """Required MJD/datetime columns for prepare_dataframe."""
    return {
        "fixedStartTime": None,
        "fixedStopTime": None,
        "scheduled_period.start": None,
        "scheduled_period.stop": None,
    }


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


class TestValidateDataframe:
    """Test validate_dataframe function."""

    def test_with_valid_dataframe__returns_no_issues(self) -> None:
        """Return no issues for valid dataframe."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB001", "SB002"],
                "priority": [5.0, 6.0],
                "decInDeg": [45.0, -30.0],
                "raInDeg": [123.45, 234.56],
            }
        )
        is_valid, issues = validate_dataframe(df)
        assert is_valid
        assert issues == []

    def test_with_missing_scheduling_block_id__reports_issue(self) -> None:
        """Report missing schedulingBlockId."""
        df = pd.DataFrame({"schedulingBlockId": ["SB001", None, "SB003"]})
        is_valid, issues = validate_dataframe(df)
        assert not is_valid
        assert any("missing schedulingBlockId" in issue for issue in issues)

    def test_with_invalid_priority__reports_issue(self) -> None:
        """Report invalid priority values."""
        df = pd.DataFrame({"priority": [5.0, float("nan"), float("inf"), -float("inf")]})
        is_valid, issues = validate_dataframe(df)
        assert not is_valid
        assert any("invalid priority" in issue for issue in issues)

    def test_with_invalid_declination__reports_issue(self) -> None:
        """Report declination outside [-90, 90]."""
        df = pd.DataFrame({"decInDeg": [45.0, -91.0, 91.0]})
        is_valid, issues = validate_dataframe(df)
        assert not is_valid
        assert any("invalid declination" in issue for issue in issues)

    def test_with_invalid_right_ascension__reports_issue(self) -> None:
        """Report right ascension outside [0, 360)."""
        df = pd.DataFrame({"raInDeg": [123.45, -10.0, 361.0]})
        is_valid, issues = validate_dataframe(df)
        assert not is_valid
        assert any("invalid right ascension" in issue for issue in issues)

    def test_with_missing_columns__handles_gracefully(self) -> None:
        """Handle missing columns gracefully."""
        df = pd.DataFrame({"someColumn": [1, 2, 3]})
        is_valid, issues = validate_dataframe(df)
        # Should not crash, may or may not be valid
        assert isinstance(is_valid, bool)
        assert isinstance(issues, list)

    def test_with_empty_dataframe__returns_valid(self) -> None:
        """Empty dataframe is considered valid."""
        df = pd.DataFrame()
        is_valid, issues = validate_dataframe(df)
        assert is_valid
        assert issues == []

    def test_with_boundary_coordinates__accepts_valid_boundaries(self) -> None:
        """Accept boundary values for coordinates."""
        df = pd.DataFrame({"decInDeg": [-90.0, 90.0], "raInDeg": [0.0, 359.999]})
        is_valid, issues = validate_dataframe(df)
        assert is_valid


class TestFilterDataframe:
    """Test filter_dataframe function."""

    def test_with_priority_range__filters_correctly(self) -> None:
        """Filter by priority range."""
        df = pd.DataFrame({"priority": [1.0, 5.0, 8.0, 10.0]})
        filtered = filter_dataframe(df, priority_range=(4.0, 9.0))
        assert len(filtered) == 2
        assert filtered["priority"].min() >= 4.0
        assert filtered["priority"].max() <= 9.0

    def test_with_scheduled_filter_all__returns_all_rows(self) -> None:
        """Return all rows when scheduled_filter is 'All'."""
        df = pd.DataFrame({"priority": [5.0, 6.0, 7.0], "scheduled_flag": [True, False, True]})
        filtered = filter_dataframe(df, scheduled_filter="All")
        assert len(filtered) == 3

    def test_with_scheduled_filter_scheduled__returns_only_scheduled(self) -> None:
        """Return only scheduled observations."""
        df = pd.DataFrame({"priority": [5.0, 6.0, 7.0], "scheduled_flag": [True, False, True]})
        filtered = filter_dataframe(df, scheduled_filter="Scheduled")
        assert len(filtered) == 2
        assert all(filtered["scheduled_flag"])

    def test_with_scheduled_filter_unscheduled__returns_only_unscheduled(self) -> None:
        """Return only unscheduled observations."""
        df = pd.DataFrame({"priority": [5.0, 6.0, 7.0], "scheduled_flag": [True, False, True]})
        filtered = filter_dataframe(df, scheduled_filter="Unscheduled")
        assert len(filtered) == 1
        assert not any(filtered["scheduled_flag"])

    def test_with_priority_bins__filters_by_bins(self) -> None:
        """Filter by priority bins."""
        df = pd.DataFrame(
            {
                "priority": [1.0, 5.0, 8.0, 10.0],
                "scheduled_flag": [False] * 4,
                "priority_bin": ["Low", "Medium", "High", "Critical"],
            }
        )
        filtered = filter_dataframe(df, priority_bins=["Low", "High"])
        assert len(filtered) == 2
        assert set(filtered["priority_bin"]) == {"Low", "High"}

    def test_with_block_ids__filters_by_ids(self) -> None:
        """Filter by block IDs."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB001", "SB002", "SB003"],
                "priority": [5.0, 6.0, 7.0],
                "scheduled_flag": [False] * 3,
            }
        )
        filtered = filter_dataframe(df, block_ids=["SB001", "SB003"])
        assert len(filtered) == 2
        assert set(filtered["schedulingBlockId"]) == {"SB001", "SB003"}

    def test_with_integer_block_ids__handles_gracefully(self) -> None:
        """Handle integer block IDs."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": [1, 2, 3],
                "priority": [5.0, 6.0, 7.0],
                "scheduled_flag": [False] * 3,
            }
        )
        filtered = filter_dataframe(df, block_ids=[1, 3])
        assert len(filtered) == 2

    def test_with_combined_filters__applies_all_filters(self) -> None:
        """Apply multiple filters simultaneously."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB001", "SB002", "SB003", "SB004"],
                "priority": [3.0, 5.0, 7.0, 9.0],
                "scheduled_flag": [True, False, True, False],
                "priority_bin": ["Low", "Medium", "High", "High"],
            }
        )
        filtered = filter_dataframe(
            df,
            priority_range=(4.0, 10.0),
            scheduled_filter="Unscheduled",
            priority_bins=["High"],
        )
        # Should return SB004: priority=9.0, unscheduled, High bin
        assert len(filtered) == 1
        assert filtered.iloc[0]["schedulingBlockId"] == "SB004"

    def test_with_empty_dataframe__returns_empty(self) -> None:
        """Handle empty dataframe."""
        df = pd.DataFrame({"priority": [], "scheduled_flag": []})
        filtered = filter_dataframe(df)
        assert len(filtered) == 0

    def test_with_no_matches__returns_empty(self) -> None:
        """Return empty when no rows match filters."""
        df = pd.DataFrame({"priority": [1.0, 2.0, 3.0], "scheduled_flag": [False] * 3})
        filtered = filter_dataframe(df, priority_range=(8.0, 10.0))
        assert len(filtered) == 0


class TestPreparationResult:
    """Test PreparationResult dataclass."""

    def test_is_frozen(self) -> None:
        """Ensure PreparationResult is immutable."""
        df = pd.DataFrame({"col": [1, 2, 3]})
        result = PreparationResult(dataframe=df, warnings=[])
        with pytest.raises(Exception):  # FrozenInstanceError
            result.dataframe = pd.DataFrame()  # type: ignore[misc]
