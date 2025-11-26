"""Comprehensive unit tests for :mod:`core.algorithms.analysis`."""

from __future__ import annotations

import pandas as pd
import pytest

pytest.importorskip("tsi_rust")

from core.algorithms.analysis import (
    AnalyticsSnapshot,
    _build_conflicts,
    _get_duration_timedelta,
    compute_correlations,
    compute_distribution_stats,
    compute_metrics,
    find_conflicts,
    generate_insights,
    get_top_observations,
    suggest_candidate_positions,
)

pytestmark = pytest.mark.unit


class TestGetDurationTimedelta:
    """Test _get_duration_timedelta helper function."""

    def test_with_valid_seconds__returns_timedelta(self) -> None:
        """Extract duration from requestedDurationSec field."""
        row = pd.Series({"requestedDurationSec": 3600.0})
        result = _get_duration_timedelta(row)
        assert result == pd.Timedelta(hours=1)

    def test_with_valid_hours__returns_timedelta(self) -> None:
        """Extract duration from requested_hours field."""
        row = pd.Series({"requested_hours": 2.5})
        result = _get_duration_timedelta(row)
        assert result == pd.Timedelta(hours=2.5)

    def test_with_missing_duration__returns_none(self) -> None:
        """Return None when no duration fields present."""
        row = pd.Series({"priority": 5})
        result = _get_duration_timedelta(row)
        assert result is None

    def test_with_nan_duration__returns_none(self) -> None:
        """Return None when duration is NaN."""
        row = pd.Series({"requestedDurationSec": float("nan")})
        result = _get_duration_timedelta(row)
        assert result is None

    def test_with_negative_duration__returns_timedelta(self) -> None:
        """Handle negative duration (edge case)."""
        row = pd.Series({"requestedDurationSec": -1800.0})
        result = _get_duration_timedelta(row)
        assert result == pd.Timedelta(seconds=-1800)

    def test_with_zero_duration__returns_zero_timedelta(self) -> None:
        """Handle zero duration."""
        row = pd.Series({"requestedDurationSec": 0.0})
        result = _get_duration_timedelta(row)
        assert result == pd.Timedelta(0)

    def test_with_invalid_type__returns_none(self) -> None:
        """Return None when duration cannot be converted to float."""
        row = pd.Series({"requestedDurationSec": "invalid"})
        result = _get_duration_timedelta(row)
        assert result is None

    def test_seconds_takes_precedence_over_hours(self) -> None:
        """requestedDurationSec is preferred over requested_hours."""
        row = pd.Series({"requestedDurationSec": 7200.0, "requested_hours": 1.0})
        result = _get_duration_timedelta(row)
        assert result == pd.Timedelta(hours=2)


