"""Test the visibility histogram Python binding."""

import pytest


def test_visibility_histogram_import():
    """Test that we can import the visibility histogram function."""
    try:
        import tsi_rust

        assert hasattr(tsi_rust, "py_get_visibility_histogram")
    except ImportError:
        pytest.skip("tsi_rust module not available")


def test_visibility_histogram_validation():
    """Test that validation errors are raised correctly."""
    try:
        import tsi_rust
    except ImportError:
        pytest.skip("tsi_rust module not available")

    # Test invalid start/end
    with pytest.raises(ValueError, match="start_unix must be less than end_unix"):
        tsi_rust.py_get_visibility_histogram(
            schedule_id=1,
            start_unix=100,
            end_unix=50,
            bin_duration_minutes=60,
        )

    # Test invalid bin duration
    with pytest.raises(ValueError, match="bin_duration_minutes must be positive"):
        tsi_rust.py_get_visibility_histogram(
            schedule_id=1,
            start_unix=0,
            end_unix=100,
            bin_duration_minutes=0,
        )


def test_visibility_histogram_return_type():
    """Test that the function returns the correct type."""
    try:
        import tsi_rust
    except ImportError:
        pytest.skip("tsi_rust module not available")

    # This will fail with DB connection error, but we can check the error type
    # indicates the function signature is correct
    try:
        result = tsi_rust.py_get_visibility_histogram(
            schedule_id=1,
            start_unix=0,
            end_unix=86400,
            bin_duration_minutes=60,
        )
        # If we get here, check the result type
        assert isinstance(result, list)
        if len(result) > 0:
            assert isinstance(result[0], dict)
            assert "bin_start_unix" in result[0]
            assert "bin_end_unix" in result[0]
            assert "count" in result[0]
    except RuntimeError as e:
        # Expected if DB is not available
        assert "Failed to" in str(e) or "connection" in str(e).lower()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
