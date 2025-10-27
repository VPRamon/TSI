"""Unit tests for :mod:`core.preprocessing.SchedulePreprocessor`."""

from __future__ import annotations

from pathlib import Path

import pytest

from core.preprocessing import SchedulePreprocessor

pytestmark = pytest.mark.unit


@pytest.fixture
def preprocessor(tmp_path: Path) -> SchedulePreprocessor:
    """Provide a pre-loaded preprocessor instance for unit tests."""

    schedule_file = tmp_path / "schedule.json"
    schedule_file.write_text(
        """
        {
            "SchedulingBlock": [
                {
                    "schedulingBlockId": "sb-1",
                    "priority": 9.5,
                    "schedulingBlockConfiguration_": {
                        "constraints_": {
                            "timeConstraint_": {
                                "minObservationTimeInSec": 900,
                                "requestedDurationSec": 1200,
                                "fixedStartTime": [{"value": 60000.0}],
                                "fixedStopTime": [{"value": 60010.0}]
                            },
                            "elevationConstraint_": {
                                "minElevationAngleInDeg": 50.0,
                                "maxElevationAngleInDeg": 70.0
                            }
                        }
                    },
                    "target": {
                        "position_": {
                            "coord": {"celestial": {"raInDeg": 45.0, "decInDeg": -10.0}}
                        }
                    }
                }
            ]
        }
        """.strip()
    )
    processor = SchedulePreprocessor(schedule_file)
    processor.load_data()
    return processor


def test_load_data__with_missing_file__raises_file_not_found(tmp_path: Path) -> None:
    """Loading from a missing path should raise a ``FileNotFoundError``."""

    # Given: a non-existent schedule path
    missing = tmp_path / "does_not_exist.json"
    processor = SchedulePreprocessor(missing)

    # When / Then: attempting to load data fails
    with pytest.raises(FileNotFoundError):
        processor.load_data()


def test_extract_dataframe__before_load__raises_value_error(tmp_path: Path) -> None:
    """Calling ``extract_dataframe`` without a loaded schedule should fail."""

    # Given: a processor that has not loaded data yet
    schedule_file = tmp_path / "schedule.json"
    schedule_file.write_text('{"SchedulingBlock": []}')
    processor = SchedulePreprocessor(schedule_file)

    # When / Then: extracting the dataframe raises a ValueError
    with pytest.raises(ValueError):
        processor.extract_dataframe()


def test_enrich_with_visibility__without_dataframe__raises_value_error(
    preprocessor: SchedulePreprocessor,
) -> None:
    """Visibility enrichment requires the base dataframe to exist."""

    # Given: a preprocessor with data loaded but no extracted dataframe
    # When / Then: enriching without dataframe should fail
    with pytest.raises(ValueError):
        preprocessor.enrich_with_visibility()


def test_add_derived_columns__without_dataframe__raises_value_error(
    preprocessor: SchedulePreprocessor,
) -> None:
    """Derived column computation expects a prepared dataframe."""

    # Given / When / Then: calling the helper early raises ``ValueError``
    with pytest.raises(ValueError):
        preprocessor.add_derived_columns()


def test_enrich_with_visibility__without_external_data__defaults_to_zero(
    preprocessor: SchedulePreprocessor,
) -> None:
    """Default enrichment should inject zero visibility statistics."""

    # Given: a raw dataframe without visibility records
    df = preprocessor.extract_dataframe()

    # When: enriching without an external visibility file
    enriched = preprocessor.enrich_with_visibility()

    # Then: counts should default to zero and the same object is returned for chaining
    assert (enriched["num_visibility_periods"] == 0).all()
    assert (enriched["total_visibility_hours"] == 0).all()
    assert enriched is df


def test_assign_priority_bin__across_ranges__covers_all_categories(
    preprocessor: SchedulePreprocessor,
) -> None:
    """Priority binning helper should cover every edge case."""

    # Given / When: evaluating representative priority values
    bins = [
        preprocessor._assign_priority_bin(value) for value in [None, -1, 0, 8.5, 10.5, 13.2, 16.7]
    ]

    # Then: bins should match the documented labels
    assert bins == [
        "Unknown",
        "Invalid (<0)",
        "Low (0-8)",
        "Medium (8-10)",
        "High (10-12)",
        "Very High (12-15)",
        "Critical (>15)",
    ]


def test_validate__with_corrupted_rows__returns_multiple_errors(
    preprocessor: SchedulePreprocessor,
) -> None:
    """Validation should surface quality issues in the dataframe."""

    # Given: a dataframe enriched with default visibility
    preprocessor.extract_dataframe()
    preprocessor.enrich_with_visibility()
    df = preprocessor.add_derived_columns()

    # When: introducing a variety of invalid values
    df.loc[0, "priority"] = -1
    df.loc[0, "raInDeg"] = 999
    df.loc[0, "decInDeg"] = -100
    df.loc[0, "requestedDurationSec"] = 0
    df.loc[0, "minElevationAngleInDeg"] = 80
    df.loc[0, "maxElevationAngleInDeg"] = 40
    df.loc[0, "scheduled_period.start"] = 10
    df.loc[0, "scheduled_period.stop"] = 5

    # Then: validation should report multiple issues
    result = preprocessor.validate()
    assert not result.is_valid
    assert len(result.errors) >= 4
    assert any("negative priority" in msg for msg in result.errors)
