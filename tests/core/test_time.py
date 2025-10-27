"""Unit tests for :mod:`core.time` utilities."""

import math
from datetime import datetime, timezone

import pandas as pd
import pytest

from core.time import (
    datetime_to_mjd,
    format_datetime_utc,
    get_time_range,
    mjd_to_datetime,
    parse_optional_mjd,
    parse_visibility_periods,
)

pytestmark = pytest.mark.unit


def test_mjd_to_datetime__with_unix_epoch__returns_epoch_timestamp() -> None:
    """MJD for the Unix epoch should map to the expected timestamp."""

    # Given: the Unix epoch MJD value
    mjd_value = 40587.0

    # When: converting to pandas timestamp
    result = mjd_to_datetime(mjd_value)

    # Then: the conversion should be exact
    expected = pd.Timestamp("1970-01-01 00:00:00", tz="UTC")
    assert result == expected


def test_mjd_to_datetime__with_known_date__returns_precise_timestamp() -> None:
    """Validate conversion against a known calendar date."""

    # Given: a mid-2020 MJD value
    mjd_value = 59000.5

    # When: converting the value back to datetime
    result = mjd_to_datetime(mjd_value)

    # Then: the timestamp should match the documented expectation
    expected = pd.Timestamp("2020-05-31 12:00:00", tz="UTC")
    assert result == expected


def test_datetime_to_mjd__with_timezone_aware_datetime__returns_expected_mjd() -> None:
    """Ensure timezone-aware datetimes convert accurately to MJD."""

    # Given: a UTC datetime
    dt = datetime(2020, 5, 31, 12, 0, 0, tzinfo=timezone.utc)

    # When: converting to MJD
    result = datetime_to_mjd(dt)

    # Then: the result should match the known MJD
    assert abs(result - 59000.5) < 1e-6


def test_datetime_to_mjd__with_naive_datetime__raises_value_error() -> None:
    """Naive datetimes should be rejected to prevent timezone bugs."""

    # Given: a naive datetime lacking timezone information
    dt = datetime(2020, 1, 1, 0, 0, 0)

    # When / Then: conversion should raise a helpful error
    with pytest.raises(ValueError, match="timezone-aware"):
        datetime_to_mjd(dt)


def test_datetime_mjd_round_trip__with_fractional_component__preserves_precision() -> None:
    """Converting from MJD to datetime and back preserves fractional parts."""

    # Given: an MJD value with fractional day component
    original_mjd = 59000.123456

    # When: performing round-trip conversion
    result_mjd = datetime_to_mjd(mjd_to_datetime(original_mjd).to_pydatetime())

    # Then: precision should be retained
    assert abs(result_mjd - original_mjd) < 1e-6


def test_parse_visibility_periods__with_valid_string__returns_timestamp_pairs() -> None:
    """Visibility strings should become timestamp tuples."""

    # Given: a valid serialized list of periods
    visibility_str = "[(61892.1997, 61892.2108), (61893.1970, 61893.2100)]"

    # When: parsing the string into timestamps
    result = parse_visibility_periods(visibility_str)

    # Then: both periods should be converted correctly
    assert len(result) == 2
    assert all(isinstance(period, tuple) and len(period) == 2 for period in result)
    assert all(isinstance(dt, pd.Timestamp) for period in result for dt in period)


def test_parse_visibility_periods__with_empty_string__returns_empty_list() -> None:
    """Empty strings should return an empty list."""

    # Given: an empty serialized value
    visibility_str = ""

    # When: parsing the value
    result = parse_visibility_periods(visibility_str)

    # Then: the parser should gracefully return an empty list
    assert result == []


def test_parse_visibility_periods__with_malformed_entries__skips_invalid_items() -> None:
    """Invalid entries should be ignored when possible."""

    # Given: a list with one malformed entry
    visibility_str = "[(61892.1997, 61892.2108), 'invalid', (61893.1970, 61893.2100)]"

    # When: parsing the list
    result = parse_visibility_periods(visibility_str)

    # Then: only valid entries should be returned
    assert len(result) == 2


def test_parse_optional_mjd__with_valid_value__returns_timestamp() -> None:
    """Optional MJD parsing should handle numeric input."""

    # Given: a valid numeric MJD
    mjd_value = 40587.0

    # When: parsing the optional value
    result = parse_optional_mjd(mjd_value)

    # Then: it should convert to the expected timestamp
    expected = pd.Timestamp("1970-01-01 00:00:00", tz="UTC")
    assert result == expected


def test_parse_optional_mjd__with_none__returns_none() -> None:
    """None input should remain None."""

    # Given: a missing value
    # When: parsing the optional value
    result = parse_optional_mjd(None)

    # Then: the parser should return None
    assert result is None


def test_parse_optional_mjd__with_nan__returns_none() -> None:
    """NaN values should be treated as missing."""

    # Given: a floating NaN value
    # When: parsing the optional value
    result = parse_optional_mjd(math.nan)

    # Then: the parser should treat it as missing
    assert result is None


def test_get_time_range__with_periods__returns_min_and_max() -> None:
    """Time range extraction should use the earliest start and latest stop."""

    # Given: two visibility periods
    dt1_start = pd.Timestamp("2020-01-01 00:00:00", tz="UTC")
    dt1_stop = pd.Timestamp("2020-01-01 12:00:00", tz="UTC")
    dt2_start = pd.Timestamp("2020-01-02 00:00:00", tz="UTC")
    dt2_stop = pd.Timestamp("2020-01-02 12:00:00", tz="UTC")
    periods = [(dt1_start, dt1_stop), (dt2_start, dt2_stop)]

    # When: calculating the range
    start, stop = get_time_range(periods)

    # Then: boundaries should correspond to earliest and latest timestamps
    assert start == dt1_start
    assert stop == dt2_stop


def test_get_time_range__with_empty_periods__returns_none_tuple() -> None:
    """Empty lists should produce ``(None, None)``."""

    # Given: an empty list of periods
    # When: computing the range
    start, stop = get_time_range([])

    # Then: both start and stop should be None
    assert start is None
    assert stop is None


def test_format_datetime_utc__with_timestamp__returns_formatted_string() -> None:
    """Formatting helper should render UTC suffix."""

    # Given: a timezone-aware timestamp
    dt = pd.Timestamp("2020-07-17 12:30:45", tz="UTC")

    # When: formatting the timestamp
    result = format_datetime_utc(dt)

    # Then: the resulting string should include the UTC marker
    assert result == "2020-07-17 12:30:45 UTC"
