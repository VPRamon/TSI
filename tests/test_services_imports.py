"""Test suite for services package reorganization.

This test ensures that all public APIs remain accessible after
the services layer reorganization into sub-packages.
"""

import pytest


class TestServicesImports:
    """Test that all services can be imported from their new locations."""

    def test_core_backend_imports(self):
        """Test core backend and database imports."""
        from tsi.services import BACKEND
        from tsi.services import (
            db_health_check,
            fetch_dark_periods_db,
            fetch_possible_periods_db,
            fetch_schedule_db,
            get_distribution_data,
            get_sky_map_data,
            get_visibility_map_data,
            init_database,
            list_schedules_db,
            store_schedule_db,
        )
        
        assert BACKEND is not None

    def test_data_imports(self):
        """Test data loading, preparation, and analytics imports."""
        from tsi.services import (
            AnalyticsSnapshot,
            compute_correlations,
            compute_metrics,
            find_conflicts,
            generate_insights,
            get_filtered_dataframe,
            get_top_observations,
            load_csv,
            load_dark_periods,
            load_schedule_rust,
            prepare_dataframe,
            validate_dataframe,
        )
        
        # Test direct subpackage import
        from tsi.services.data import loaders, analytics, preparation
        
        assert hasattr(loaders, 'load_csv')
        assert hasattr(analytics, 'compute_metrics')

    def test_filters_imports(self):
        """Test filter module imports."""
        from tsi.services import (
            apply_insights_filter,
            build_palette,
            check_filter_support,
            compute_impossible_mask,
            filter_blocks,
            filter_impossible_observations,
        )
        
        # Test direct subpackage import
        from tsi.services.filters import impossible, sky_map
        
        assert hasattr(impossible, 'filter_impossible_observations')
        assert hasattr(sky_map, 'filter_blocks')

    def test_processors_imports(self):
        """Test processor module imports."""
        from tsi.services import (
            apply_search_filters,
            apply_trends_filters,
            calculate_observation_gaps,
            filter_dark_periods,
            filter_scheduled_data,
            get_priority_range,
            get_priority_range_from_blocks,
            get_scheduled_time_range,
            prepare_display_dataframe,
            prepare_priority_bins_from_blocks,
            prepare_scheduled_data,
            validate_required_columns,
        )
        
        # Test direct subpackage import
        from tsi.services.processors import timeline, trends, sky_map, compare
        
        assert hasattr(timeline, 'prepare_scheduled_data')
        assert hasattr(trends, 'apply_trends_filters')
        assert hasattr(sky_map, 'get_priority_range')
        assert hasattr(compare, 'calculate_observation_gaps')

    def test_utils_imports(self):
        """Test utility module imports."""
        from tsi.services import (
            compute_effective_priority_range,
            datetime_to_mjd,
            filter_visibility_blocks,
            format_datetime_utc,
            get_all_block_ids,
            get_time_range,
            mjd_to_datetime,
            parse_optional_mjd,
            parse_visibility_periods,
        )
        
        # Test direct subpackage import
        from tsi.services.utils import time, visibility_processing
        
        assert hasattr(time, 'mjd_to_datetime')
        assert hasattr(visibility_processing, 'filter_visibility_blocks')

    def test_backward_compatibility(self):
        """Test that old import patterns still work via __init__ re-exports."""
        # These should work because services/__init__.py re-exports everything
        from tsi.services import load_csv, compute_metrics, filter_blocks
        from tsi.services import mjd_to_datetime, get_priority_range
        
        # All should be callable/usable
        assert callable(load_csv)
        assert callable(compute_metrics)
        assert callable(filter_blocks)
        assert callable(mjd_to_datetime)
        assert callable(get_priority_range)


class TestSubpackageStructure:
    """Test that subpackages are properly structured."""

    def test_data_subpackage_exists(self):
        """Test data subpackage exists and has expected modules."""
        from tsi.services import data
        
        assert hasattr(data, 'loaders')
        assert hasattr(data, 'analytics')
        assert hasattr(data, 'preparation')

    def test_filters_subpackage_exists(self):
        """Test filters subpackage exists and has expected modules."""
        from tsi.services import filters
        
        assert hasattr(filters, 'impossible')
        assert hasattr(filters, 'sky_map')

    def test_processors_subpackage_exists(self):
        """Test processors subpackage exists and has expected modules."""
        from tsi.services import processors
        
        assert hasattr(processors, 'timeline')
        assert hasattr(processors, 'trends')
        assert hasattr(processors, 'sky_map')
        assert hasattr(processors, 'compare')

    def test_utils_subpackage_exists(self):
        """Test utils subpackage exists and has expected modules."""
        from tsi.services.utils import time, visibility_processing
        # visibility_cache and report are available but not re-exported to avoid circular imports
        from tsi.services.utils import visibility_cache, report
        
        assert hasattr(time, 'mjd_to_datetime')
        assert hasattr(visibility_processing, 'filter_visibility_blocks')
        assert hasattr(visibility_cache, 'ensure_visibility_parsed')
        assert hasattr(report, 'generate_html_report')


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
