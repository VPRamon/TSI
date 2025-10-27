"""Unit tests for :mod:`core.algorithms`."""

from __future__ import annotations

import pandas as pd
import pytest

from core.algorithms import compute_metrics, greedy_schedule
from core.domain import Observation

pytestmark = pytest.mark.unit


def test_compute_metrics__with_basic_dataframe__returns_expected_snapshot() -> None:
    """Validate the computed snapshot for a minimal dataframe."""

    # Given: a dataframe with a scheduled and unscheduled observation
    df = pd.DataFrame(
        {
            "priority": [5, 6],
            "scheduled_flag": [True, False],
            "total_visibility_hours": [1.0, 2.0],
            "requested_hours": [1.0, 1.5],
        }
    )

    # When: computing the aggregate metrics snapshot
    snapshot = compute_metrics(df)

    # Then: totals and scheduled counts should match expectations
    assert snapshot.total_observations == 2
    assert snapshot.scheduled_count == 1
    assert pytest.approx(snapshot.scheduling_rate) == 0.5


def test_greedy_schedule__with_two_observations__produces_converged_solution() -> None:
    """Ensure the greedy solver returns a converged solution."""

    # Given: a pair of observations with compatible windows
    observations = [
        Observation("a", 0.0, 0.0, 5.0, 1.0),
        Observation("b", 10.0, 0.0, 3.0, 1.0),
    ]

    # When: running the greedy scheduling algorithm
    result = greedy_schedule(observations)

    # Then: the algorithm should converge and produce assignments
    assert result.solution
    assert result.converged