class TestBuildConflicts:
    """Test _build_conflicts helper function."""

    def test_with_no_conflicts__returns_empty_list(self) -> None:
        """No conflicts when placement is valid."""
        row = pd.Series({})
        window_start = pd.Timestamp("2024-01-01 00:00:00", tz="UTC")
        window_stop = pd.Timestamp("2024-01-01 12:00:00", tz="UTC")
        candidate_start = pd.Timestamp("2024-01-01 01:00:00", tz="UTC")
        candidate_end = pd.Timestamp("2024-01-01 03:00:00", tz="UTC")
        scheduled_df = pd.DataFrame()

        conflicts = _build_conflicts(
            row,
            candidate_start,
            candidate_end,
            window_start,
            window_stop,
            scheduled_df,
        )

        assert conflicts == []

    def test_start_before_window__adds_conflict(self) -> None:
        """Detect start before visibility window."""
        row = pd.Series({})
        window_start = pd.Timestamp("2024-01-01 02:00:00", tz="UTC")
        window_stop = pd.Timestamp("2024-01-01 12:00:00", tz="UTC")
        candidate_start = pd.Timestamp("2024-01-01 01:00:00", tz="UTC")
        candidate_end = pd.Timestamp("2024-01-01 03:00:00", tz="UTC")
        scheduled_df = pd.DataFrame()

        conflicts = _build_conflicts(
            row,
            candidate_start,
            candidate_end,
            window_start,
            window_stop,
            scheduled_df,
        )

        assert "Start before visibility window" in conflicts

    def test_end_after_window__adds_conflict(self) -> None:
        """Detect end after visibility window."""
        row = pd.Series({})
        window_start = pd.Timestamp("2024-01-01 00:00:00", tz="UTC")
        window_stop = pd.Timestamp("2024-01-01 10:00:00", tz="UTC")
        candidate_start = pd.Timestamp("2024-01-01 08:00:00", tz="UTC")
        candidate_end = pd.Timestamp("2024-01-01 12:00:00", tz="UTC")
        scheduled_df = pd.DataFrame()

        conflicts = _build_conflicts(
            row,
            candidate_start,
            candidate_end,
            window_start,
            window_stop,
            scheduled_df,
        )

        assert any("End outside visibility window" in c for c in conflicts)

    def test_violates_fixed_start__adds_conflict(self) -> None:
        """Detect violation of fixed start constraint."""
        fixed_start = pd.Timestamp("2024-01-01 03:00:00", tz="UTC")
        row = pd.Series({"fixed_start_dt": fixed_start})
        window_start = pd.Timestamp("2024-01-01 00:00:00", tz="UTC")
        window_stop = pd.Timestamp("2024-01-01 12:00:00", tz="UTC")
        candidate_start = pd.Timestamp("2024-01-01 01:00:00", tz="UTC")
        candidate_end = pd.Timestamp("2024-01-01 04:00:00", tz="UTC")
        scheduled_df = pd.DataFrame()

        conflicts = _build_conflicts(
            row,
            candidate_start,
            candidate_end,
            window_start,
            window_stop,
            scheduled_df,
        )

        assert any("Violates fixed start" in c for c in conflicts)

    def test_violates_fixed_stop__adds_conflict(self) -> None:
        """Detect violation of fixed stop constraint."""
        fixed_stop = pd.Timestamp("2024-01-01 02:00:00", tz="UTC")
        row = pd.Series({"fixed_stop_dt": fixed_stop})
        window_start = pd.Timestamp("2024-01-01 00:00:00", tz="UTC")
        window_stop = pd.Timestamp("2024-01-01 12:00:00", tz="UTC")
        candidate_start = pd.Timestamp("2024-01-01 01:00:00", tz="UTC")
        candidate_end = pd.Timestamp("2024-01-01 03:00:00", tz="UTC")
        scheduled_df = pd.DataFrame()

        conflicts = _build_conflicts(
            row,
            candidate_start,
            candidate_end,
            window_start,
            window_stop,
            scheduled_df,
        )

        assert any("Violates fixed end" in c for c in conflicts)

    def test_overlapping_scheduled_observation__adds_conflict(self) -> None:
        """Detect overlap with scheduled observation."""
        row = pd.Series({})
        window_start = pd.Timestamp("2024-01-01 00:00:00", tz="UTC")
        window_stop = pd.Timestamp("2024-01-01 12:00:00", tz="UTC")
        candidate_start = pd.Timestamp("2024-01-01 02:00:00", tz="UTC")
        candidate_end = pd.Timestamp("2024-01-01 05:00:00", tz="UTC")
        scheduled_df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB123"],
                "scheduled_start_dt": [pd.Timestamp("2024-01-01 03:00:00", tz="UTC")],
                "scheduled_stop_dt": [pd.Timestamp("2024-01-01 06:00:00", tz="UTC")],
            }
        )

        conflicts = _build_conflicts(
            row,
            candidate_start,
            candidate_end,
            window_start,
            window_stop,
            scheduled_df,
        )

        assert any("Solapa con bloque" in c for c in conflicts)
        assert any("SB123" in c for c in conflicts)

    def test_multiple_overlaps__lists_up_to_three(self) -> None:
        """List first 3 overlapping observations."""
        row = pd.Series({})
        window_start = pd.Timestamp("2024-01-01 00:00:00", tz="UTC")
        window_stop = pd.Timestamp("2024-01-01 12:00:00", tz="UTC")
        candidate_start = pd.Timestamp("2024-01-01 01:00:00", tz="UTC")
        candidate_end = pd.Timestamp("2024-01-01 10:00:00", tz="UTC")
        scheduled_df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB1", "SB2", "SB3", "SB4"],
                "scheduled_start_dt": [
                    pd.Timestamp("2024-01-01 02:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 04:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 06:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 08:00:00", tz="UTC"),
                ],
                "scheduled_stop_dt": [
                    pd.Timestamp("2024-01-01 03:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 05:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 07:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 09:00:00", tz="UTC"),
                ],
            }
        )

        conflicts = _build_conflicts(
            row,
            candidate_start,
            candidate_end,
            window_start,
            window_stop,
            scheduled_df,
        )

        # Should list 3 conflicts plus "… y 1 conflictos adicionales"
        conflict_str = " ".join(conflicts)
        assert conflict_str.count("Solapa con bloque") == 3
        assert "conflictos adicionales" in conflict_str


