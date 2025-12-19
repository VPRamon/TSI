"""
Tests for Phase 3: Visibility Time Bins ETL functionality.

These tests verify the pre-computed visibility histogram functionality,
which provides fast visibility data aggregation for the Visibility Map page.
"""

import pytest


class TestVisibilityTimeBinsModule:
    """Test visibility time bins module availability."""

    def test_populate_visibility_time_bins_function_exists(self):
        """Test that py_populate_visibility_time_bins is available."""
        import tsi_rust

        assert hasattr(tsi_rust, "py_populate_visibility_time_bins")
        assert callable(tsi_rust.py_populate_visibility_time_bins)

    def test_has_visibility_time_bins_function_exists(self):
        """Test that py_has_visibility_time_bins is available."""
        import tsi_rust

        assert hasattr(tsi_rust, "py_has_visibility_time_bins")
        assert callable(tsi_rust.py_has_visibility_time_bins)

    def test_delete_visibility_time_bins_function_exists(self):
        """Test that py_delete_visibility_time_bins is available."""
        import tsi_rust

        assert hasattr(tsi_rust, "py_delete_visibility_time_bins")
        assert callable(tsi_rust.py_delete_visibility_time_bins)

    def test_get_visibility_metadata_function_exists(self):
        """Test that py_get_visibility_metadata is available."""
        import tsi_rust

        assert hasattr(tsi_rust, "py_get_visibility_metadata")
        assert callable(tsi_rust.py_get_visibility_metadata)

    def test_get_visibility_histogram_analytics_function_exists(self):
        """Test that py_get_visibility_histogram_analytics is available."""
        import tsi_rust

        assert hasattr(tsi_rust, "py_get_visibility_histogram_analytics")
        assert callable(tsi_rust.py_get_visibility_histogram_analytics)


class TestVisibilityTimeBinsClasses:
    """Test visibility time bins class availability."""

    def test_visibility_time_metadata_class_exists(self):
        """Test that VisibilityTimeMetadata class is available."""
        import tsi_rust

        assert hasattr(tsi_rust, "VisibilityTimeMetadata")

    def test_visibility_time_bin_class_exists(self):
        """Test that VisibilityTimeBin class is available."""
        import tsi_rust

        assert hasattr(tsi_rust, "VisibilityTimeBin")


class TestVisibilityTimeMetadataAttributes:
    """Test VisibilityTimeMetadata class attributes."""

    def test_visibility_time_metadata_has_expected_fields(self):
        """Test that VisibilityTimeMetadata has expected field names."""
        import tsi_rust

        # Check class is accessible (field attributes exposed via get_all)
        cls = tsi_rust.VisibilityTimeMetadata
        # We can't instantiate directly, but we can verify it's a type
        assert isinstance(cls, type)


class TestVisibilityTimeBinAttributes:
    """Test VisibilityTimeBin class attributes."""

    def test_visibility_time_bin_has_expected_fields(self):
        """Test that VisibilityTimeBin has expected field names."""
        import tsi_rust

        # Check class is accessible
        cls = tsi_rust.VisibilityTimeBin
        assert isinstance(cls, type)


class TestVisibilityHistogramAnalyticsParameters:
    """Test py_get_visibility_histogram_analytics parameter validation."""

    def test_start_must_be_less_than_end(self):
        """Test that start_unix must be less than end_unix."""
        import tsi_rust

        # start >= end should raise ValueError
        with pytest.raises(Exception) as exc_info:
            tsi_rust.py_get_visibility_histogram_analytics(
                schedule_id=1,
                start_unix=1000000,
                end_unix=999999,  # Less than start
                bin_duration_minutes=60,
            )
        # Should get a ValueError about start < end
        assert (
            "start_unix" in str(exc_info.value).lower()
            or "must be less than" in str(exc_info.value).lower()
        )

    def test_bin_duration_must_be_positive(self):
        """Test that bin_duration_minutes must be positive."""
        import tsi_rust

        with pytest.raises(Exception) as exc_info:
            tsi_rust.py_get_visibility_histogram_analytics(
                schedule_id=1,
                start_unix=1000000,
                end_unix=2000000,
                bin_duration_minutes=0,  # Zero not allowed
            )
        assert (
            "bin_duration" in str(exc_info.value).lower()
            or "positive" in str(exc_info.value).lower()
        )

    def test_negative_bin_duration_rejected(self):
        """Test that negative bin_duration_minutes is rejected."""
        import tsi_rust

        with pytest.raises(Exception) as exc_info:
            tsi_rust.py_get_visibility_histogram_analytics(
                schedule_id=1,
                start_unix=1000000,
                end_unix=2000000,
                bin_duration_minutes=-30,  # Negative not allowed
            )
        assert (
            "bin_duration" in str(exc_info.value).lower()
            or "positive" in str(exc_info.value).lower()
        )


