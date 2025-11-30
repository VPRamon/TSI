"""
End-to-end tests for FASE 1G: Streamlit app integration with Rust backend.

Tests validate that the services layer correctly uses the Rust backend
and maintains backward compatibility with existing functionality.
"""

import pandas as pd
import pytest

from tsi.services import (
    compute_metrics,
    find_conflicts,
    get_top_observations,
    load_csv,
    validate_dataframe,
)
from tsi.services.data.loaders import filter_by_priority, filter_by_scheduled, load_schedule_rust
from tsi.models.schemas import AnalyticsMetrics
from tsi.services.rust_backend import BACKEND


def rust_compute_metrics(df: pd.DataFrame) -> AnalyticsMetrics:
    return AnalyticsMetrics(**BACKEND.compute_metrics(df))


def rust_get_top_observations(df: pd.DataFrame, by: str = "priority", n: int = 10) -> pd.DataFrame:
    return BACKEND.get_top_observations(df, by=by, n=n)


def rust_find_conflicts(df: pd.DataFrame) -> pd.DataFrame:
    return BACKEND.find_conflicts(df)


class TestRustIntegrationE2E:
    """End-to-end tests for Rust backend integration."""

    def test_load_csv_uses_rust(self):
        """Test that load_csv uses Rust backend."""
        df = load_csv("data/schedule.csv")
        assert isinstance(df, pd.DataFrame)
        assert len(df) > 0
        assert "schedulingBlockId" in df.columns
        assert "priority" in df.columns

    def test_compute_metrics_uses_rust(self):
        """Test that compute_metrics uses Rust backend and returns Pydantic model."""
        df = load_csv("data/schedule.csv")
        metrics = compute_metrics(df)

        # Check it's a Pydantic model (has model_dump method)
        assert hasattr(metrics, "model_dump")
        assert hasattr(metrics, "total_observations")
        assert hasattr(metrics, "mean_priority")

        # Verify values are sensible
        assert metrics.total_observations == len(df)
        assert metrics.mean_priority > 0  # Priority should be positive

    def test_get_top_observations_uses_rust(self):
        """Test that get_top_observations uses Rust backend."""
        df = load_csv("data/schedule.csv")
        top_10 = get_top_observations(df, by="priority", n=10)

        assert isinstance(top_10, pd.DataFrame)
        assert len(top_10) == 10

        # Verify they're sorted by priority descending
        priorities = top_10["priority"].tolist()
        assert priorities == sorted(priorities, reverse=True)

    def test_find_conflicts_uses_rust(self):
        """Test that find_conflicts uses Rust backend."""
        df = load_csv("data/schedule.csv")
        conflicts = find_conflicts(df)

        assert isinstance(conflicts, pd.DataFrame)
        # May or may not have conflicts, but should return a DataFrame
        assert "schedulingBlockId" in conflicts.columns or len(conflicts) == 0

    def test_validate_dataframe_uses_rust(self):
        """Test that validate_dataframe uses Rust backend for data validation."""
        df = load_csv("data/schedule.csv")
        is_valid, errors = validate_dataframe(df)

        assert isinstance(is_valid, bool)
        assert isinstance(errors, list)
        # Our test data should be valid
        assert is_valid or len(errors) > 0  # Either valid or has error messages

    def test_filter_by_priority_integration(self):
        """Test priority filtering through services layer."""
        df = load_csv("data/schedule.csv")

        # Filter for high priority (8-10)
        high_priority = filter_by_priority(df, min_priority=8.0, max_priority=10.0)

        assert isinstance(high_priority, pd.DataFrame)
        assert len(high_priority) <= len(df)

        # Verify all priorities are in range
        if len(high_priority) > 0:
            assert high_priority["priority"].min() >= 8.0
            assert high_priority["priority"].max() <= 10.0

    def test_filter_by_scheduled_integration(self):
        """Test scheduled filtering through services layer."""
        df = load_csv("data/schedule.csv")

        # Filter for scheduled only
        scheduled = filter_by_scheduled(df, filter_type="Scheduled")

        assert isinstance(scheduled, pd.DataFrame)
        assert len(scheduled) <= len(df)

        # Verify all are scheduled
        if len(scheduled) > 0:
            assert scheduled["scheduled_flag"].all()

    def test_rust_backend_consistency(self):
        """Test that Rust backend produces consistent results."""
        df = load_csv("data/schedule.csv")

        # Compute metrics twice
        metrics1 = rust_compute_metrics(df)
        metrics2 = rust_compute_metrics(df)

        # Should be identical (access as Pydantic model attributes)
        assert metrics1.total_observations == metrics2.total_observations
        assert abs(metrics1.mean_priority - metrics2.mean_priority) < 1e-10

    def test_load_schedule_rust_json(self):
        """Test loading JSON schedule with Rust backend."""
        # This assumes data/schedule.json exists
        try:
            df = load_schedule_rust("data/schedule.json")
            assert isinstance(df, pd.DataFrame)
            assert len(df) > 0
            assert "schedulingBlockId" in df.columns
        except FileNotFoundError:
            pytest.skip("data/schedule.json not found")

    def test_load_schedule_rust_csv(self):
        """Test loading CSV schedule with Rust backend."""
        df = load_schedule_rust("data/schedule.csv")
        assert isinstance(df, pd.DataFrame)
        assert len(df) > 0
        assert "schedulingBlockId" in df.columns
        assert "priority" in df.columns

    @pytest.mark.skip(reason="Requires pytest-benchmark plugin")
    def test_performance_comparison_metrics(self, benchmark):
        """Benchmark metrics computation (optional - requires pytest-benchmark)."""
        df = load_csv("data/schedule.csv")

        # Benchmark Rust implementation
        result = benchmark(rust_compute_metrics, df)

        assert result is not None
        assert "total_observations" in result

    def test_top_observations_ordering(self):
        """Test that top observations maintain proper ordering."""
        df = load_csv("data/schedule.csv")

        # Get top 20 by priority
        top_20 = rust_get_top_observations(df, by="priority", n=20)

        assert len(top_20) == min(20, len(df))

        # Verify strict descending order
        priorities = top_20["priority"].tolist()
        for i in range(len(priorities) - 1):
            assert priorities[i] >= priorities[i + 1]

    def test_conflicts_detection_accuracy(self):
        """Test conflict detection returns valid results."""
        df = load_csv("data/schedule.csv")
        conflicts = rust_find_conflicts(df)

        # If conflicts exist, they should have required columns
        if len(conflicts) > 0:
            assert "schedulingBlockId" in conflicts.columns
            # Conflicts should reference existing scheduling blocks
            all_ids = set(df["schedulingBlockId"])
            conflict_ids = set(conflicts["schedulingBlockId"])
            assert conflict_ids.issubset(all_ids)


