"""Comprehensive unit tests for schedule loading.

⚠️ LEGACY TESTS - Uses deprecated core.loaders API

These tests reference the old core.loaders.schedule_loader module which has been
replaced by the Rust backend (tsi_rust_api.TSIBackend). Tests are disabled until
they are migrated to the new API.
"""

from __future__ import annotations

import io
import json
from pathlib import Path
from types import SimpleNamespace

import pandas as pd
import polars as pl
import pytest

# LEGACY: core.loaders.schedule_loader no longer exists - use tsi_rust_api.TSIBackend
# from core.loaders.schedule_loader import (
#     ScheduleLoadResult,
#     ValidationResult,
#     load_schedule_from_csv,
#     load_schedule_from_iteration,
#     load_schedule_from_json,
# )

pytestmark = [pytest.mark.unit, pytest.mark.skip("Legacy core.loaders replaced by Rust backend")]


@pytest.fixture
def minimal_schedule_data() -> dict:
    """Minimal valid schedule JSON structure."""
    return {
        "SchedulingBlock": [
            {
                "id": "SB001",
                "priority": 5.0,
                "requestedDurationSec": 3600.0,
                "celestialCoordinates": {"ra": 123.45, "dec": -23.45},
                "scheduled_period": {"start": None, "stop": None},
            }
        ]
    }


@pytest.fixture
def malformed_schedule_data() -> dict:
    """Schedule with missing required fields."""
    return {"SchedulingBlock": [{"id": "SB001"}]}


@pytest.fixture
def schedule_with_visibility() -> tuple[dict, dict]:
    """Schedule and visibility data."""
    schedule = {
        "SchedulingBlock": [
            {
                "id": "SB001",
                "priority": 5.0,
                "requestedDurationSec": 3600.0,
                "celestialCoordinates": {"ra": 123.45, "dec": -23.45},
                "scheduled_period": {"start": None, "stop": None},
            }
        ]
    }
    visibility = {"SB001": [["2024-01-01T00:00:00Z", "2024-01-01T12:00:00Z"]]}
    return schedule, visibility


@pytest.fixture
def valid_csv_content() -> str:
    """Valid CSV schedule content."""
    return """schedulingBlockId,priority,requestedDurationSec,scheduled_period.start,scheduled_period.stop,scheduled_flag
SB001,5.0,3600.0,,,False
SB002,8.0,7200.0,2024-01-01T00:00:00Z,2024-01-01T02:00:00Z,True
"""


@pytest.fixture
def csv_with_visibility_string() -> str:
    """CSV with visibility column as string representation."""
    return """schedulingBlockId,priority,requestedDurationSec,scheduled_period.start,scheduled_period.stop,scheduled_flag,visibility
SB001,5.0,3600.0,,,False,"[['2024-01-01T00:00:00Z', '2024-01-01T12:00:00Z']]"
SB002,8.0,7200.0,2024-01-01T00:00:00Z,2024-01-01T02:00:00Z,True,[]
"""


@pytest.fixture
def csv_missing_required_columns() -> str:
    """CSV missing required columns."""
    return """schedulingBlockId,priority
SB001,5.0
"""


@pytest.fixture(autouse=True)
def stub_rust_backend(monkeypatch: pytest.MonkeyPatch) -> None:
    """Stub the tsi_rust backend so tests can run without compiled Rust."""

    from core.loaders import schedule_loader

    class DummyRustValidation:
        def __init__(
            self,
            is_valid: bool = True,
            errors: list[str] | None = None,
            warnings: list[str] | None = None,
            stats: dict[str, int] | None = None,
        ) -> None:
            self.is_valid = is_valid
            self.errors = errors or []
            self.warnings = warnings or []
            self._stats = stats or {}

        def get_stats(self) -> dict[str, int]:
            return self._stats

    def _load_blocks(schedule_path: str | Path) -> list[dict]:
        with open(schedule_path) as handle:
            payload = json.load(handle)
        return payload.get("SchedulingBlock", [])

    def fake_py_preprocess_schedule(
        schedule_path: str | None,
        visibility_path: str | None,
        validate: bool,
    ) -> tuple[pl.DataFrame, DummyRustValidation]:
        """Emulate the Rust preprocessing output with lightweight pandas logic."""

        blocks = _load_blocks(schedule_path) if schedule_path else []
        block_ids = [
            block.get("schedulingBlockId")
            or block.get("id")
            or f"sb-{idx}"
            for idx, block in enumerate(blocks)
        ]
        scheduled_flags = [
            bool((block.get("scheduled_period") or {}).get("start")) for block in blocks
        ]
        df_pd = pd.DataFrame(
            {
                "schedulingBlockId": block_ids,
                "priority": [block.get("priority", 0.0) for block in blocks],
                "scheduled_flag": scheduled_flags,
                "visibility": [[] for _ in blocks],
            }
        )
        df_polars = pl.from_pandas(df_pd)
        scheduled_count = sum(1 for flag in scheduled_flags if flag)
        stats = {
            "total_blocks": len(blocks),
            "scheduled_blocks": scheduled_count,
            "unscheduled_blocks": max(len(blocks) - scheduled_count, 0),
        }
        return df_polars, DummyRustValidation(stats=stats)

    fake_module = SimpleNamespace(py_preprocess_schedule=fake_py_preprocess_schedule)
    monkeypatch.setattr(schedule_loader, "tsi_rust", fake_module, raising=False)