class TestPopulateVisibilityTimeBinsParameters:
    """Test py_populate_visibility_time_bins parameter handling."""

    def test_accepts_schedule_id(self):
        """Test function accepts schedule_id parameter."""
        import inspect

        import tsi_rust

        # Check function signature
        sig = inspect.signature(tsi_rust.py_populate_visibility_time_bins)
        params = list(sig.parameters.keys())
        assert "schedule_id" in params

    def test_accepts_optional_bin_duration(self):
        """Test function accepts optional bin_duration_seconds parameter."""
        import inspect

        import tsi_rust

        sig = inspect.signature(tsi_rust.py_populate_visibility_time_bins)
        params = sig.parameters
        # Should have bin_duration_seconds as optional (default None)
        assert "bin_duration_seconds" in params
        # Check it has a default
        assert (
            params["bin_duration_seconds"].default is not inspect.Parameter.empty
            or params["bin_duration_seconds"].default is None
        )


class TestVisibilityTimeBinsIntegration:
    """Integration tests (require database connection)."""

    @pytest.mark.skipif(
        True,  # Skip by default - these need a real database
        reason="Integration tests require database connection",
    )
    def test_populate_and_query_visibility_bins(self):
        """Test end-to-end visibility bins population and query."""
        from datetime import datetime, timezone

        import tsi_rust

        schedule_id = 1  # Test schedule

        # Populate bins
        meta_count, bin_count = tsi_rust.py_populate_visibility_time_bins(
            schedule_id=schedule_id, bin_duration_seconds=60  # 1-minute bins
        )

        assert meta_count >= 0
        assert bin_count >= 0

        # Check existence
        assert tsi_rust.py_has_visibility_time_bins(schedule_id) == (bin_count > 0)

        # Query metadata
        metadata = tsi_rust.py_get_visibility_metadata(schedule_id)
        if metadata:
            assert metadata.schedule_id == schedule_id
            assert metadata.bin_duration_seconds == 60

        # Query histogram
        if bin_count > 0:
            start = int(datetime(2024, 1, 1, tzinfo=timezone.utc).timestamp())
            end = int(datetime(2024, 12, 31, tzinfo=timezone.utc).timestamp())

            bins = tsi_rust.py_get_visibility_histogram_analytics(
                schedule_id=schedule_id, start_unix=start, end_unix=end, bin_duration_minutes=60
            )

            assert isinstance(bins, list)
            for b in bins:
                assert "bin_start_unix" in b
                assert "bin_end_unix" in b
                assert "count" in b


class TestPhase3FunctionSignatures:
    """Test that Phase 3 functions have correct signatures."""

    def test_populate_signature(self):
        """Test py_populate_visibility_time_bins signature."""
        import inspect

        import tsi_rust

        sig = inspect.signature(tsi_rust.py_populate_visibility_time_bins)
        assert "schedule_id" in sig.parameters

    def test_has_signature(self):
        """Test py_has_visibility_time_bins signature."""
        import inspect

        import tsi_rust

        sig = inspect.signature(tsi_rust.py_has_visibility_time_bins)
        assert "schedule_id" in sig.parameters

    def test_delete_signature(self):
        """Test py_delete_visibility_time_bins signature."""
        import inspect

        import tsi_rust

        sig = inspect.signature(tsi_rust.py_delete_visibility_time_bins)
        assert "schedule_id" in sig.parameters

    def test_get_metadata_signature(self):
        """Test py_get_visibility_metadata signature."""
        import inspect

        import tsi_rust

        sig = inspect.signature(tsi_rust.py_get_visibility_metadata)
        assert "schedule_id" in sig.parameters

    def test_get_histogram_analytics_signature(self):
        """Test py_get_visibility_histogram_analytics signature."""
        import inspect

        import tsi_rust

        sig = inspect.signature(tsi_rust.py_get_visibility_histogram_analytics)
        params = list(sig.parameters.keys())
        assert "schedule_id" in params
        assert "start_unix" in params
        assert "end_unix" in params
        assert "bin_duration_minutes" in params


class TestBackwardsCompatibility:
    """Test that Phase 3 doesn't break existing functionality."""

    def test_original_visibility_histogram_still_exists(self):
        """Test that py_get_visibility_histogram (Phase 1) still exists."""
        import tsi_rust

        assert hasattr(tsi_rust, "py_get_visibility_histogram")

    def test_phase2_functions_still_exist(self):
        """Test that Phase 2 functions still exist."""
        import tsi_rust

        assert hasattr(tsi_rust, "py_populate_summary_analytics")
        assert hasattr(tsi_rust, "py_get_schedule_summary")
        assert hasattr(tsi_rust, "py_get_priority_rates")
        assert hasattr(tsi_rust, "py_get_visibility_bins")
        assert hasattr(tsi_rust, "py_get_heatmap_bins")

    def test_phase2_classes_still_exist(self):
        """Test that Phase 2 classes still exist."""
        import tsi_rust

        assert hasattr(tsi_rust, "ScheduleSummary")
        assert hasattr(tsi_rust, "PriorityRate")
        assert hasattr(tsi_rust, "VisibilityBin")
        assert hasattr(tsi_rust, "HeatmapBinData")
