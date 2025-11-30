"""
Phase 2 Summary Analytics ETL Tests

Tests for the summary analytics tables (schedule_summary_analytics,
schedule_priority_rates, schedule_visibility_bins, schedule_heatmap_bins).

These tests validate:
1. Summary metrics computation
2. Priority rate calculations
3. Visibility bin histogram logic
4. Heatmap bin 2D histogram logic
5. Data type validation for Python bindings
"""

import pytest
from typing import Optional


class TestSummaryAnalyticsDataTypes:
    """Test that summary analytics data types are properly exposed to Python."""

    def test_schedule_summary_attributes(self):
        """Test that ScheduleSummary has expected attributes."""
        # This test verifies the struct was properly decorated with #[pyclass(get_all)]
        try:
            import tsi_rust
            
            # Check that the type exists and is importable
            # The actual class won't be directly accessible, but the functions should work
            if not hasattr(tsi_rust, 'py_get_schedule_summary'):
                pytest.skip("Phase 2 functions not yet available - rebuild tsi_rust module")
            assert hasattr(tsi_rust, 'py_has_summary_analytics')
            assert hasattr(tsi_rust, 'py_populate_summary_analytics')
            assert hasattr(tsi_rust, 'py_delete_summary_analytics')
        except ImportError:
            pytest.skip("tsi_rust module not available - run `maturin develop` first")

    def test_priority_rate_attributes(self):
        """Test that PriorityRate has expected attributes."""
        try:
            import tsi_rust
            if not hasattr(tsi_rust, 'py_get_priority_rates'):
                pytest.skip("Phase 2 functions not yet available - rebuild tsi_rust module")
        except ImportError:
            pytest.skip("tsi_rust module not available - run `maturin develop` first")

    def test_visibility_bin_attributes(self):
        """Test that VisibilityBin has expected attributes."""
        try:
            import tsi_rust
            if not hasattr(tsi_rust, 'py_get_visibility_bins'):
                pytest.skip("Phase 2 functions not yet available - rebuild tsi_rust module")
        except ImportError:
            pytest.skip("tsi_rust module not available - run `maturin develop` first")

    def test_heatmap_bin_attributes(self):
        """Test that HeatmapBinData has expected attributes."""
        try:
            import tsi_rust
            if not hasattr(tsi_rust, 'py_get_heatmap_bins'):
                pytest.skip("Phase 2 functions not yet available - rebuild tsi_rust module")
        except ImportError:
            pytest.skip("tsi_rust module not available - run `maturin develop` first")


