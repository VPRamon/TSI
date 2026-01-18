"""
Module for scheduling trends analysis.

Provides functions to compute empirical rates, fit logistic models
with interactions and predict scheduling probabilities.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import pandas as pd
from sklearn.linear_model import LogisticRegression
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import PolynomialFeatures, StandardScaler


@dataclass
class EmpiricalRates:
    """Result of compute_empirical_rates."""

    by_priority: pd.DataFrame
    by_visibility_bins: pd.DataFrame
    by_time_bins: pd.DataFrame


def compute_empirical_rates(
    df: pd.DataFrame,
    priority_col: str = "priority",
    visibility_col: str = "total_visibility_hours",
    time_col: str = "requested_hours",
    target_col: str = "scheduled_flag",
    n_bins: int = 10,
) -> EmpiricalRates:
    """
    Compute empirical scheduling rates by category.

    Args:
        df: DataFrame with data
        priority_col: Name of priority column
        visibility_col: Name of visibility column
        time_col: Name of requested time column
        target_col: Name of target column (boolean or 0/1)
        n_bins: Number of bins for continuous variables

    Returns:
        EmpiricalRates with rates by priority, visibility and time

    Raises:
        ValueError: If required columns are missing
    """
    required_cols = [priority_col, visibility_col, time_col, target_col]
    missing = [col for col in required_cols if col not in df.columns]
    if missing:
        raise ValueError(f"Missing columns in DataFrame: {missing}")

    # Normalize target to 0/1
    target = df[target_col].astype(int)

    # By priority (use unique values if few, otherwise bins)
    unique_priorities = df[priority_col].nunique()
    if unique_priorities <= 20:
        # Use unique values
        by_priority = (
            df.groupby(priority_col, as_index=False)
            .agg(
                scheduled_rate=(target_col, lambda x: x.astype(int).mean()),
                n=(target_col, "count"),
            )
            .sort_values(priority_col)
        )
    else:
        # Use bins
        df_temp = df.copy()
        df_temp["priority_binned"] = pd.cut(df[priority_col], bins=n_bins)
        by_priority = (
            df_temp.groupby("priority_binned", as_index=False)
            .agg(
                scheduled_rate=(target_col, lambda x: x.astype(int).mean()),
                n=(target_col, "count"),
                priority_mid=(priority_col, "mean"),
            )
            .sort_values("priority_mid")
        )
        by_priority = by_priority.rename(columns={"priority_binned": priority_col})

    # By visibility (bins)
    df_vis = df.copy()
    df_vis["visibility_binned"] = pd.cut(df[visibility_col], bins=n_bins, duplicates="drop")
    by_visibility = (
        df_vis.groupby("visibility_binned", as_index=False, observed=False)
        .agg(
            scheduled_rate=(target_col, lambda x: x.astype(int).mean()),
            n=(target_col, "count"),
            visibility_mid=(visibility_col, "mean"),
        )
        .sort_values("visibility_mid")
    )

    # By requested time (bins)
    df_time = df.copy()
    df_time["time_binned"] = pd.cut(df[time_col], bins=n_bins, duplicates="drop")
    by_time = (
        df_time.groupby("time_binned", as_index=False, observed=False)
        .agg(
            scheduled_rate=(time_col, lambda x: target[df_time["time_binned"] == x.name].mean()),
            n=(target_col, "count"),
            time_mid=(time_col, "mean"),
        )
        .sort_values("time_mid")
    )

    return EmpiricalRates(
        by_priority=by_priority,
        by_visibility_bins=by_visibility,
        by_time_bins=by_time,
    )


def smooth_trend(
    df: pd.DataFrame,
    x_col: str,
    y_col: str = "scheduled_flag",
    bandwidth: float = 0.3,
    n_points: int = 100,
) -> pd.DataFrame:
    """
    Compute smoothed trend using weighted moving average.

    Args:
        df: DataFrame with data
        x_col: X column (e.g., 'total_visibility_hours')
        y_col: Target Y column
        bandwidth: Bandwidth for smoothing (fraction of range)
        n_points: Number of points in smoothed curve

    Returns:
        DataFrame with columns 'x', 'y_smoothed', 'n_samples'

    Raises:
        ValueError: If columns are missing or insufficient data
    """
    if x_col not in df.columns or y_col not in df.columns:
        raise ValueError(f"Columns {x_col} or {y_col} not found")

    if len(df) < 10:
        raise ValueError("Insufficient data for smoothing (minimum 10 rows)")

    # Remove NaNs
    clean_df = df[[x_col, y_col]].dropna()
    if len(clean_df) < 10:
        raise ValueError("Insufficient data after removing NaNs")

    x_vals = clean_df[x_col].values
    y_vals = clean_df[y_col].astype(int).values

    # Cast to ndarray to satisfy type checker
    x_vals_array = np.asarray(x_vals)
    x_min, x_max = x_vals_array.min(), x_vals_array.max()
    x_range = x_max - x_min
    if x_range == 0:
        raise ValueError(f"No variation in {x_col}")

    # Generate grid
    x_grid = np.linspace(x_min, x_max, n_points)
    y_smooth = np.zeros(n_points)
    n_samples = np.zeros(n_points, dtype=int)

    bw = bandwidth * x_range

    for i, x_point in enumerate(x_grid):
        # Gaussian weights
        distances = np.abs(x_vals - x_point)
        weights = np.exp(-0.5 * (distances / bw) ** 2)

        # Normalize
        weights = weights / weights.sum()

        # Weighted average
        y_smooth[i] = np.sum(weights * y_vals)
        n_samples[i] = int(np.sum(weights > 0.01))  # Count points with significant weight

    return pd.DataFrame(
        {
            "x": x_grid,
            "y_smoothed": y_smooth,
            "n_samples": n_samples,
        }
    )


@dataclass
class LogisticModelResult:
    """Result of logistic model fit."""

    pipeline: Pipeline
    feature_names: list[str]
    auc_score: float | None
    accuracy: float
    n_samples: int
    n_scheduled: int


def fit_logistic_with_interactions(
    df: pd.DataFrame,
    priority_col: str = "priority",
    visibility_col: str = "total_visibility_hours",
    time_col: str = "requested_hours",
    target_col: str = "scheduled_flag",
    exclude_zero_visibility: bool = False,
    class_weight: str | dict[int, float] | None = "balanced",
    max_iter: int = 500,
) -> LogisticModelResult:
    """
    Fit logistic model with interaction terms.

    Includes interactions: priority × visibility, visibility × time.

    Args:
        df: DataFrame with data
        priority_col: Priority column
        visibility_col: Visibility column
        time_col: Requested time column
        target_col: Target column (boolean or 0/1)
        exclude_zero_visibility: If True, exclude rows with visibility = 0
        class_weight: Class weighting ('balanced', dict or None)
        max_iter: Maximum iterations for convergence

    Returns:
        LogisticModelResult with trained pipeline and metrics

    Raises:
        ValueError: If columns are missing or insufficient data
    """
    from sklearn.metrics import accuracy_score, roc_auc_score

    required_cols = [priority_col, visibility_col, time_col, target_col]
    missing = [col for col in required_cols if col not in df.columns]
    if missing:
        raise ValueError(f"Missing columns: {missing}")

    # Prepare data
    df_model = df[[priority_col, visibility_col, time_col, target_col]].copy()
    df_model = df_model.dropna()

    if exclude_zero_visibility:
        df_model = df_model[df_model[visibility_col] > 0]

    if len(df_model) < 20:
        raise ValueError(
            f"Insufficient data to train model: {len(df_model)} rows " f"(minimum 20 required)"
        )

    # Convert priority to numeric if categorical
    if df_model[priority_col].dtype == "object" or df_model[priority_col].dtype.name == "category":
        # Try ordinal mapping
        priority_map = {
            "Low": 1,
            "Medium": 2,
            "High": 3,
            "Very High": 4,
            "VeryHigh": 4,
            "Baja": 1,
            "Media": 2,
            "Alta": 3,
            "Muy Alta": 4,
            "MuyAlta": 4,
        }
        df_model[priority_col] = df_model[priority_col].map(priority_map)
        df_model = df_model.dropna(subset=[priority_col])

    # Rename for internal consistency
    df_model = df_model.rename(
        columns={
            priority_col: "priority_num",
            visibility_col: "visibility",
            time_col: "requested_time",
        }
    )

    X = df_model[["priority_num", "visibility", "requested_time"]].values
    y = df_model[target_col].astype(int).values

    # Verify that there are at least two classes
    y_array = np.asarray(y)
    unique_classes = np.unique(y_array)
    if len(unique_classes) < 2:
        raise ValueError(f"at least 2 classes required in target. classes found: {unique_classes}")

    # Pipeline
    scaler = StandardScaler()
    poly = PolynomialFeatures(degree=2, interaction_only=True, include_bias=False)
    clf = LogisticRegression(
        max_iter=max_iter,
        class_weight=class_weight,
        random_state=42,
        solver="lbfgs",
    )

    pipeline = Pipeline(
        [
            ("scaler", scaler),
            ("poly", poly),
            ("classifier", clf),
        ]
    )

    # Train
    pipeline.fit(X, y)

    # Predictions
    y_pred = pipeline.predict(X)
    y_pred_proba = pipeline.predict_proba(X)[:, 1]

    # Metrics
    accuracy = accuracy_score(y, y_pred)

    try:
        auc = roc_auc_score(y, y_pred_proba)
    except ValueError:
        auc = None  # Only one class in data

    # Feature names after poly
    feature_names_input = ["priority_num", "visibility", "requested_time"]
    poly_fitted = PolynomialFeatures(degree=2, interaction_only=True, include_bias=False)
    poly_fitted.fit(X)
    feature_names = poly_fitted.get_feature_names_out(feature_names_input)

    # Cast to ndarray to get sum attribute
    y_array = np.asarray(y)

    return LogisticModelResult(
        pipeline=pipeline,
        feature_names=list(feature_names),
        auc_score=auc,
        accuracy=accuracy,
        n_samples=len(df_model),
        n_scheduled=int(y_array.sum()),
    )


def predict_probs(
    df: pd.DataFrame,
    model_result: LogisticModelResult,
    priority_col: str = "priority",
    visibility_col: str = "total_visibility_hours",
    time_col: str = "requested_hours",
    fixed_params: dict[str, float] | None = None,
) -> pd.DataFrame:
    """
    Predict scheduling probabilities using trained model.

    Args:
        df: DataFrame with data for prediction
        model_result: Logistic model result
        priority_col: Priority column
        visibility_col: Visibility column
        time_col: Requested time column
        fixed_params: Fixed values for some variables (e.g., {'requested_time': 2.0})

    Returns:
        DataFrame with original columns + 'scheduled_prob'

    Raises:
        ValueError: If columns are missing
    """
    required_cols = [priority_col, visibility_col, time_col]
    missing = [col for col in required_cols if col not in df.columns]
    if missing:
        raise ValueError(f"Missing columns: {missing}")

    df_pred = df.copy()

    # Convert priority if categorical
    if df_pred[priority_col].dtype == "object" or df_pred[priority_col].dtype.name == "category":
        priority_map = {
            "Low": 1,
            "Medium": 2,
            "High": 3,
            "Very High": 4,
            "VeryHigh": 4,
            "Baja": 1,
            "Media": 2,
            "Alta": 3,
            "Muy Alta": 4,
            "MuyAlta": 4,
        }
        df_pred["priority_num"] = df_pred[priority_col].map(priority_map)
    else:
        df_pred["priority_num"] = df_pred[priority_col]

    df_pred = df_pred.rename(
        columns={
            visibility_col: "visibility",
            time_col: "requested_time",
        }
    )

    # Apply fixed parameters
    if fixed_params:
        for param, value in fixed_params.items():
            if param in df_pred.columns:
                df_pred[param] = value

    X_pred = df_pred[["priority_num", "visibility", "requested_time"]].values

    # Predict
    probs = model_result.pipeline.predict_proba(X_pred)[:, 1]

    df_result = df.copy()
    df_result["scheduled_prob"] = probs

    return df_result


def create_prediction_grid(
    visibility_range: tuple[float, float],
    priority_levels: list[float],
    requested_time: float = 1.0,
    n_points: int = 100,
) -> pd.DataFrame:
    """
    Create prediction grid for visualization.

    Args:
        visibility_range: Visibility range (min, max)
        priority_levels: Priority levels to evaluate
        requested_time: Fixed requested time value
        n_points: Points on visibility axis

    Returns:
        DataFrame with variable combinations for prediction
    """
    vis_min, vis_max = visibility_range
    vis_grid = np.linspace(vis_min, vis_max, n_points)

    rows = []
    for priority in priority_levels:
        for vis in vis_grid:
            rows.append(
                {
                    "priority": priority,
                    "total_visibility_hours": vis,
                    "requested_hours": requested_time,
                }
            )

    return pd.DataFrame(rows)
