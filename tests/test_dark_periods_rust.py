"""Test dark periods loading with Rust backend."""

from pathlib import Path

import pytest


@pytest.mark.skip(
    reason="load_dark_periods requires database initialization - function was never working standalone"
)
def test_load_dark_periods_rust():
    """Test loading dark periods using Rust backend."""
    try:
        from tsi_rust_api import load_dark_periods
    except ImportError:
        pytest.skip("Rust backend not available (needs rebuild)")

    # Get path to test data
    repo_root = Path(__file__).parent.parent
    dark_periods_path = repo_root / "data" / "dark_periods.json"

    if not dark_periods_path.exists():
        pytest.skip("dark_periods.json not found")

    # Load dark periods
    df = load_dark_periods(dark_periods_path)

    # Verify DataFrame structure
    assert not df.empty, "DataFrame should not be empty"
    assert "start_dt" in df.columns, "Missing start_dt column"
    assert "stop_dt" in df.columns, "Missing stop_dt column"
    assert "start_mjd" in df.columns, "Missing start_mjd column"
    assert "stop_mjd" in df.columns, "Missing stop_mjd column"
    assert "duration_hours" in df.columns, "Missing duration_hours column"
    assert "months" in df.columns, "Missing months column"

    # Verify data types
    assert df["start_mjd"].dtype == "float64", "start_mjd should be float64"
    assert df["stop_mjd"].dtype == "float64", "stop_mjd should be float64"
    assert df["duration_hours"].dtype == "float64", "duration_hours should be float64"

    # Verify data consistency
    assert (df["stop_mjd"] > df["start_mjd"]).all(), "stop_mjd should be after start_mjd"
    assert (df["duration_hours"] > 0).all(), "duration_hours should be positive"

    # Verify that months is a list-like column (can be list or numpy array after conversion)
    first_months = df["months"].iloc[0]
    assert hasattr(first_months, "__iter__"), "months should be list-like (iterable)"
    assert hasattr(first_months, "__len__"), "months should have length"
    if len(first_months) > 0:
        assert isinstance(first_months[0], str), "months items should be strings"
        assert "-" in first_months[0], "month format should be YYYY-MM"

    print(f"✓ Loaded {len(df)} dark periods successfully")
    print(f"  Columns: {list(df.columns)}")
    print(f"  Date range: {df['start_mjd'].min():.2f} to {df['stop_mjd'].max():.2f} MJD")
    print(f"  Total duration: {df['duration_hours'].sum():.1f} hours")


if __name__ == "__main__":
    test_load_dark_periods_rust()
