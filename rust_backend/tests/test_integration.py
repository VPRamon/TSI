"""
Integration tests for Rust backend from Python
"""
import pytest
from datetime import datetime, timezone


def test_import_module():
    """Test that the tsi_rust module can be imported"""
    import tsi_rust
    assert tsi_rust is not None


def test_mjd_to_datetime():
    """Test MJD to datetime conversion"""
    import tsi_rust
    
    # MJD 59580.0 ≈ 2022-01-01 00:00:00 UTC
    result = tsi_rust.mjd_to_datetime(59580.0)
    assert result.year == 2022
    assert result.month == 1
    assert result.day == 1


def test_datetime_to_mjd():
    """Test datetime to MJD conversion"""
    import tsi_rust
    
    dt = datetime(2022, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
    mjd = tsi_rust.datetime_to_mjd(dt)
    
    # Should be approximately 59580.0
    assert abs(mjd - 59580.0) < 1.0


def test_mjd_roundtrip():
    """Test roundtrip MJD -> datetime -> MJD"""
    import tsi_rust
    
    original_mjd = 59580.123456
    dt = tsi_rust.mjd_to_datetime(original_mjd)
    back_to_mjd = tsi_rust.datetime_to_mjd(dt)
    
    # Should be very close (within microsecond precision)
    assert abs(original_mjd - back_to_mjd) < 1e-6


def test_parse_empty_visibility():
    """Test parsing empty visibility periods"""
    import tsi_rust
    
    result = tsi_rust.parse_visibility_periods("[]")
    assert len(result) == 0


def test_parse_single_visibility_period():
    """Test parsing single visibility period"""
    import tsi_rust
    
    input_str = "[(59580.0, 59581.0)]"
    result = tsi_rust.parse_visibility_periods(input_str)
    
    assert len(result) == 1
    start, stop = result[0]
    assert start.year == 2022
    assert stop.year == 2022


def test_parse_multiple_visibility_periods():
    """Test parsing multiple visibility periods"""
    import tsi_rust
    
    input_str = "[(59580.0, 59580.5), (59581.0, 59581.25)]"
    result = tsi_rust.parse_visibility_periods(input_str)
    
    assert len(result) == 2
    
    # First period
    start1, stop1 = result[0]
    assert start1.year == 2022
    
    # Second period
    start2, stop2 = result[1]
    assert start2.year == 2022


def test_performance_batch_mjd_conversion():
    """Test performance of batch MJD conversions"""
    import tsi_rust
    import time
    
    mjd_values = [59580.0 + i * 0.01 for i in range(10000)]
    
    start = time.time()
    for mjd in mjd_values:
        tsi_rust.mjd_to_datetime(mjd)
    elapsed = time.time() - start
    
    print(f"\n✅ Converted {len(mjd_values)} MJD values in {elapsed:.3f}s")
    print(f"   ({len(mjd_values)/elapsed:.0f} conversions/sec)")
    
    # Should be fast (at least 10k conversions per second)
    assert elapsed < 1.0


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