class TestSuggestCandidatePositions:
    """Test suggest_candidate_positions function."""

    def test_with_no_visibility_periods__returns_empty_list(self) -> None:
        """Return empty list when no visibility periods."""
        df = pd.DataFrame({"scheduled_flag": []})
        row = pd.Series({"visibility_periods_parsed": []})
        candidates = suggest_candidate_positions(df, row)
        assert candidates == []

    def test_with_no_duration__returns_empty_list(self) -> None:
        """Return empty list when duration cannot be determined."""
        df = pd.DataFrame({"scheduled_flag": []})
        row = pd.Series(
            {
                "visibility_periods_parsed": [
                    (
                        pd.Timestamp("2024-01-01 00:00:00", tz="UTC"),
                        pd.Timestamp("2024-01-01 12:00:00", tz="UTC"),
                    )
                ]
            }
        )
        candidates = suggest_candidate_positions(df, row)
        assert candidates == []

    def test_with_zero_duration__returns_empty_list(self) -> None:
        """Return empty list when duration is zero."""
        df = pd.DataFrame({"scheduled_flag": []})
        row = pd.Series(
            {
                "visibility_periods_parsed": [
                    (
                        pd.Timestamp("2024-01-01 00:00:00", tz="UTC"),
                        pd.Timestamp("2024-01-01 12:00:00", tz="UTC"),
                    )
                ],
                "requestedDurationSec": 0.0,
            }
        )
        candidates = suggest_candidate_positions(df, row)
        assert candidates == []

    def test_with_negative_duration__returns_empty_list(self) -> None:
        """Return empty list when duration is negative."""
        df = pd.DataFrame({"scheduled_flag": []})
        row = pd.Series(
            {
                "visibility_periods_parsed": [
                    (
                        pd.Timestamp("2024-01-01 00:00:00", tz="UTC"),
                        pd.Timestamp("2024-01-01 12:00:00", tz="UTC"),
                    )
                ],
                "requestedDurationSec": -1800.0,
            }
        )
        candidates = suggest_candidate_positions(df, row)
        assert candidates == []

    def test_with_single_visibility_window__returns_one_candidate(self) -> None:
        """Generate candidate at window start."""
        df = pd.DataFrame({"scheduled_flag": []})
        row = pd.Series(
            {
                "visibility_periods_parsed": [
                    (
                        pd.Timestamp("2024-01-01 00:00:00", tz="UTC"),
                        pd.Timestamp("2024-01-01 12:00:00", tz="UTC"),
                    )
                ],
                "requestedDurationSec": 3600.0,
            }
        )
        candidates = suggest_candidate_positions(df, row)
        assert len(candidates) == 1
        assert candidates[0].anchor == "Window start"
        assert candidates[0].candidate_start == pd.Timestamp("2024-01-01 00:00:00", tz="UTC")

    def test_with_scheduled_observations__generates_multiple_candidates(self) -> None:
        """Generate candidates after scheduled observations."""
        df = pd.DataFrame(
            {
                "scheduled_flag": [True, True],
                "scheduled_start_dt": [
                    pd.Timestamp("2024-01-01 02:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 05:00:00", tz="UTC"),
                ],
                "scheduled_stop_dt": [
                    pd.Timestamp("2024-01-01 04:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 07:00:00", tz="UTC"),
                ],
                "schedulingBlockId": ["SB1", "SB2"],
            }
        )
        row = pd.Series(
            {
                "visibility_periods_parsed": [
                    (
                        pd.Timestamp("2024-01-01 00:00:00", tz="UTC"),
                        pd.Timestamp("2024-01-01 12:00:00", tz="UTC"),
                    )
                ],
                "requestedDurationSec": 3600.0,
            }
        )
        candidates = suggest_candidate_positions(df, row)
        # Should have: window start, after SB1, after SB2
        assert len(candidates) >= 3
        anchors = [c.anchor for c in candidates]
        assert "Window start" in anchors
        assert any("After block SB1" in a for a in anchors)
        assert any("After block SB2" in a for a in anchors)

    def test_candidates_are_deterministically_ordered(self) -> None:
        """Candidates should be sorted by (window_start, candidate_start)."""
        df = pd.DataFrame(
            {
                "scheduled_flag": [True],
                "scheduled_start_dt": [pd.Timestamp("2024-01-01 05:00:00", tz="UTC")],
                "scheduled_stop_dt": [pd.Timestamp("2024-01-01 07:00:00", tz="UTC")],
                "schedulingBlockId": ["SB1"],
            }
        )
        row = pd.Series(
            {
                "visibility_periods_parsed": [
                    (
                        pd.Timestamp("2024-01-01 00:00:00", tz="UTC"),
                        pd.Timestamp("2024-01-01 12:00:00", tz="UTC"),
                    )
                ],
                "requestedDurationSec": 3600.0,
            }
        )
        candidates = suggest_candidate_positions(df, row)
        # Window start should come before "After block"
        assert candidates[0].anchor == "Window start"
        assert candidates[0].candidate_start < candidates[1].candidate_start

    def test_with_nan_visibility_window__skips_window(self) -> None:
        """Skip visibility windows with NaN timestamps."""
        df = pd.DataFrame({"scheduled_flag": []})
        row = pd.Series(
            {
                "visibility_periods_parsed": [
                    (pd.NaT, pd.Timestamp("2024-01-01 12:00:00", tz="UTC")),
                    (
                        pd.Timestamp("2024-01-02 00:00:00", tz="UTC"),
                        pd.Timestamp("2024-01-02 12:00:00", tz="UTC"),
                    ),
                ],
                "requestedDurationSec": 3600.0,
            }
        )
        candidates = suggest_candidate_positions(df, row)
        # Should only generate candidate for the valid window
        assert len(candidates) == 1
        assert candidates[0].window_start == pd.Timestamp("2024-01-02 00:00:00", tz="UTC")


