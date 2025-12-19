"""Test suite for services package reorganization.

This test ensures that all public APIs remain accessible after
the services layer reorganization into sub-packages.
"""

import pytest


class TestServicesImports:
    """Test that all services can be imported from their new locations."""

    def test_core_backend_imports(self):
        """Test core backend and database imports."""
        from tsi.services import (
            BACKEND,
        )

        assert BACKEND is not None

    def test_data_imports(self):
        """Test data loading, preparation, and analytics imports."""

        # Test direct subpackage import
        from tsi.services.data import analytics, loaders

        assert hasattr(loaders, "load_json")
        assert hasattr(analytics, "compute_metrics")

    def test_filters_imports(self):
        """Test filter module imports."""

        # Test direct subpackage import
        from tsi.services.filters import impossible, sky_map

        assert hasattr(impossible, "filter_impossible_observations")
        assert hasattr(sky_map, "filter_blocks")

    def test_processors_imports(self):
        """Test processor module imports."""

        # Test direct subpackage import
        from tsi.services.processors import compare, sky_map, timeline, trends

        assert hasattr(timeline, "prepare_scheduled_data")
        assert hasattr(trends, "apply_trends_filters")
        assert hasattr(sky_map, "get_priority_range")
        assert hasattr(compare, "calculate_observation_gaps")

    def test_utils_imports(self):
        """Test utility module imports."""

        # Test direct subpackage import
        from tsi.services.utils import time, visibility_processing

        assert hasattr(time, "mjd_to_datetime")
        assert hasattr(visibility_processing, "filter_visibility_blocks")

    def test_backward_compatibility(self):
        """Test that old import patterns still work via __init__ re-exports."""
        # These should work because services/__init__.py re-exports everything
        from tsi.services import (
            compute_metrics,
            filter_blocks,
            get_priority_range,
            load_json,
            mjd_to_datetime,
        )

        # All should be callable/usable
        assert callable(load_json)
        assert callable(compute_metrics)
        assert callable(filter_blocks)
        assert callable(mjd_to_datetime)
        assert callable(get_priority_range)


class TestSubpackageStructure:
    """Test that subpackages are properly structured."""

    def test_data_subpackage_exists(self):
        """Test data subpackage exists and has expected modules."""
        from tsi.services import data

        assert hasattr(data, "loaders")
        assert hasattr(data, "analytics")
        assert hasattr(data, "preparation")

    def test_filters_subpackage_exists(self):
        """Test filters subpackage exists and has expected modules."""
        from tsi.services import filters

        assert hasattr(filters, "impossible")
        assert hasattr(filters, "sky_map")

    def test_processors_subpackage_exists(self):
        """Test processors subpackage exists and has expected modules."""
        from tsi.services import processors

        assert hasattr(processors, "timeline")
        assert hasattr(processors, "trends")
        assert hasattr(processors, "sky_map")
        assert hasattr(processors, "compare")

    def test_utils_subpackage_exists(self):
        """Test utils subpackage exists and has expected modules."""
        # visibility_cache and report are available but not re-exported to avoid circular imports
        from tsi.services.utils import report, time, visibility_cache, visibility_processing

        assert hasattr(time, "mjd_to_datetime")
        assert hasattr(visibility_processing, "filter_visibility_blocks")
        assert hasattr(visibility_cache, "ensure_visibility_parsed")
        assert hasattr(report, "generate_html_report")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