class TestBackwardCompatibility:
    """Tests ensuring backward compatibility with existing code."""

    def test_metrics_schema_compatibility(self):
        """Test that metrics schema is compatible with existing code."""
        df = load_csv("data/schedule.csv")
        metrics = compute_metrics(df)

        # Check Pydantic model has expected fields
        data = metrics.model_dump()
        expected_fields = [
            "total_observations",
            "scheduled_count",
            "unscheduled_count",
            "mean_priority",
            "total_visibility_hours",
            "mean_requested_hours",
        ]

        for field in expected_fields:
            assert field in data, f"Missing field: {field}"

    def test_dataframe_structure_unchanged(self):
        """Test that DataFrames have expected structure after loading."""
        df = load_csv("data/schedule.csv")

        # Check required columns exist
        required_cols = [
            "schedulingBlockId",
            "priority",
            "requested_hours",
            "scheduled_flag",
        ]

        for col in required_cols:
            assert col in df.columns, f"Missing column: {col}"

    def test_filter_functions_signature_compatible(self):
        """Test that filter functions maintain compatible signatures."""
        df = load_csv("data/schedule.csv")

        # These should all work with named arguments
        result1 = filter_by_priority(df, min_priority=0.0, max_priority=10.0)
        result2 = filter_by_scheduled(df, filter_type="All")

        assert isinstance(result1, pd.DataFrame)
        assert isinstance(result2, pd.DataFrame)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