class TestComputeDistributionStats:
    """Test compute_distribution_stats function."""

    def test_with_empty_series__returns_empty_dict(self) -> None:
        """Return empty dict for empty series."""
        series = pd.Series([], dtype=float)
        stats = compute_distribution_stats(series)
        assert stats == {}

    def test_with_all_nan__returns_empty_dict(self) -> None:
        """Return empty dict when all values are NaN."""
        series = pd.Series([float("nan"), float("nan")])
        stats = compute_distribution_stats(series)
        assert stats == {}

    def test_with_single_value__computes_stats(self) -> None:
        """Compute stats for single value."""
        series = pd.Series([5.0])
        stats = compute_distribution_stats(series)
        assert stats["mean"] == 5.0
        assert stats["median"] == 5.0
        assert stats["min"] == 5.0
        assert stats["max"] == 5.0
        assert stats["count"] == 1

    def test_with_normal_distribution__computes_all_stats(self) -> None:
        """Compute all statistics for normal data."""
        series = pd.Series([1.0, 2.0, 3.0, 4.0, 5.0])
        stats = compute_distribution_stats(series)
        assert stats["mean"] == 3.0
        assert stats["median"] == 3.0
        assert stats["min"] == 1.0
        assert stats["max"] == 5.0
        assert stats["q25"] == 2.0
        assert stats["q75"] == 4.0
        assert stats["count"] == 5
        assert "std" in stats

    def test_with_mixed_nan__ignores_nan(self) -> None:
        """Ignore NaN values in calculations."""
        series = pd.Series([1.0, float("nan"), 3.0, float("nan"), 5.0])
        stats = compute_distribution_stats(series)
        assert stats["count"] == 3
        assert stats["mean"] == 3.0


