"""Unit tests for :mod:`core.domain`."""

from __future__ import annotations

from datetime import datetime, timedelta

import pytest

from core.domain import Observation, Schedule, calculate_airmass, is_observable

pytestmark = pytest.mark.unit


def test_observation__with_invalid_dec__raises_value_error() -> None:
    """Ensure the observation dataclass validates declination bounds."""

    # Given: an observation declaration with invalid declination
    # When / Then: instantiating should raise a validation error
    with pytest.raises(ValueError):
        Observation("id", 400.0, 0.0, 1.0, 1.0)


def test_schedule__with_end_before_start__raises_value_error() -> None:
    """Schedules cannot finish before they start."""

    # Given: start time later than end time
    start = datetime.now()
    end = start - timedelta(hours=1)
    obs = Observation("id", 10.0, 0.0, 1.0, 1.0)

    # When / Then: constructing the schedule should fail
    with pytest.raises(ValueError):
        Schedule((obs,), start, end, 0.5)


def test_calculate_airmass__at_equator_and_meridian__returns_between_one_and_two() -> None:
    """Verify airmass stays within expected physical range."""

    # Given / When: computing airmass at equator and meridian
    value = calculate_airmass(ra_deg=0.0, dec_deg=0.0, lst_hours=0.0, latitude_deg=0.0)

    # Then: the resulting airmass should be valid
    assert 1.0 <= value < 2.0


def test_is_observable__with_varying_thresholds__matches_constraints() -> None:
    """Airmass threshold toggles visibility as expected."""

    # Given: a canonical observation
    obs = Observation("id", 0.0, 0.0, 1.0, 1.0)

    # When / Then: raising or lowering the threshold flips observability
    assert is_observable(obs, lst_hours=0.0, latitude_deg=0.0, max_airmass=2.0) is True
    assert is_observable(obs, lst_hours=0.0, latitude_deg=0.0, max_airmass=1.01) is False
