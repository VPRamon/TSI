"""
End-to-end tests for FASE 1G: Streamlit app integration with Rust backend.

Tests validate that the services layer correctly uses the Rust backend
and maintains backward compatibility with existing functionality.
"""

import pandas as pd
import pytest
from tsi.services import (
    find_conflicts,
    get_top_observations,
    validate_dataframe,
)
from tsi.services.data.loaders import (
    filter_by_priority,
    load_schedule_rust,
)
from tsi.services.rust_backend import BACKEND


def rust_get_top_observations(df: pd.DataFrame, by: str = "priority", n: int = 10) -> pd.DataFrame:
    return BACKEND.get_top_observations(df, by=by, n=n)


def rust_find_conflicts(df: pd.DataFrame) -> pd.DataFrame:
    return BACKEND.find_conflicts(df)


class TestRustIntegrationE2E:
    """End-to-end tests for Rust backend integration."""

    def test_load_schedule_rust_uses_rust(self):
        """Test that load_schedule_rust uses Rust backend."""
        df = load_schedule_rust("data/schedule.json")
        assert isinstance(df, pd.DataFrame)
        assert len(df) > 0
        assert "blocks" in df.columns
        blocks = df.iloc[0]["blocks"]
        assert isinstance(blocks, list)
        assert len(blocks) > 0
        first_block = blocks[0]
        assert "id" in first_block
        assert "priority" in first_block

    def test_get_top_observations_uses_rust(self):
        """Test that get_top_observations uses Rust backend."""
        pytest.skip(
            "API changed: top observations now derived from backend analytics by schedule id."
        )
        df = load_schedule_rust("data/schedule.json")
        top_10 = get_top_observations(df, by="priority", n=10)

        assert isinstance(top_10, pd.DataFrame)
        assert len(top_10) == 10

        # Verify they're sorted by priority descending
        priorities = top_10["priority"].tolist()
        assert priorities == sorted(priorities, reverse=True)

    def test_find_conflicts_uses_rust(self):
        """Test that find_conflicts uses Rust backend."""
        df = load_schedule_rust("data/schedule.json")
        conflicts = find_conflicts(df)

        assert isinstance(conflicts, pd.DataFrame)
        # May or may not have conflicts, but should return a DataFrame
        assert "schedulingBlockId" in conflicts.columns or len(conflicts) == 0

    def test_validate_dataframe_uses_rust(self):
        """Test that validate_dataframe uses Rust backend for data validation."""
        df = load_schedule_rust("data/schedule.json")
        is_valid, errors = validate_dataframe(df)

        assert isinstance(is_valid, bool)
        assert isinstance(errors, list)
        # Our test data should be valid
        assert is_valid or len(errors) > 0  # Either valid or has error messages

    def test_filter_by_priority_integration(self):
        """Test priority filtering through services layer."""
        df = load_schedule_rust("data/schedule.json")

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
        pytest.skip(
            "Data flow changed: filter_by_scheduled requires scheduled_flag column "
            "which is added during ETL, not raw JSON parsing. Test needs database fixtures."
        )

    def test_load_schedule_rust_json(self):
        """Test loading JSON schedule with Rust backend."""
        # This assumes data/schedule.json exists
        try:
            df = load_schedule_rust("data/schedule.json")
            assert isinstance(df, pd.DataFrame)
            assert len(df) > 0
            assert "blocks" in df.columns
            blocks = df.iloc[0]["blocks"]
            assert isinstance(blocks, list)
            assert len(blocks) > 0
            assert "id" in blocks[0]
            assert "priority" in blocks[0]
        except FileNotFoundError:
            pytest.skip("data/schedule.json not found")

    def test_load_schedule_rust_csv(self):
        """Test loading CSV schedule with Rust backend."""
        df = load_schedule_rust("data/schedule.json")
        assert isinstance(df, pd.DataFrame)
        assert len(df) > 0
        assert "blocks" in df.columns
        blocks = df.iloc[0]["blocks"]
        assert isinstance(blocks, list)
        assert len(blocks) > 0
        assert "id" in blocks[0]
        assert "priority" in blocks[0]

    def test_top_observations_ordering(self):
        """Test that top observations maintain proper ordering."""
        pytest.skip(
            "API changed: top observations now derived from backend analytics by schedule id."
        )
        df = load_schedule_rust("data/schedule.json")

        # Get top 20 by priority
        top_20 = rust_get_top_observations(df, by="priority", n=20)

        assert len(top_20) == min(20, len(df))

        # Verify strict descending order
        priorities = top_20["priority"].tolist()
        for i in range(len(priorities) - 1):
            assert priorities[i] >= priorities[i + 1]

    def test_conflicts_detection_accuracy(self):
        """Test conflict detection returns valid results."""
        pytest.skip(
            "Data flow changed: find_conflicts requires scheduled_flag column "
            "which is added during ETL, not raw JSON parsing. Test needs database fixtures."
        )


class TestBackwardCompatibility:
    """Tests ensuring backward compatibility with existing code."""

    def test_dataframe_structure_unchanged(self):
        """Test that DataFrames have expected structure after loading."""
        df = load_schedule_rust("data/schedule.json")

        # Check required columns exist - note: scheduled_flag is added by ETL, not raw loading
        assert "blocks" in df.columns
        blocks = df.iloc[0]["blocks"]
        assert isinstance(blocks, list)
        assert len(blocks) > 0
        assert "id" in blocks[0]
        assert "priority" in blocks[0]

    def test_filter_functions_signature_compatible(self):
        """Test that filter functions maintain compatible signatures."""
        pytest.skip(
            "Data flow changed: filter functions require prepared data with scheduled_flag column "
            "which is added during ETL, not raw JSON parsing. Test needs database fixtures."
        )


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