class TestLoadScheduleFromJson:
    """Test load_schedule_from_json function."""

    def test_with_dict_input__loads_successfully(self, minimal_schedule_data: dict) -> None:
        """Load from parsed JSON dict."""
        result = load_schedule_from_json(minimal_schedule_data)
        assert isinstance(result, ScheduleLoadResult)
        assert isinstance(result.dataframe, pd.DataFrame)
        assert result.source_type == "json"
        assert len(result.dataframe) > 0

    def test_with_file_path__loads_successfully(self, tmp_path: Path) -> None:
        """Load from file path."""
        schedule_file = tmp_path / "schedule.json"
        schedule_data = {
            "SchedulingBlock": [
                {
                    "id": "SB001",
                    "priority": 5.0,
                    "requestedDurationSec": 3600.0,
                    "celestialCoordinates": {"ra": 123.45, "dec": -23.45},
                    "scheduled_period": {"start": None, "stop": None},
                }
            ]
        }
        schedule_file.write_text(json.dumps(schedule_data))

        result = load_schedule_from_json(schedule_file)
        assert result.source_path == str(schedule_file)
        assert len(result.dataframe) > 0

    def test_with_nonexistent_file__raises_file_not_found(self, tmp_path: Path) -> None:
        """Raise FileNotFoundError for missing file."""
        nonexistent = tmp_path / "nonexistent.json"
        with pytest.raises(FileNotFoundError, match="Schedule file not found"):
            load_schedule_from_json(nonexistent)

    def test_with_file_like_object__loads_successfully(self, minimal_schedule_data: dict) -> None:
        """Load from file-like object (StringIO)."""
        file_obj = io.StringIO(json.dumps(minimal_schedule_data))
        result = load_schedule_from_json(file_obj)
        assert isinstance(result.dataframe, pd.DataFrame)
        # File pointer should be reset
        assert file_obj.tell() == 0

    def test_with_bytes_file_like__loads_successfully(self, minimal_schedule_data: dict) -> None:
        """Load from bytes file-like object (BytesIO)."""
        file_obj = io.BytesIO(json.dumps(minimal_schedule_data).encode("utf-8"))
        result = load_schedule_from_json(file_obj)
        assert isinstance(result.dataframe, pd.DataFrame)

    def test_with_visibility_dict__enriches_dataframe(
        self, schedule_with_visibility: tuple[dict, dict]
    ) -> None:
        """Enrich dataframe with visibility data."""
        schedule, visibility = schedule_with_visibility
        result = load_schedule_from_json(schedule, visibility)
        # Check that visibility processing occurred
        assert isinstance(result.dataframe, pd.DataFrame)

    def test_with_visibility_file_path__enriches_dataframe(
        self, schedule_with_visibility: tuple[dict, dict], tmp_path: Path
    ) -> None:
        """Enrich with visibility from file path."""
        schedule, visibility = schedule_with_visibility
        schedule_file = tmp_path / "schedule.json"
        visibility_file = tmp_path / "visibility.json"
        schedule_file.write_text(json.dumps(schedule))
        visibility_file.write_text(json.dumps(visibility))

        result = load_schedule_from_json(schedule_file, visibility_file)
        assert isinstance(result.dataframe, pd.DataFrame)

    def test_with_visibility_file_like__enriches_dataframe(
        self, schedule_with_visibility: tuple[dict, dict]
    ) -> None:
        """Enrich with visibility from file-like object."""
        schedule, visibility = schedule_with_visibility
        visibility_obj = io.StringIO(json.dumps(visibility))
        result = load_schedule_from_json(schedule, visibility_obj)
        assert isinstance(result.dataframe, pd.DataFrame)

    def test_with_invalid_json_type__raises_type_error(self) -> None:
        """Raise TypeError for unsupported input type."""
        with pytest.raises(TypeError, match="Unsupported schedule_json type"):
            load_schedule_from_json(12345)  # type: ignore[arg-type]

    def test_with_malformed_json__raises_json_error(self, tmp_path: Path) -> None:
        """Raise JSONDecodeError for malformed JSON."""
        malformed_file = tmp_path / "malformed.json"
        malformed_file.write_text("{invalid json")

        with pytest.raises(json.JSONDecodeError):
            load_schedule_from_json(malformed_file)

    def test_with_validate_false__skips_validation(self, minimal_schedule_data: dict) -> None:
        """Skip validation when validate=False."""
        result = load_schedule_from_json(minimal_schedule_data, validate=False)
        assert result.validation.is_valid is True
        assert result.validation.errors == []

    def test_with_validate_true__performs_validation(self, minimal_schedule_data: dict) -> None:
        """Perform validation when validate=True."""
        result = load_schedule_from_json(minimal_schedule_data, validate=True)
        assert isinstance(result.validation.is_valid, bool)

    def test_with_empty_scheduling_blocks__returns_empty_dataframe(self) -> None:
        """Handle empty SchedulingBlock list."""
        empty_data = {"SchedulingBlock": []}
        result = load_schedule_from_json(empty_data)
        assert len(result.dataframe) == 0

    def test_with_missing_celestial_coords__processes_successfully(self) -> None:
        """Handle missing celestial coordinates."""
        data = {
            "SchedulingBlock": [
                {
                    "id": "SB001",
                    "priority": 5.0,
                    "requestedDurationSec": 3600.0,
                    "scheduled_period": {"start": None, "stop": None},
                }
            ]
        }
        result = load_schedule_from_json(data, validate=False)
        assert len(result.dataframe) > 0

    def test_with_51910_5_sentinel__handles_gracefully(self) -> None:
        """Handle legacy 51910.5 sentinel value."""
        data = {
            "SchedulingBlock": [
                {
                    "id": "SB001",
                    "priority": 5.0,
                    "requestedDurationSec": 3600.0,
                    "celestialCoordinates": {"ra": 51910.5, "dec": -23.45},
                    "scheduled_period": {"start": None, "stop": None},
                }
            ]
        }
        result = load_schedule_from_json(data, validate=False)
        assert len(result.dataframe) > 0

    def test_with_fixed_times_as_dict__processes_successfully(self) -> None:
        """Handle fixed times as dict vs list."""
        data = {
            "SchedulingBlock": [
                {
                    "id": "SB001",
                    "priority": 5.0,
                    "requestedDurationSec": 3600.0,
                    "celestialCoordinates": {"ra": 123.45, "dec": -23.45},
                    "fixed_time": {"start": "2024-01-01T00:00:00Z", "stop": "2024-01-01T12:00:00Z"},
                    "scheduled_period": {"start": None, "stop": None},
                }
            ]
        }
        result = load_schedule_from_json(data, validate=False)
        assert len(result.dataframe) > 0