class TestComputeCorrelations:
    """Test compute_correlations function."""

    def test_with_less_than_two_columns__returns_empty_dataframe(self) -> None:
        """Return empty dataframe when fewer than 2 columns."""
        df = pd.DataFrame({"a": [1, 2, 3]})
        corr_df = compute_correlations(df, columns=["a"])
        assert corr_df.empty

    def test_with_missing_columns__uses_available_columns(self) -> None:
        """Use only columns that exist in dataframe."""
        df = pd.DataFrame({"a": [1, 2, 3], "b": [4, 5, 6]})
        corr_df = compute_correlations(df, columns=["a", "b", "nonexistent"])
        assert not corr_df.empty
        assert "a" in corr_df.columns
        assert "b" in corr_df.columns
        assert "nonexistent" not in corr_df.columns

    def test_with_valid_columns__computes_spearman_correlation(self) -> None:
        """Compute Spearman correlation matrix."""
        df = pd.DataFrame({"a": [1, 2, 3, 4, 5], "b": [2, 4, 6, 8, 10]})
        corr_df = compute_correlations(df, columns=["a", "b"])
        assert corr_df.loc["a", "b"] == pytest.approx(1.0)

    def test_with_nan_values__drops_nan_rows(self) -> None:
        """Drop NaN rows before computing correlation."""
        df = pd.DataFrame({"a": [1, 2, float("nan"), 4, 5], "b": [2, 4, 6, 8, 10]})
        corr_df = compute_correlations(df, columns=["a", "b"])
        assert not corr_df.empty


