"""
Test suite for tsi_rust_api module - the consolidated Python interface to Rust backend.

This test suite validates:
1. TSIBackend class functionality
2. Direct functional API (load_schedule, filter_by_priority, etc.)
3. Analytics functions (get_top_observations, find_conflicts)
4. Data loading and transformation
5. Integration with services layer
"""

from __future__ import annotations

import json
from pathlib import Path

import pandas as pd
import pytest

import tsi_rust_api
from tsi_rust_api import (
    TSIBackend,
    filter_by_priority,
    find_conflicts,
    get_top_observations,
    load_dark_periods,
    load_schedule,
    load_schedule_file,
    load_schedule_from_string,
)


class TestTSIBackendClass:
    """Test TSIBackend class interface."""

    def test_backend_instantiation(self):
        """Test that TSIBackend can be instantiated."""
        backend = TSIBackend()
        assert backend is not None
        assert backend.use_pandas is True
        assert repr(backend) == "TSIBackend(use_pandas=True)"

    def test_backend_load_schedule_file(self, tmp_path: Path):
        """Test loading schedule from file via TSIBackend."""
        # Create a test JSON file
        test_data = {
            "schedulingBlocks": [
                {"schedulingBlockId": "1", "priority": 8.5},
                {"schedulingBlockId": "2", "priority": 6.2},
            ]
        }
        test_file = tmp_path / "test_schedule.json"
        test_file.write_text(json.dumps(test_data))

        backend = TSIBackend()
        df = backend.load_schedule(test_file)

        assert isinstance(df, pd.DataFrame)
        assert len(df) == 2
        assert "schedulingBlockId" in df.columns
        assert "priority" in df.columns

    def test_backend_load_schedule_from_string(self):
        """Test loading schedule from JSON string via TSIBackend."""
        json_data = json.dumps(
            {
                "schedulingBlocks": [
                    {"schedulingBlockId": "1", "priority": 8.5},
                    {"schedulingBlockId": "2", "priority": 6.2},
                ]
            }
        )

        backend = TSIBackend()
        df = backend.load_schedule_from_string(json_data, format="json")

        assert isinstance(df, pd.DataFrame)
        assert len(df) == 2

    def test_backend_filter_by_priority(self):
        """Test priority filtering via TSIBackend."""
        test_data = pd.DataFrame(
            {
                "schedulingBlockId": ["1", "2", "3"],
                "priority": [8.5, 6.2, 9.1],
            }
        )

        backend = TSIBackend()
        high_priority = backend.filter_by_priority(test_data, min_priority=8.0)

        assert isinstance(high_priority, pd.DataFrame)
        assert len(high_priority) == 2
        assert all(high_priority["priority"] >= 8.0)

    def test_backend_get_top_observations(self):
        """Test get_top_observations via TSIBackend."""
        test_data = pd.DataFrame(
            {
                "schedulingBlockId": ["1", "2", "3", "4", "5"],
                "priority": [8.5, 6.2, 9.1, 7.3, 8.9],
            }
        )

        backend = TSIBackend()
        top_3 = backend.get_top_observations(test_data, n=3, by="priority")

        assert isinstance(top_3, pd.DataFrame)
        assert len(top_3) == 3
        # Should be sorted in descending order
        assert top_3.iloc[0]["priority"] == 9.1
        assert top_3.iloc[1]["priority"] == 8.9
        assert top_3.iloc[2]["priority"] == 8.5

    def test_backend_find_conflicts(self):
        """Test find_conflicts via TSIBackend."""
        test_data = pd.DataFrame(
            {
                "schedulingBlockId": ["1", "2"],
                "priority": [8.5, 6.2],
                "scheduled_start": [100.0, 150.0],
                "scheduled_stop": [200.0, 250.0],
            }
        )

        backend = TSIBackend()
        conflicts = backend.find_conflicts(test_data)

        # Should return empty DataFrame (no conflicts in test data)
        assert isinstance(conflicts, pd.DataFrame)