class TestLoadScheduleFromCsv:
    """Test load_schedule_from_csv function."""

    def test_with_valid_csv_file__loads_successfully(
        self, tmp_path: Path, valid_csv_content: str
    ) -> None:
        """Load from valid CSV file."""
        csv_file = tmp_path / "schedule.json"
        csv_file.write_text(valid_csv_content)

        result = load_schedule_from_csv(csv_file)
        assert isinstance(result.dataframe, pd.DataFrame)
        assert result.source_type == "csv"
        assert len(result.dataframe) == 2

    def test_with_file_like_object__loads_successfully(self, valid_csv_content: str) -> None:
        """Load from file-like object."""
        csv_obj = io.StringIO(valid_csv_content)
        result = load_schedule_from_csv(csv_obj)
        assert isinstance(result.dataframe, pd.DataFrame)
        # File pointer should be reset
        assert csv_obj.tell() == 0

    def test_with_nonexistent_file__raises_file_not_found(self, tmp_path: Path) -> None:
        """Raise FileNotFoundError for missing file."""
        nonexistent = tmp_path / "nonexistent.csv"
        with pytest.raises(FileNotFoundError, match="CSV file not found"):
            load_schedule_from_csv(nonexistent)

    def test_with_visibility_string__parses_visibility(
        self, tmp_path: Path, csv_with_visibility_string: str
    ) -> None:
        """Parse visibility column from string representation."""
        csv_file = tmp_path / "schedule.json"
        csv_file.write_text(csv_with_visibility_string)

        result = load_schedule_from_csv(csv_file)
        df = result.dataframe
        assert "visibility" in df.columns
        # First row should have parsed visibility list
        first_vis = df.iloc[0]["visibility"]
        assert isinstance(first_vis, list)

    def test_with_invalid_visibility_string__returns_empty_list(self, tmp_path: Path) -> None:
        """Handle invalid visibility string gracefully."""
        csv_content = """schedulingBlockId,priority,requestedDurationSec,scheduled_period.start,scheduled_period.stop,scheduled_flag,visibility
SB001,5.0,3600.0,,,False,"invalid{string"
"""
        csv_file = tmp_path / "schedule.json"
        csv_file.write_text(csv_content)

        result = load_schedule_from_csv(csv_file)
        df = result.dataframe
        # Should return empty list for invalid string
        assert df.iloc[0]["visibility"] == []

    def test_with_nan_visibility__returns_empty_list(self, tmp_path: Path) -> None:
        """Handle NaN visibility values."""
        csv_content = """schedulingBlockId,priority,requestedDurationSec,scheduled_period.start,scheduled_period.stop,scheduled_flag,visibility
SB001,5.0,3600.0,,,False,
"""
        csv_file = tmp_path / "schedule.json"
        csv_file.write_text(csv_content)

        result = load_schedule_from_csv(csv_file)
        df = result.dataframe
        assert df.iloc[0]["visibility"] == []

    def test_with_missing_required_columns__validation_fails(
        self, tmp_path: Path, csv_missing_required_columns: str
    ) -> None:
        """Validation fails when required columns missing."""
        csv_file = tmp_path / "schedule.json"
        csv_file.write_text(csv_missing_required_columns)

        result = load_schedule_from_csv(csv_file, validate=True)
        assert result.validation.is_valid is False
        assert any("Missing required columns" in err for err in result.validation.errors)

    def test_with_validate_false__skips_validation(
        self, tmp_path: Path, valid_csv_content: str
    ) -> None:
        """Skip validation when validate=False."""
        csv_file = tmp_path / "schedule.json"
        csv_file.write_text(valid_csv_content)

        result = load_schedule_from_csv(csv_file, validate=False)
        assert result.validation.is_valid is True

    def test_with_scheduled_flag__computes_stats(
        self, tmp_path: Path, valid_csv_content: str
    ) -> None:
        """Compute scheduled/unscheduled stats."""
        csv_file = tmp_path / "schedule.json"
        csv_file.write_text(valid_csv_content)

        result = load_schedule_from_csv(csv_file)
        stats = result.validation.stats
        assert "total_blocks" in stats
        assert "scheduled_blocks" in stats
        assert "unscheduled_blocks" in stats

    def test_with_invalid_csv_type__raises_type_error(self) -> None:
        """Raise TypeError for unsupported input type."""
        with pytest.raises(TypeError, match="Unsupported csv_path type"):
            load_schedule_from_csv(12345)  # type: ignore[arg-type]


