"""Unit tests for consolidated priority range calculation."""

from __future__ import annotations

from types import SimpleNamespace

import pandas as pd
import pytest

from tsi.services.processors.sky_map import get_priority_range


pytestmark = pytest.mark.unit


class TestGetPriorityRange:
    """Tests for the unified get_priority_range function."""

    def test_dataframe_with_normal_priorities(self):
        """Should return min and max from DataFrame with normal priority values."""
        df = pd.DataFrame({"priority": [1.0, 5.0, 10.0]})
        result = get_priority_range(df)
        assert result == (1.0, 10.0)

    def test_dataframe_with_single_priority(self):
        """Should handle single priority value by adding 1.0 to max."""
        df = pd.DataFrame({"priority": [5.0, 5.0, 5.0]})
        result = get_priority_range(df)
        assert result == (5.0, 6.0)

    def test_dataframe_with_empty_priorities(self):
        """Should return default range for empty priorities."""
        df = pd.DataFrame({"priority": []})
        result = get_priority_range(df)
        assert result == (0.0, 10.0)

    def test_dataframe_with_null_priorities(self):
        """Should handle null values by dropping them."""
        df = pd.DataFrame({"priority": [None, 5.0, None, 10.0]})
        result = get_priority_range(df)
        assert result == (5.0, 10.0)

    def test_dataframe_missing_priority_column(self):
        """Should return default range when priority column is missing."""
        df = pd.DataFrame({"other_column": [1, 2, 3]})
        result = get_priority_range(df)
        assert result == (0.0, 10.0)

    def test_blocks_list_with_normal_priorities(self):
        """Should return min and max from blocks list with normal priorities."""
        blocks = [
            SimpleNamespace(priority=1.0),
            SimpleNamespace(priority=5.0),
            SimpleNamespace(priority=10.0),
        ]
        result = get_priority_range(blocks)
        assert result == (1.0, 10.0)

    def test_blocks_list_with_single_priority(self):
        """Should handle single priority value in blocks by adding 1.0 to max."""
        blocks = [
            SimpleNamespace(priority=5.0),
            SimpleNamespace(priority=5.0),
        ]
        result = get_priority_range(blocks)
        assert result == (5.0, 6.0)

    def test_empty_blocks_list(self):
        """Should return default range for empty blocks list."""
        result = get_priority_range([])
        assert result == (0.0, 10.0)

    def test_float_conversion(self):
        """Should properly convert to float type."""
        df = pd.DataFrame({"priority": [1, 5, 10]})  # integers
        min_pri, max_pri = get_priority_range(df)
        assert isinstance(min_pri, float)
        assert isinstance(max_pri, float)
        assert (min_pri, max_pri) == (1.0, 10.0)

    def test_invalid_source_type(self):
        """Should return default range for unknown source types."""
        result = get_priority_range("invalid")
        assert result == (0.0, 10.0)

    def test_negative_priorities(self):
        """Should handle negative priority values correctly."""
        df = pd.DataFrame({"priority": [-5.0, 0.0, 5.0]})
        result = get_priority_range(df)
        assert result == (-5.0, 5.0)

    def test_fractional_priorities(self):
        """Should handle fractional priority values correctly."""
        df = pd.DataFrame({"priority": [1.5, 2.7, 9.3]})
        result = get_priority_range(df)
        assert result == (1.5, 9.3)
