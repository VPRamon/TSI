"""Integration tests for the schedule preprocessing pipeline."""

from __future__ import annotations

import json
from pathlib import Path

import pandas as pd
import pytest

from core.preprocessing import SchedulePreprocessor, preprocess_iteration, preprocess_schedule

pytestmark = pytest.mark.integration


@pytest.fixture
def sample_schedule_data() -> dict:
    """Representative subset of a scheduling export."""

    return {
        "SchedulingBlock": [
            {
                "schedulingBlockId": "1000001",
                "priority": 8.5,
                "schedulingBlockConfiguration_": {
                    "constraints_": {
                        "timeConstraint_": {
                            "minObservationTimeInSec": 1_200,
                            "requestedDurationSec": 1_800,
                            "fixedStartTime": [],
                            "fixedStopTime": [],
                        },
                        "azimuthConstraint_": {
                            "minAzimuthAngleInDeg": 0.0,
                            "maxAzimuthAngleInDeg": 360.0,
                        },
                        "elevationConstraint_": {
                            "minElevationAngleInDeg": 60.0,
                            "maxElevationAngleInDeg": 90.0,
                        },
                    }
                },
                "target": {
                    "position_": {
                        "coord": {
                            "celestial": {
                                "raInDeg": 123.45,
                                "decInDeg": 45.67,
                            }
                        }
                    }
                },
                "schedulingBlock": {
                    "startTime": {"value": 61894.5, "format": "MJD", "scale": "UTC"},
                    "stopTime": {"value": 61894.6, "format": "MJD", "scale": "UTC"},
                },
            },
            {
                "schedulingBlockId": "1000002",
                "priority": 5.0,
                "schedulingBlockConfiguration_": {
                    "constraints_": {
                        "timeConstraint_": {
                            "minObservationTimeInSec": 600,
                            "requestedDurationSec": 900,
                            "fixedStartTime": [],
                            "fixedStopTime": [],
                        },
                        "azimuthConstraint_": {
                            "minAzimuthAngleInDeg": 0.0,
                            "maxAzimuthAngleInDeg": 360.0,
                        },
                        "elevationConstraint_": {
                            "minElevationAngleInDeg": 50.0,
                            "maxElevationAngleInDeg": 85.0,
                        },
                    }
                },
                "target": {
                    "position_": {
                        "coord": {
                            "celestial": {
                                "raInDeg": 234.56,
                                "decInDeg": -23.45,
                            }
                        }
                    }
                },
            },
        ]
    }


@pytest.fixture
def sample_visibility_data() -> dict:
    """Visibility windows keyed by scheduling block identifier."""

    return {
        "SchedulingBlock": {
            "1000001": [
                {"startTime": {"value": 61894.4}, "stopTime": {"value": 61894.5}},
                {"startTime": {"value": 61895.4}, "stopTime": {"value": 61895.6}},
            ],
            "1000002": [
                {"startTime": {"value": 61894.3}, "stopTime": {"value": 61894.7}},
            ],
        }
    }


def _write_json(path: Path, payload: dict) -> None:
    path.write_text(json.dumps(payload))


def test_preprocess_schedule__with_visibility__returns_enriched_dataframe(
    tmp_path: Path,
    sample_schedule_data: dict,
    sample_visibility_data: dict,
) -> None:
    """The high-level preprocess helper should produce derived columns."""

    # Given: schedule and visibility JSON files
    schedule_file = tmp_path / "schedule.json"
    visibility_file = tmp_path / "visibility.json"
    _write_json(schedule_file, sample_schedule_data)
    _write_json(visibility_file, sample_visibility_data)

    # When: running the preprocessing pipeline
    result = preprocess_schedule(schedule_file, visibility_file)

    # Then: the dataframe should include derived metrics
    assert isinstance(result.dataframe, pd.DataFrame)
    assert {"requested_hours", "total_visibility_hours"}.issubset(result.dataframe.columns)
    assert result.metadata.total_blocks == 2


def test_preprocess_iteration__writes_outputs(tmp_path: Path, sample_schedule_data: dict) -> None:
    """The CLI-style iteration helper should materialize output files."""

    # Given: raw schedule JSON
    schedule_file = tmp_path / "schedule.json"
    _write_json(schedule_file, sample_schedule_data)

    output_dir = tmp_path / "outputs"

    # When: preprocessing the iteration without visibility data
    preprocess_iteration(schedule_file, None, output_dir)

    # Then: parquet and CSV artefacts should be created
    parquet_file = output_dir / "schedule_preprocessed.parquet"
    csv_file = output_dir / "schedule_preprocessed.csv"
    assert parquet_file.exists()
    assert csv_file.exists()


def test_schedule_preprocessor__with_manual_steps__produces_valid_dataframe(
    tmp_path: Path,
    sample_schedule_data: dict,
    sample_visibility_data: dict,
) -> None:
    """Manual orchestration of the preprocessor should mirror the helper outputs."""

    # Given: schedule and visibility files written to disk
    schedule_file = tmp_path / "schedule.json"
    visibility_file = tmp_path / "visibility.json"
    _write_json(schedule_file, sample_schedule_data)
    _write_json(visibility_file, sample_visibility_data)

    processor = SchedulePreprocessor(schedule_file)

    # When: executing the preprocessing steps manually
    processor.load_data()
    df = processor.extract_dataframe()
    processor.enrich_with_visibility(visibility_file)
    processed = processor.add_derived_columns()

    # Then: the resulting dataframe should include expected columns and types
    assert df is processed
    required_columns = {
        "schedulingBlockId",
        "priority",
        "requested_hours",
        "total_visibility_hours",
    }
    assert required_columns.issubset(processed.columns)
