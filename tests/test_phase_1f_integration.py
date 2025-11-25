"""Integration tests for FASE 1F: Python Integration Layer"""

import pandas as pd
import polars as pl
import pytest

# Test high-level API wrapper
from tsi_rust_api import TSIBackend, compute_metrics


@pytest.fixture
def sample_df():
    """Create a sample DataFrame for testing"""
    return pl.DataFrame(
        {
            "schedulingBlockId": ["SB001", "SB002", "SB003", "SB004", "SB005"],
            "priority": [5.0, 10.0, 15.0, 20.0, 25.0],
            "scheduled_flag": [True, True, False, True, False],
            "raInDeg": [120.0, 150.0, 180.0, 210.0, 240.0],
            "decInDeg": [45.0, 30.0, -15.0, -30.0, -45.0],
            "requestedDurationSec": [3600.0, 7200.0, 5400.0, 1800.0, 9000.0],
            "requested_hours": [1.0, 2.0, 1.5, 0.5, 2.5],
            "priority_bin": ["Low", "Medium", "High", "High", "Very High"],
            "total_visibility_hours": [10.0, 15.0, 12.0, 8.0, 20.0],
            "num_visibility_periods": [2, 3, 2, 1, 4],
            "elevation_range_deg": [30.0, 40.0, 35.0, 25.0, 45.0],
        }
    )


def test_backend_initialization():
    """Test backend can be initialized"""
    backend = TSIBackend(use_pandas=True)
    assert backend.use_pandas is True

    backend_polars = TSIBackend(use_pandas=False)
    assert backend_polars.use_pandas is False
    print("✓ Backend initialization: OK")


def test_compute_metrics_wrapper(sample_df):
    """Test compute_metrics with wrapper"""
    backend = TSIBackend()
    df_pandas = sample_df.to_pandas()

    metrics = backend.compute_metrics(df_pandas)

    assert isinstance(metrics, dict)
    assert "total_observations" in metrics
    assert "scheduled_count" in metrics
    assert "mean_priority" in metrics
    assert metrics["total_observations"] == 5
    assert metrics["scheduled_count"] == 3
    print(
        f"✓ Compute metrics wrapper: {metrics['scheduled_count']}/{metrics['total_observations']} scheduled"
    )


def test_filter_by_priority_wrapper(sample_df):
    """Test filter_by_priority with wrapper"""
    backend = TSIBackend(use_pandas=True)
    df_pandas = sample_df.to_pandas()

    filtered = backend.filter_by_priority(df_pandas, min_priority=10.0, max_priority=20.0)

    assert isinstance(filtered, pd.DataFrame)
    assert len(filtered) == 3  # 10, 15, 20
    assert all(filtered["priority"] >= 10.0)
    assert all(filtered["priority"] <= 20.0)
    print(f"✓ Filter by priority wrapper: {len(sample_df)} -> {len(filtered)} rows")


def test_filter_by_scheduled_wrapper(sample_df):
    """Test filter_by_scheduled with wrapper"""
    backend = TSIBackend(use_pandas=True)
    df_pandas = sample_df.to_pandas()

    scheduled = backend.filter_by_scheduled(df_pandas, "Scheduled")
    assert len(scheduled) == 3
    assert all(scheduled["scheduled_flag"])

    unscheduled = backend.filter_by_scheduled(df_pandas, "Unscheduled")
    assert len(unscheduled) == 2
    assert not any(unscheduled["scheduled_flag"])
    print(f"✓ Filter by scheduled: {len(scheduled)} scheduled, {len(unscheduled)} unscheduled")


def test_get_top_observations_wrapper(sample_df):
    """Test get_top_observations with wrapper"""
    backend = TSIBackend(use_pandas=True)
    df_pandas = sample_df.to_pandas()

    top_3 = backend.get_top_observations(df_pandas, n=3, by="priority")

    assert isinstance(top_3, pd.DataFrame)
    assert len(top_3) == 3
    # Should be descending order
    priorities = top_3["priority"].tolist()
    assert priorities == sorted(priorities, reverse=True)
    print(f"✓ Get top observations: Top 3 priorities = {priorities}")


def test_remove_duplicates_wrapper(sample_df):
    """Test remove_duplicates with wrapper"""
    backend = TSIBackend(use_pandas=True)

    # Add duplicate
    df_with_dup = pl.concat([sample_df, sample_df.head(1)])
    df_pandas = df_with_dup.to_pandas()

    unique_df = backend.remove_duplicates(df_pandas)

    assert isinstance(unique_df, pd.DataFrame)
    assert len(unique_df) == len(sample_df)  # Should remove 1 duplicate
    print(f"✓ Remove duplicates: {len(df_pandas)} -> {len(unique_df)} rows")