class TestGenerateInsights:
    """Test generate_insights function."""

    def test_with_basic_metrics__generates_scheduling_rate_insight(self) -> None:
        """Always generate scheduling rate insight."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB001", "SB002"],
                "priority": [5.0, 6.0],
                "scheduled_flag": [True, False],
                "total_visibility_hours": [1.0, 2.0],
                "requested_hours": [1.0, 1.5],
                "scheduled_start_dt": [pd.Timestamp("2024-01-01", tz="UTC"), pd.NaT],
                "scheduled_stop_dt": [pd.Timestamp("2024-01-02", tz="UTC"), pd.NaT],
            }
        )
        metrics = AnalyticsSnapshot(
            total_observations=2,
            scheduled_count=1,
            unscheduled_count=1,
            scheduling_rate=0.5,
            mean_priority=5.5,
            median_priority=5.5,
            mean_priority_scheduled=5.0,
            mean_priority_unscheduled=6.0,
            total_visibility_hours=3.0,
            mean_requested_hours=1.25,
        )
        insights = generate_insights(df, metrics)
        assert any("Scheduling Rate" in insight for insight in insights)

    def test_with_priority_bias__generates_priority_insight(self) -> None:
        """Generate priority bias insight when diff > 0.5."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB001", "SB002"],
                "priority": [8.0, 3.0],
                "scheduled_flag": [True, False],
                "total_visibility_hours": [1.0, 2.0],
                "requested_hours": [1.0, 1.5],
                "scheduled_start_dt": [pd.Timestamp("2024-01-01", tz="UTC"), pd.NaT],
                "scheduled_stop_dt": [pd.Timestamp("2024-01-02", tz="UTC"), pd.NaT],
            }
        )
        metrics = AnalyticsSnapshot(
            total_observations=2,
            scheduled_count=1,
            unscheduled_count=1,
            scheduling_rate=0.5,
            mean_priority=5.5,
            median_priority=5.5,
            mean_priority_scheduled=8.0,
            mean_priority_unscheduled=3.0,
            total_visibility_hours=3.0,
            mean_requested_hours=1.25,
        )
        insights = generate_insights(df, metrics)
        assert any("Priority Bias" in insight for insight in insights)

    def test_with_visibility_hours__generates_visibility_insight(self) -> None:
        """Generate visibility insight when total hours > 0."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB001", "SB002"],
                "priority": [5.0, 6.0],
                "scheduled_flag": [True, False],
                "total_visibility_hours": [100.0, 200.0],
                "requested_hours": [1.0, 1.5],
                "scheduled_start_dt": [pd.Timestamp("2024-01-01", tz="UTC"), pd.NaT],
                "scheduled_stop_dt": [pd.Timestamp("2024-01-02", tz="UTC"), pd.NaT],
            }
        )
        metrics = AnalyticsSnapshot(
            total_observations=2,
            scheduled_count=1,
            unscheduled_count=1,
            scheduling_rate=0.5,
            mean_priority=5.5,
            median_priority=5.5,
            mean_priority_scheduled=5.0,
            mean_priority_unscheduled=6.0,
            total_visibility_hours=300.0,
            mean_requested_hours=1.25,
        )
        insights = generate_insights(df, metrics)
        assert any("Total Visibility" in insight for insight in insights)

    def test_with_strong_correlation__generates_correlation_insight(self) -> None:
        """Generate correlation insight when |correlation| > 0.3."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB001", "SB002", "SB003", "SB004", "SB005"],
                "priority": [1.0, 2.0, 3.0, 4.0, 5.0],
                "scheduled_flag": [True, False, True, False, True],
                "total_visibility_hours": [10, 20, 30, 40, 50],
                "requested_hours": [1.0, 2.0, 3.0, 4.0, 5.0],
                "elevation_range_deg": [5, 10, 15, 20, 25],
                "scheduled_start_dt": [
                    pd.Timestamp("2024-01-01", tz="UTC"),
                    pd.NaT,
                    pd.Timestamp("2024-01-03", tz="UTC"),
                    pd.NaT,
                    pd.Timestamp("2024-01-05", tz="UTC"),
                ],
                "scheduled_stop_dt": [
                    pd.Timestamp("2024-01-02", tz="UTC"),
                    pd.NaT,
                    pd.Timestamp("2024-01-04", tz="UTC"),
                    pd.NaT,
                    pd.Timestamp("2024-01-06", tz="UTC"),
                ],
            }
        )
        metrics = AnalyticsSnapshot(
            total_observations=5,
            scheduled_count=3,
            unscheduled_count=2,
            scheduling_rate=0.6,
            mean_priority=3.0,
            median_priority=3.0,
            mean_priority_scheduled=3.0,
            mean_priority_unscheduled=3.0,
            total_visibility_hours=150.0,
            mean_requested_hours=3.0,
        )
        insights = generate_insights(df, metrics)
        # Should have at least one correlation insight
        correlation_insights = [i for i in insights if "Correlation" in i]
        assert len(correlation_insights) > 0

    def test_with_empty_dataframe__generates_minimal_insights(self) -> None:
        """Handle empty dataframe gracefully."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": [],
                "priority": [],
                "scheduled_flag": [],
                "total_visibility_hours": [],
                "requested_hours": [],
            }
        )
        metrics = AnalyticsSnapshot(
            total_observations=0,
            scheduled_count=0,
            unscheduled_count=0,
            scheduling_rate=0.0,
            mean_priority=0.0,
            median_priority=0.0,
            mean_priority_scheduled=0.0,
            mean_priority_unscheduled=0.0,
            total_visibility_hours=0.0,
            mean_requested_hours=0.0,
        )
        insights = generate_insights(df, metrics)
        # Should at least have scheduling rate
        assert len(insights) >= 1


class TestComputeMetrics:
    """Test compute_metrics function (integration with Rust backend)."""

    def test_with_valid_dataframe__returns_snapshot(self) -> None:
        """Compute metrics via Rust backend."""
        df = pd.DataFrame(
            {
                "priority": [5.0, 6.0, 7.0],
                "scheduled_flag": [True, False, True],
                "total_visibility_hours": [1.0, 2.0, 3.0],
                "requested_hours": [1.0, 1.5, 2.0],
            }
        )
        snapshot = compute_metrics(df)
        assert isinstance(snapshot, AnalyticsSnapshot)
        assert snapshot.total_observations == 3
        assert snapshot.scheduled_count == 2


class TestGetTopObservations:
    """Test get_top_observations function."""

    def test_with_valid_dataframe__returns_top_n(self) -> None:
        """Return top N observations by specified column."""
        df = pd.DataFrame(
            {
                "priority": [1, 5, 3, 9, 2],
                "scheduled_flag": [False, True, False, True, False],
            }
        )
        top_obs = get_top_observations(df, by="priority", n=3)
        assert len(top_obs) <= 3


class TestFindConflicts:
    """Test find_conflicts function."""

    def test_with_valid_dataframe__returns_conflicts(self) -> None:
        """Find scheduling conflicts via Rust backend."""
        df = pd.DataFrame(
            {
                "schedulingBlockId": ["SB1", "SB2"],
                "scheduled_flag": [True, True],
                "scheduled_start_dt": [
                    pd.Timestamp("2024-01-01 00:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 01:00:00", tz="UTC"),
                ],
                "scheduled_stop_dt": [
                    pd.Timestamp("2024-01-01 02:00:00", tz="UTC"),
                    pd.Timestamp("2024-01-01 03:00:00", tz="UTC"),
                ],
            }
        )
        conflicts = find_conflicts(df)
        assert isinstance(conflicts, pd.DataFrame)
