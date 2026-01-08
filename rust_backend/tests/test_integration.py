"""
Integration tests for Rust backend from Python
"""
import pytest
from datetime import datetime, timezone


def test_import_module():
    """Test that the tsi_rust module can be imported"""
    import tsi_rust

    assert tsi_rust is not None


@pytest.mark.skip(reason="mjd_to_datetime function not exposed in tsi_rust module")
def test_mjd_to_datetime():
    """Test MJD to datetime conversion"""
    import tsi_rust

    # MJD 59580.0 ≈ 2022-01-01 00:00:00 UTC
    result = tsi_rust.mjd_to_datetime(59580.0)
    assert result.year == 2022
    assert result.month == 1
    assert result.day == 1