def test_remove_missing_coordinates_wrapper():
    """Test remove_missing_coordinates with wrapper"""
    backend = TSIBackend(use_pandas=True)

    df_with_nulls = pl.DataFrame(
        {
            "raInDeg": [120.0, None, 180.0],
            "decInDeg": [45.0, 30.0, None],
            "priority": [5.0, 10.0, 15.0],
        }
    )
    df_pandas = df_with_nulls.to_pandas()

    clean_df = backend.remove_missing_coordinates(df_pandas)

    assert isinstance(clean_df, pd.DataFrame)
    assert len(clean_df) == 1  # Only first row has both coordinates
    print(f"✓ Remove missing coordinates: {len(df_pandas)} -> {len(clean_df)} rows")


def test_validate_dataframe_wrapper(sample_df):
    """Test validate_dataframe with wrapper"""
    backend = TSIBackend()
    df_pandas = sample_df.to_pandas()

    is_valid, issues = backend.validate_dataframe(df_pandas)

    assert isinstance(is_valid, bool)
    assert isinstance(issues, list)
    assert is_valid  # Sample data should be valid
    assert len(issues) == 0
    print(f"✓ Validate dataframe: valid={is_valid}, issues={len(issues)}")


def test_time_conversions_static():
    """Test time conversion static methods"""
    from datetime import datetime

    mjd = 59580.5
    dt = TSIBackend.mjd_to_datetime(mjd)
    assert isinstance(dt, datetime)  # Rust returns Python datetime object
    assert dt.year == 2022

    # Convert datetime back to MJD (Rust expects datetime object, not string)
    mjd_back = TSIBackend.datetime_to_mjd(dt)
    assert isinstance(mjd_back, float)
    assert abs(mjd - mjd_back) < 1e-6
    print(f"✓ Time conversions: MJD {mjd} ↔ {dt} (error: {abs(mjd - mjd_back):.10f})")


def test_convenience_functions(sample_df):
    """Test convenience functions"""
    df_pandas = sample_df.to_pandas()

    # compute_metrics convenience function
    metrics = compute_metrics(df_pandas)
    assert isinstance(metrics, dict)
    assert metrics["total_observations"] == 5
    print("✓ Convenience functions: compute_metrics works standalone")


def test_polars_mode(sample_df):
    """Test using Polars mode (zero-copy)"""
    backend = TSIBackend(use_pandas=False)

    # Operations should work with Polars
    metrics = backend.compute_metrics(sample_df)
    assert isinstance(metrics, dict)

    filtered = backend.filter_by_priority(sample_df, min_priority=10.0)
    assert isinstance(filtered, pl.DataFrame)
    assert len(filtered) == 4  # 10, 15, 20, 25
    print(f"✓ Polars mode: Returns Polars DataFrames, {len(filtered)} rows")


def test_pandas_mode(sample_df):
    """Test using pandas mode (default)"""
    backend = TSIBackend(use_pandas=True)

    filtered = backend.filter_by_priority(sample_df.to_pandas(), min_priority=10.0)
    assert isinstance(filtered, pd.DataFrame)
    assert len(filtered) == 4
    print(f"✓ Pandas mode: Returns pandas DataFrames, {len(filtered)} rows")


def test_filter_dataframe_complex(sample_df):
    """Test complex multi-filter"""
    backend = TSIBackend(use_pandas=True)
    df_pandas = sample_df.to_pandas()

    filtered = backend.filter_dataframe(
        df_pandas,
        priority_min=10.0,
        priority_max=20.0,
        scheduled_filter="All",
        priority_bins=["Medium", "High"],
    )

    assert isinstance(filtered, pd.DataFrame)
    assert len(filtered) == 3  # 10 (Medium), 15 (High), 20 (High)
    print(f"✓ Complex filter: {len(df_pandas)} -> {len(filtered)} rows")


def test_error_handling():
    """Test error handling in API"""
    backend = TSIBackend()

    # Invalid DataFrame should raise error
    with pytest.raises(Exception):
        backend.compute_metrics(pd.DataFrame())

    print("✓ Error handling: Exceptions raised correctly")


def test_type_conversions(sample_df):
    """Test automatic type conversions (pandas ↔ polars)"""
    backend = TSIBackend(use_pandas=True)

    # Pass Polars, get pandas back
    result = backend.filter_by_priority(sample_df, min_priority=10.0)
    assert isinstance(result, pd.DataFrame)

    # Pass pandas, get pandas back
    result2 = backend.filter_by_priority(sample_df.to_pandas(), min_priority=10.0)
    assert isinstance(result2, pd.DataFrame)

    print("✓ Type conversions: Automatic pandas ↔ polars conversion works")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
