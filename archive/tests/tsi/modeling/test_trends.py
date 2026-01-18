"""Unit tests for :mod:`tsi.modeling.trends`."""

from __future__ import annotations

import numpy as np
import pandas as pd
import pytest
from hypothesis import given
from hypothesis import strategies as st

from tsi.modeling.trends import (
    compute_empirical_rates,
    create_prediction_grid,
    fit_logistic_with_interactions,
    predict_probs,
    smooth_trend,
)

pytestmark = pytest.mark.unit


@pytest.fixture
def minimal_df() -> pd.DataFrame:
    """Create a minimal random dataframe with scheduling outcomes."""

    rng = np.random.default_rng(42)
    n = 120
    return pd.DataFrame(
        {
            "priority": rng.integers(1, 10, size=n),
            "total_visibility_hours": rng.uniform(0.5, 120.0, size=n),
            "requested_hours": rng.uniform(0.25, 4.0, size=n),
            "scheduled_flag": rng.integers(0, 2, size=n),
        }
    )


@pytest.fixture
def df_with_zero_visibility() -> pd.DataFrame:
    """Dataset with explicit zero-visibility rows for exclusion logic."""

    rng = np.random.default_rng(123)
    n = 60
    return pd.DataFrame(
        {
            "priority": rng.integers(1, 10, size=n),
            "total_visibility_hours": np.concatenate(
                [np.zeros(15), rng.uniform(5.0, 120.0, size=n - 15)]
            ),
            "requested_hours": rng.uniform(0.5, 5.0, size=n),
            "scheduled_flag": np.concatenate(
                [np.zeros(15, dtype=int), rng.integers(0, 2, size=n - 15)]
            ),
        }
    )


def test_compute_empirical_rates__with_complete_dataset__returns_all_tables(
    minimal_df: pd.DataFrame,
) -> None:
    """Empirical rates computation should return tables for all facets."""

    # Given / When: computing empirical rates with 5 bins
    result = compute_empirical_rates(minimal_df, n_bins=5)

    # Then: each aggregated table should be populated and normalized
    assert not result.by_priority.empty
    assert not result.by_visibility_bins.empty
    assert not result.by_time_bins.empty
    assert (result.by_priority["scheduled_rate"].between(0, 1)).all()


def test_compute_empirical_rates__with_missing_columns__raises_value_error() -> None:
    """Missing required columns should raise an error message."""

    # Given: incomplete dataframe
    df = pd.DataFrame({"priority": [1, 2, 3]})

    # When / Then: expect validation error
    with pytest.raises(ValueError, match="Missing columns"):
        compute_empirical_rates(df)


def test_smooth_trend__with_bandwidth__returns_smoothed_curve(minimal_df: pd.DataFrame) -> None:
    """Trend smoothing should produce a dataframe with smoothed values."""

    # Given / When: smoothing on visibility vs scheduling
    result = smooth_trend(
        minimal_df,
        x_col="total_visibility_hours",
        y_col="scheduled_flag",
        bandwidth=0.3,
    )

    # Then: expected columns should exist with normalized values
    assert {"x", "y_smoothed", "n_samples"}.issubset(result.columns)
    assert (result["y_smoothed"].between(0, 1)).all()


def test_smooth_trend__with_insufficient_samples__raises_value_error() -> None:
    """Too few samples should produce a helpful error."""

    # Given: dataset below minimum sample requirement
    df = pd.DataFrame({"x": [1, 2, 3], "y": [0, 1, 0]})

    # When / Then: smoothing should fail with informative message
    with pytest.raises(ValueError, match="Insufficient data"):
        smooth_trend(df, x_col="x", y_col="y")


def test_fit_logistic_with_interactions__with_valid_dataset__returns_fitted_model(
    minimal_df: pd.DataFrame,
) -> None:
    """Fitting logistic model should produce a pipeline with interaction terms."""

    # Given / When: fitting the logistic regression pipeline
    result = fit_logistic_with_interactions(minimal_df)

    # Then: metadata is populated and includes interaction features
    assert result.pipeline is not None
    assert 0.0 <= result.accuracy <= 1.0
    assert result.n_samples == len(minimal_df)
    assert any(" " in feature for feature in result.feature_names)


def test_fit_logistic_with_interactions__with_zero_visibility_filter__reduces_samples(
    df_with_zero_visibility: pd.DataFrame,
) -> None:
    """Excluding zero-visibility rows should reduce training sample size."""

    # Given / When: fitting with and without the exclusion flag
    included = fit_logistic_with_interactions(
        df_with_zero_visibility, exclude_zero_visibility=False
    )
    excluded = fit_logistic_with_interactions(df_with_zero_visibility, exclude_zero_visibility=True)

    # Then: excluded run should have fewer samples but same feature dimensionality
    assert excluded.n_samples < included.n_samples
    assert len(excluded.feature_names) == len(included.feature_names)


def test_fit_logistic_with_interactions__with_single_class__raises_value_error() -> None:
    """Datasets with a single class should be rejected."""

    # Given: dataset containing only scheduled observations
    df = pd.DataFrame(
        {
            "priority": [1, 2, 3, 4, 5] * 5,
            "total_visibility_hours": [10, 20, 30, 40, 50] * 5,
            "requested_hours": [1, 2, 3, 4, 5] * 5,
            "scheduled_flag": [1] * 25,
        }
    )

    # When / Then: fitting should raise a descriptive error
    with pytest.raises(ValueError, match="at least 2 classes"):
        fit_logistic_with_interactions(df)


def test_predict_probs__with_fitted_model__returns_probabilities(minimal_df: pd.DataFrame) -> None:
    """Predictions should include the ``scheduled_prob`` column in range."""

    # Given: trained model result
    model_result = fit_logistic_with_interactions(minimal_df)

    # When: predicting on the same dataframe
    predictions = predict_probs(minimal_df, model_result)

    # Then: predictions contain probability column with valid bounds
    assert "scheduled_prob" in predictions.columns
    assert len(predictions) == len(minimal_df)
    assert (predictions["scheduled_prob"].between(0, 1)).all()


def test_create_prediction_grid__with_priority_levels__returns_cartesian_product() -> None:
    """Prediction grid should honor the requested resolution and ranges."""

    # Given / When: creating a grid across visibility and priority levels
    grid = create_prediction_grid(
        visibility_range=(0, 100),
        priority_levels=[1, 5, 10],
        requested_time=2.0,
        n_points=40,
    )

    # Then: grid length equals number of points times priority levels
    assert len(grid) == 40 * 3
    assert {"priority", "total_visibility_hours", "requested_hours"}.issubset(grid.columns)
    assert (grid["requested_hours"] == 2.0).all()


@given(
    visibility_hours=st.floats(min_value=0.0, max_value=120.0),
    requested_hours=st.floats(min_value=0.25, max_value=5.0),
    priority=st.integers(min_value=1, max_value=10),
)
def test_create_prediction_grid__property_constraints__produce_valid_ranges(
    visibility_hours: float,
    requested_hours: float,
    priority: int,
) -> None:
    """Hypothesis-based sanity check on grid generation inputs."""

    # Given: a single-point grid request from generated inputs
    grid = create_prediction_grid(
        visibility_range=(0, visibility_hours + 1.0),
        priority_levels=[priority],
        requested_time=requested_hours,
        n_points=1,
    )

    # Then: the resulting row should respect requested parameters
    assert len(grid) == 1
    assert grid.iloc[0]["priority"] == priority
    assert grid.iloc[0]["requested_hours"] == pytest.approx(requested_hours)
    assert 0 <= grid.iloc[0]["total_visibility_hours"] <= visibility_hours + 1.0