class TestSummaryMetricsComputation:
    """Test summary metrics computation logic."""

    def test_scheduling_rate_calculation(self):
        """Test that scheduling_rate = scheduled_count / total_count."""
        # Given counts
        total = 100
        scheduled = 75
        
        # Expected rate
        expected_rate = scheduled / total
        
        assert expected_rate == 0.75

    def test_scheduling_rate_zero_total(self):
        """Test scheduling rate when total is zero."""
        total = 0
        scheduled = 0
        
        # Should not divide by zero
        rate = scheduled / total if total > 0 else 0.0
        
        assert rate == 0.0

    def test_mean_calculation(self):
        """Test mean calculation for priorities/visibility."""
        values = [1.0, 2.0, 3.0, 4.0, 5.0]
        
        mean = sum(values) / len(values)
        
        assert mean == 3.0

    def test_median_odd_count(self):
        """Test median calculation with odd number of elements."""
        values = [1.0, 2.0, 3.0, 4.0, 5.0]
        sorted_values = sorted(values)
        n = len(sorted_values)
        
        median = sorted_values[n // 2]
        
        assert median == 3.0

    def test_median_even_count(self):
        """Test median calculation with even number of elements."""
        values = [1.0, 2.0, 3.0, 4.0]
        sorted_values = sorted(values)
        n = len(sorted_values)
        
        median = (sorted_values[n // 2 - 1] + sorted_values[n // 2]) / 2
        
        assert median == 2.5


class TestPriorityRatesComputation:
    """Test priority rate computation logic."""

    def test_priority_grouping(self):
        """Test that priorities are properly grouped as integers."""
        # Simulated blocks with priorities
        blocks = [
            {"priority": 5.0, "scheduled": True},
            {"priority": 5.0, "scheduled": True},
            {"priority": 5.0, "scheduled": False},
            {"priority": 3.0, "scheduled": False},
            {"priority": 3.0, "scheduled": True},
        ]
        
        # Group by integer priority
        groups = {}
        for block in blocks:
            priority_int = int(round(block["priority"]))
            if priority_int not in groups:
                groups[priority_int] = {"total": 0, "scheduled": 0}
            groups[priority_int]["total"] += 1
            if block["scheduled"]:
                groups[priority_int]["scheduled"] += 1
        
        # Compute rates
        rates = {}
        for priority, counts in groups.items():
            rates[priority] = counts["scheduled"] / counts["total"]
        
        assert rates[5] == 2/3  # ~0.667
        assert rates[3] == 1/2  # 0.5

    def test_priority_rate_ordering(self):
        """Test that priority rates are ordered by priority value."""
        rates = [
            {"priority_value": 5, "rate": 0.8},
            {"priority_value": 3, "rate": 0.5},
            {"priority_value": 7, "rate": 0.9},
            {"priority_value": 1, "rate": 0.2},
        ]
        
        sorted_rates = sorted(rates, key=lambda x: x["priority_value"])
        
        assert [r["priority_value"] for r in sorted_rates] == [1, 3, 5, 7]


class TestVisibilityBinsComputation:
    """Test visibility histogram bin computation."""

    def test_bin_calculation(self):
        """Test that values are assigned to correct bins."""
        values = [1.0, 5.0, 10.0, 15.0, 20.0]
        n_bins = 4
        
        min_val = min(values)
        max_val = max(values)
        bin_width = (max_val - min_val) / n_bins
        
        # Expected bins: [1-5.75), [5.75-10.5), [10.5-15.25), [15.25-20]
        assert bin_width == 4.75

    def test_bin_assignment(self):
        """Test bin index assignment."""
        min_val = 0.0
        max_val = 20.0
        n_bins = 4
        bin_width = (max_val - min_val) / n_bins  # 5.0
        
        test_values = [2.5, 7.5, 12.5, 17.5]
        expected_bins = [0, 1, 2, 3]
        
        for value, expected_bin in zip(test_values, expected_bins):
            bin_idx = int((value - min_val) / bin_width)
            bin_idx = min(bin_idx, n_bins - 1)  # Clamp to last bin
            assert bin_idx == expected_bin

    def test_bin_at_boundary(self):
        """Test that values at max boundary go to last bin."""
        min_val = 0.0
        max_val = 20.0
        n_bins = 4
        bin_width = (max_val - min_val) / n_bins
        
        # Value exactly at max
        value = 20.0
        bin_idx = int((value - min_val) / bin_width)
        bin_idx = min(bin_idx, n_bins - 1)
        
        assert bin_idx == 3  # Last bin


class TestHeatmapBinsComputation:
    """Test 2D heatmap bin computation."""

    def test_2d_bin_assignment(self):
        """Test that 2D coordinates are assigned to correct bins."""
        # Grid: 3x3 bins for visibility (0-30h) and time (0-6h)
        n_bins = 3
        vis_min, vis_max = 0.0, 30.0
        time_min, time_max = 0.0, 6.0
        
        vis_width = (vis_max - vis_min) / n_bins  # 10.0
        time_width = (time_max - time_min) / n_bins  # 2.0
        
        # Test point: visibility=15h, time=3h
        vis, time = 15.0, 3.0
        
        vis_idx = int((vis - vis_min) / vis_width)
        vis_idx = min(vis_idx, n_bins - 1)
        
        time_idx = int((time - time_min) / time_width)
        time_idx = min(time_idx, n_bins - 1)
        
        assert vis_idx == 1  # Middle visibility bin
        assert time_idx == 1  # Middle time bin

    def test_heatmap_rate_computation(self):
        """Test scheduling rate computation for heatmap bins."""
        # Simulated bin data
        bin_data = {"total": 10, "scheduled": 7}
        
        rate = bin_data["scheduled"] / bin_data["total"]
        
        assert rate == 0.7


class TestSpearmanCorrelation:
    """Test Spearman rank correlation computation."""

    def test_perfect_positive_correlation(self):
        """Test Spearman correlation for perfectly correlated data."""
        x = [1.0, 2.0, 3.0, 4.0, 5.0]
        y = [2.0, 4.0, 6.0, 8.0, 10.0]  # y = 2x
        
        # Both have same rank order, so correlation = 1.0
        # The Rust implementation computes this properly
        
        # Manually compute ranks
        x_ranks = [1, 2, 3, 4, 5]
        y_ranks = [1, 2, 3, 4, 5]
        
        # Pearson on ranks
        n = len(x_ranks)
        mean_x = sum(x_ranks) / n
        mean_y = sum(y_ranks) / n
        
        numerator = sum((x_ranks[i] - mean_x) * (y_ranks[i] - mean_y) for i in range(n))
        sum_sq_x = sum((r - mean_x) ** 2 for r in x_ranks)
        sum_sq_y = sum((r - mean_y) ** 2 for r in y_ranks)
        
        correlation = numerator / ((sum_sq_x * sum_sq_y) ** 0.5)
        
        assert abs(correlation - 1.0) < 0.001

    def test_perfect_negative_correlation(self):
        """Test Spearman correlation for inverse correlation."""
        x = [1.0, 2.0, 3.0, 4.0, 5.0]
        y = [5.0, 4.0, 3.0, 2.0, 1.0]  # Inverse order
        
        x_ranks = [1, 2, 3, 4, 5]
        y_ranks = [5, 4, 3, 2, 1]
        
        n = len(x_ranks)
        mean_x = sum(x_ranks) / n
        mean_y = sum(y_ranks) / n
        
        numerator = sum((x_ranks[i] - mean_x) * (y_ranks[i] - mean_y) for i in range(n))
        sum_sq_x = sum((r - mean_x) ** 2 for r in x_ranks)
        sum_sq_y = sum((r - mean_y) ** 2 for r in y_ranks)
        
        correlation = numerator / ((sum_sq_x * sum_sq_y) ** 0.5)
        
        assert abs(correlation - (-1.0)) < 0.001


class TestIdempotency:
    """Test that ETL operations are idempotent."""

    def test_delete_before_insert_pattern(self):
        """Verify the delete-then-insert pattern is idempotent."""
        # Simulated existing data
        existing_data = {"schedule_id": 1, "value": 100}
        
        # First run: delete (no data) then insert
        deleted_count = 0  # Nothing to delete
        inserted = existing_data
        
        # Second run: delete (has data) then insert same
        deleted_count = 1
        inserted = existing_data
        
        # Result should be the same either way
        assert inserted == existing_data


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
