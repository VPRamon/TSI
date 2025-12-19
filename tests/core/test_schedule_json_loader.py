"""Tests for loading schedule data from JSON sources.

⚠️ LEGACY TESTS - Uses deprecated core.loaders API

These tests reference the old core.loaders module which has been replaced by
the Rust backend (tsi_rust_api.TSIBackend). Tests are disabled until they are
migrated to the new API.
"""

from __future__ import annotations

from pathlib import Path

import pytest

# LEGACY: core.loaders no longer exists - use tsi_rust_api.TSIBackend instead
# from core.loaders import load_schedule_from_json

PROJECT_ROOT = Path(__file__).resolve().parents[2]


@pytest.fixture(scope="module")
def schedule_paths() -> tuple[Path, Path | None]:
    """Provide paths to the JSON schedule fixtures bundled with the repo."""

    schedule_path = PROJECT_ROOT / "data" / "schedule.json"
    visibility_path = PROJECT_ROOT / "data" / "possible_periods.json"

    if not schedule_path.exists():
        pytest.skip("Repository fixture data/schedule.json is not available.")

    return schedule_path, visibility_path if visibility_path.exists() else None


@pytest.mark.skip("Legacy core.loaders module removed - needs migration to tsi_rust_api")
def test_load_schedule_from_json_provides_expected_schema(
    schedule_paths: tuple[Path, Path | None],
) -> None:
    """Ensure the JSON loader returns data with the expected columns and validation."""
    pytest.skip("Test disabled - core.loaders replaced by Rust backend")


@pytest.mark.skip("Legacy core.loaders module removed - needs migration to tsi_rust_api")
def test_load_schedule_from_json_without_visibility(
    schedule_paths: tuple[Path, Path | None],
) -> None:
    """Verify that loading without a visibility file still returns consistent results."""
    pytest.skip("Test disabled - core.loaders replaced by Rust backend")