class TestFunctionalAPI:
    """Test functional API (non-class methods)."""

    def test_load_schedule_function(self, tmp_path: Path):
        """Test load_schedule convenience function."""
        test_data = {
            "schedulingBlocks": [
                {"schedulingBlockId": "1", "priority": 8.5},
            ]
        }
        test_file = tmp_path / "test.json"
        test_file.write_text(json.dumps(test_data))

        df = load_schedule(test_file)

        assert isinstance(df, pd.DataFrame)
        assert len(df) == 1

    def test_load_schedule_file_function(self, tmp_path: Path):
        """Test load_schedule_file function."""
        test_data = {
            "schedulingBlocks": [
                {"schedulingBlockId": "1", "priority": 8.5},
            ]
        }
        test_file = tmp_path / "test.json"
        test_file.write_text(json.dumps(test_data))

        df = load_schedule_file(test_file, format="auto")

        assert isinstance(df, pd.DataFrame)
        assert len(df) == 1

    def test_load_schedule_from_string_function(self):
        """Test load_schedule_from_string function."""
        json_data = json.dumps({"schedulingBlocks": [{"schedulingBlockId": "1", "priority": 8.5}]})

        df = load_schedule_from_string(json_data, format="json")

        assert isinstance(df, pd.DataFrame)
        assert len(df) == 1

    def test_filter_by_priority_function(self):
        """Test filter_by_priority convenience function."""
        test_data = pd.DataFrame(
            {
                "schedulingBlockId": ["1", "2", "3"],
                "priority": [8.5, 6.2, 9.1],
            }
        )

        high_priority = filter_by_priority(test_data, min_priority=8.0)

        assert isinstance(high_priority, pd.DataFrame)
        assert len(high_priority) == 2

    def test_get_top_observations_function(self):
        """Test get_top_observations function."""
        test_data = pd.DataFrame(
            {
                "schedulingBlockId": ["1", "2", "3"],
                "priority": [8.5, 6.2, 9.1],
            }
        )

        top_2 = get_top_observations(test_data, n=2, by="priority")

        assert isinstance(top_2, pd.DataFrame)
        assert len(top_2) == 2

    def test_find_conflicts_function(self):
        """Test find_conflicts function."""
        test_data = pd.DataFrame(
            {
                "schedulingBlockId": ["1", "2"],
                "priority": [8.5, 6.2],
            }
        )

        conflicts = find_conflicts(test_data)

        assert isinstance(conflicts, pd.DataFrame)

    def test_load_dark_periods_function(self, tmp_path: Path):
        """Test load_dark_periods function."""
        test_data = {
            "dark_periods": [
                {
                    "start_mjd": 59000.0,
                    "stop_mjd": 59100.0,
                    "duration_hours": 2400.0,
                }
            ]
        }
        test_file = tmp_path / "dark_periods.json"
        test_file.write_text(json.dumps(test_data))

        df = load_dark_periods(test_file)

        assert isinstance(df, pd.DataFrame)
        assert len(df) == 1
        assert "start_mjd" in df.columns


class TestDataLoading:
    """Test various data loading scenarios."""

    def test_load_json_with_schedulingBlocks_key(self, tmp_path: Path):
        """Test loading JSON with 'schedulingBlocks' key."""
        test_data = {
            "schedulingBlocks": [
                {"schedulingBlockId": "1", "priority": 8.5},
                {"schedulingBlockId": "2", "priority": 6.2},
            ]
        }
        test_file = tmp_path / "test.json"
        test_file.write_text(json.dumps(test_data))

        df = load_schedule_file(test_file)

        assert len(df) == 2

    def test_load_json_with_SchedulingBlock_key(self, tmp_path: Path):
        """Test loading JSON with 'SchedulingBlock' key."""
        test_data = {
            "SchedulingBlock": [
                {"schedulingBlockId": "1", "priority": 8.5},
            ]
        }
        test_file = tmp_path / "test.json"
        test_file.write_text(json.dumps(test_data))

        df = load_schedule_file(test_file)

        assert len(df) == 1

    def test_load_json_list_directly(self, tmp_path: Path):
        """Test loading JSON that is a direct list."""
        test_data = [
            {"schedulingBlockId": "1", "priority": 8.5},
        ]
        test_file = tmp_path / "test.json"
        test_file.write_text(json.dumps(test_data))

        df = load_schedule_file(test_file)

        assert len(df) == 1

    def test_invalid_format_raises_error(self, tmp_path: Path):
        """Test that invalid format raises error."""
        test_file = tmp_path / "test.csv"
        test_file.write_text("col1,col2\n1,2\n")

        with pytest.raises(ValueError, match="Only JSON files are supported"):
            load_schedule_file(test_file, format="auto")


class TestAPIExports:
    """Test that the module exports the expected API."""

    def test_module_has_expected_exports(self):
        """Test that tsi_rust_api exports the expected symbols."""
        expected_exports = [
            "TSIBackend",
            "load_schedule",
            "load_schedule_file",
            "load_schedule_from_string",
            "load_dark_periods",
            "filter_by_priority",
            "get_top_observations",
            "find_conflicts",
        ]

        for export in expected_exports:
            assert hasattr(tsi_rust_api, export), f"Missing export: {export}"

    def test_module_version(self):
        """Test that module has version info."""
        assert hasattr(tsi_rust_api, "__version__")
        assert isinstance(tsi_rust_api.__version__, str)


class TestServicesIntegration:
    """Test integration with services layer."""

    def test_services_can_import_backend(self):
        """Test that services can import BACKEND."""
        from tsi.services import BACKEND

        assert BACKEND is not None
        assert isinstance(BACKEND, TSIBackend)

    def test_services_backend_is_tsi_rust_api_backend(self):
        """Test that services BACKEND is a TSIBackend instance."""
        from tsi.services.rust_backend import BACKEND

        assert type(BACKEND).__name__ == "TSIBackend"
        assert type(BACKEND).__module__ == "tsi_rust_api"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