class TestLoadScheduleFromIteration:
    """Test load_schedule_from_iteration function."""

    def test_with_valid_directory__loads_successfully(self, tmp_path: Path) -> None:
        """Load from valid data directory."""
        schedule_data = {
            "SchedulingBlock": [
                {
                    "id": "SB001",
                    "priority": 5.0,
                    "requestedDurationSec": 3600.0,
                    "celestialCoordinates": {"ra": 123.45, "dec": -23.45},
                    "scheduled_period": {"start": None, "stop": None},
                }
            ]
        }
        schedule_file = tmp_path / "schedule.json"
        schedule_file.write_text(json.dumps(schedule_data))

        result = load_schedule_from_iteration(tmp_path)
        assert result.source_type == "data_directory"
        assert result.source_path == str(tmp_path)
        assert len(result.dataframe) > 0

    def test_with_visibility_file__enriches_dataframe(self, tmp_path: Path) -> None:
        """Enrich with visibility from possible_periods.json."""
        schedule_data = {
            "SchedulingBlock": [
                {
                    "id": "SB001",
                    "priority": 5.0,
                    "requestedDurationSec": 3600.0,
                    "celestialCoordinates": {"ra": 123.45, "dec": -23.45},
                    "scheduled_period": {"start": None, "stop": None},
                }
            ]
        }
        visibility_data = {"SB001": [["2024-01-01T00:00:00Z", "2024-01-01T12:00:00Z"]]}

        schedule_file = tmp_path / "schedule.json"
        visibility_file = tmp_path / "possible_periods.json"
        schedule_file.write_text(json.dumps(schedule_data))
        visibility_file.write_text(json.dumps(visibility_data))

        result = load_schedule_from_iteration(tmp_path)
        assert len(result.dataframe) > 0

    def test_with_legacy_structure__loads_successfully(self, tmp_path: Path) -> None:
        """Load from legacy directory structure."""
        schedule_dir = tmp_path / "schedule"
        schedule_dir.mkdir()
        schedule_data = {
            "SchedulingBlock": [
                {
                    "id": "SB001",
                    "priority": 5.0,
                    "requestedDurationSec": 3600.0,
                    "celestialCoordinates": {"ra": 123.45, "dec": -23.45},
                    "scheduled_period": {"start": None, "stop": None},
                }
            ]
        }
        schedule_file = schedule_dir / "schedule.json"
        schedule_file.write_text(json.dumps(schedule_data))

        result = load_schedule_from_iteration(tmp_path)
        assert len(result.dataframe) > 0

    def test_with_legacy_visibility_structure__enriches_dataframe(self, tmp_path: Path) -> None:
        """Load visibility from legacy structure."""
        schedule_data = {
            "SchedulingBlock": [
                {
                    "id": "SB001",
                    "priority": 5.0,
                    "requestedDurationSec": 3600.0,
                    "celestialCoordinates": {"ra": 123.45, "dec": -23.45},
                    "scheduled_period": {"start": None, "stop": None},
                }
            ]
        }
        visibility_data = {"SB001": [["2024-01-01T00:00:00Z", "2024-01-01T12:00:00Z"]]}

        schedule_file = tmp_path / "schedule.json"
        visibility_dir = tmp_path / "possible periods"
        visibility_dir.mkdir()
        visibility_file = visibility_dir / "possible_periods.json"
        schedule_file.write_text(json.dumps(schedule_data))
        visibility_file.write_text(json.dumps(visibility_data))

        result = load_schedule_from_iteration(tmp_path)
        assert len(result.dataframe) > 0

    def test_with_nonexistent_directory__raises_file_not_found(self, tmp_path: Path) -> None:
        """Raise FileNotFoundError for missing directory."""
        nonexistent = tmp_path / "nonexistent"
        with pytest.raises(FileNotFoundError, match="Data directory not found"):
            load_schedule_from_iteration(nonexistent)

    def test_with_missing_schedule_json__raises_file_not_found(self, tmp_path: Path) -> None:
        """Raise FileNotFoundError when schedule.json missing."""
        # Create directory but no schedule.json
        with pytest.raises(FileNotFoundError, match="Schedule file not found"):
            load_schedule_from_iteration(tmp_path)


class TestScheduleLoadResult:
    """Test ScheduleLoadResult dataclass."""

    def test_with_all_fields__creates_successfully(self) -> None:
        """Create result with all fields."""
        df = pd.DataFrame({"col": [1, 2, 3]})
        validation = ValidationResult(is_valid=True, errors=[], warnings=[], stats={})
        result = ScheduleLoadResult(
            dataframe=df,
            validation=validation,
            source_type="json",
            source_path="/path/to/file.json",
        )
        assert result.dataframe is df
        assert result.validation is validation
        assert result.source_type == "json"
        assert result.source_path == "/path/to/file.json"

    def test_with_optional_source_path__defaults_to_none(self) -> None:
        """source_path defaults to None."""
        df = pd.DataFrame({"col": [1, 2, 3]})
        validation = ValidationResult(is_valid=True, errors=[], warnings=[], stats={})
        result = ScheduleLoadResult(dataframe=df, validation=validation, source_type="json")
        assert result.source_path is None
