"""
Analysis page for unscheduled observation blocks.

This page helps explain why an observation block was not scheduled using trained
machine-learning models and SHAP explainers.
"""

import json
import logging
from datetime import datetime
from typing import Any

import joblib
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import shap
import streamlit as st

from tsi import state
from tsi.config import PROJECT_ROOT
from tsi.modeling.data.feature_engineering import AstronomicalFeatureEngineer
from tsi.services.visibility_cache import ensure_visibility_parsed

# Configure logging
logger = logging.getLogger(__name__)

# Paths to model artifacts
ARTIFACTS_DIR = PROJECT_ROOT / "src" / "tsi" / "modeling" / "artifacts"
MODEL_PATH = ARTIFACTS_DIR / "models" / "best_model.pkl"
PREPROCESSOR_PATH = ARTIFACTS_DIR / "preprocessors" / "preprocessor.pkl"
THRESHOLD_PATH = ARTIFACTS_DIR / "threshold.json"
CONFIG_PATH = PROJECT_ROOT / "src" / "tsi" / "modeling" / "config" / "model_config.yaml"
LOGS_DIR = PROJECT_ROOT / "src" / "tsi" / "modeling" / "reports" / "streamlit_logs"


def prepare_observation_data(df: pd.DataFrame) -> pd.DataFrame:
    """
    Prepare observation data by applying feature engineering.

    Args:
        df: DataFrame with the base observation data.

    Returns:
        DataFrame enriched with additional engineered features.
    """
    try:
        # Build a simple configuration object for the feature engineer
        # We only need the object, not the full settings
        class SimpleConfig:
            pass

        config = SimpleConfig()

        # Ensure the basic columns use the expected names
        df_prepared = df.copy()

        # Map column names when necessary
        column_mapping = {
            "raInDeg": "ra_deg",
            "decInDeg": "dec_deg",
            "priority": "Priority",  # The feature engineer expects 'Priority' capitalized
        }

        for old_col, new_col in column_mapping.items():
            if old_col in df_prepared.columns and new_col not in df_prepared.columns:
                df_prepared[new_col] = df_prepared[old_col]

        # Also ensure the inverse columns exist
        for new_col, old_col in column_mapping.items():
            if new_col in df_prepared.columns and old_col not in df_prepared.columns:
                df_prepared[old_col] = df_prepared[new_col]

        # Ensure Priority exists and has valid values
        if "Priority" not in df_prepared.columns and "priority" in df_prepared.columns:
            df_prepared["Priority"] = df_prepared["priority"]

        # If Priority is NaN, fall back to a default value
        if "Priority" in df_prepared.columns:
            df_prepared["Priority"] = df_prepared["Priority"].fillna(
                5.0
            )  # Medium priority by default

        # Compute scheduled_seconds if scheduled period information is available
        if (
            "scheduled_period.start" in df_prepared.columns
            and "scheduled_period.stop" in df_prepared.columns
        ):
            # When a scheduled period exists, compute its duration in seconds
            if df_prepared["scheduled_period.start"].notna().any():
                df_prepared["scheduled_seconds"] = (
                    (df_prepared["scheduled_period.stop"] - df_prepared["scheduled_period.start"])
                    * 86400
                ).fillna(0)
            else:
                df_prepared["scheduled_seconds"] = 0
        else:
            df_prepared["scheduled_seconds"] = 0

        # Ensure visibility_periods_parsed is available (with smart caching)
        # This only parses when not cached to avoid repeated work
        df_prepared = ensure_visibility_parsed(df_prepared)

        # Compute visibility statistics from visibility_periods_parsed
        if "visibility_periods_parsed" in df_prepared.columns:

            def calc_visibility_stats(periods: Any) -> dict[str, float]:
                """Calculate statistics for visibility periods."""
                if not periods or len(periods) == 0:
                    return {
                        "num_visibility_periods": 0,
                        "mean_visibility_duration": 0.0,
                        "max_visibility_gap": 0.0,
                    }

                # Calculate durations in hours
                durations = []
                for start_dt, end_dt in periods:
                    if start_dt and end_dt:
                        duration_hours = (end_dt - start_dt).total_seconds() / 3600.0
                        durations.append(duration_hours)

                # Calculate gaps between periods
                gaps = []
                for i in range(len(periods) - 1):
                    if periods[i][1] and periods[i + 1][0]:
                        gap_hours = (periods[i + 1][0] - periods[i][1]).total_seconds() / 3600.0
                        gaps.append(gap_hours)

                return {
                    "num_visibility_periods": len(periods),
                    "mean_visibility_duration": np.mean(durations) if durations else 0.0,
                    "max_visibility_gap": max(gaps) if gaps else 0.0,
                }

            # Apply the calculation row by row
            visibility_stats = df_prepared["visibility_periods_parsed"].apply(calc_visibility_stats)

            for stat_name in [
                "num_visibility_periods",
                "mean_visibility_duration",
                "max_visibility_gap",
            ]:
                if stat_name not in df_prepared.columns:
                    df_prepared[stat_name] = visibility_stats.apply(lambda x: x[stat_name])
        else:
            # If no parsed visibility data, use default values
            for col in ["num_visibility_periods", "mean_visibility_duration", "max_visibility_gap"]:
                if col not in df_prepared.columns:
                    df_prepared[col] = 0 if col == "num_visibility_periods" else 0.0

        # Ensure total_visibility_hours exists
        if "total_visibility_hours" not in df_prepared.columns:
            # If it doesn't exist, try to calculate it from the periods
            if "visibility_periods_parsed" in df_prepared.columns:

                def calc_total_hours(periods: Any) -> float:
                    if not periods or len(periods) == 0:
                        return 0.0
                    total = 0.0
                    for start_dt, end_dt in periods:
                        if start_dt and end_dt:
                            total += (end_dt - start_dt).total_seconds() / 3600.0
                    return total

                df_prepared["total_visibility_hours"] = df_prepared[
                    "visibility_periods_parsed"
                ].apply(calc_total_hours)
            else:
                df_prepared["total_visibility_hours"] = 0.0

        # Apply feature engineering
        engineer = AstronomicalFeatureEngineer(config)

        # For a single observation, some features need special handling
        # Run feature engineering steps individually to handle errors
        try:
            df_engineered = engineer.create_astronomical_features(df_prepared)
        except Exception as e:
            logger.warning(f"Error creating astronomical features: {e}")
            df_engineered = df_prepared

        try:
            # For operational features, handle the priority_category edge case
            # which fails with pd.cut when there is only one value
            df_temp = engineer.create_operational_features(df_engineered)
            df_engineered = df_temp
        except Exception as e:
            logger.warning(f"Error creating operational features: {e}")
            # Manually create the basic operational features
            if "Priority" in df_engineered.columns:
                df_engineered["priority"] = df_engineered["Priority"]
                priority_val = df_engineered["priority"].iloc[0]
                priority_min = max(0, priority_val - 10)  # Artificial range for normalization
                priority_max = min(100, priority_val + 10)

                if priority_max > priority_min:
                    df_engineered["priority_normalized"] = (
                        df_engineered["priority"] - priority_min
                    ) / (priority_max - priority_min)
                else:
                    df_engineered["priority_normalized"] = 0.5

                # Assign category based on value
                if priority_val <= 5:
                    df_engineered["priority_category"] = "Low"
                elif priority_val <= 10:
                    df_engineered["priority_category"] = "Medium"
                elif priority_val <= 20:
                    df_engineered["priority_category"] = "High"
                else:
                    df_engineered["priority_category"] = "Critical"

            # Duration features
            if "requestedDurationSec" in df_engineered.columns:
                df_engineered["duration_minutes"] = df_engineered["requestedDurationSec"] / 60.0
                df_engineered["duration_hours"] = df_engineered["requestedDurationSec"] / 3600.0

            if "minObservationTimeInSec" in df_engineered.columns:
                df_engineered["min_observation_minutes"] = (
                    df_engineered["minObservationTimeInSec"] / 60.0
                )

                if "duration_minutes" in df_engineered.columns:
                    df_engineered["duration_flexibility"] = (
                        df_engineered["duration_minutes"]
                        / df_engineered["min_observation_minutes"].clip(lower=1)
                    ) - 1.0

            # Period saturation (use a neutral value for a single observation)
            df_engineered["period_saturation"] = 1
            df_engineered["period_saturation_normalized"] = 0.5
            df_engineered["competing_observations"] = 0

        try:
            df_engineered = engineer.create_coordinate_features(df_engineered)
        except Exception as e:
            logger.warning(f"Error creating coordinate features: {e}")

        try:
            df_engineered = engineer.create_interaction_features(df_engineered)
        except Exception as e:
            logger.warning(f"Error creating interaction features: {e}")

        logger.info(
            f"Feature engineering completed. Original columns: {len(df.columns)}, Final columns: {len(df_engineered.columns)}"
        )

        return df_engineered  # type: ignore[return-value,no-any-return]

    except Exception as e:
        logger.exception("Error during feature engineering")
        st.warning(f"⚠️ Not all features could be generated: {str(e)}")
        # Return the original DataFrame if feature engineering fails
        return df


def load_artifacts() -> tuple[Any, Any, float, dict]:
    """
    Load the trained model artifacts required for analysis.

    Returns:
        Tuple of (model, preprocessor, threshold, configuration).
    """
    try:
        # Load model
        if not MODEL_PATH.exists():
            st.error(f"❌ Model not found at: {MODEL_PATH}")
            st.info("💡 Please train the model first by running `python scripts/train_model.py`")
            return (None, None, None, None)  # type: ignore[return-value]

        model = joblib.load(MODEL_PATH)
        logger.info(f"✅ Model loaded from {MODEL_PATH}")

        # Load preprocessor
        if not PREPROCESSOR_PATH.exists():
            st.error(f"❌ Preprocessor not found at: {PREPROCESSOR_PATH}")
            return (None, None, None, None)  # type: ignore[return-value]

        preprocessor_data = joblib.load(PREPROCESSOR_PATH)
        preprocessor = preprocessor_data
        logger.info(f"✅ Preprocessor loaded from {PREPROCESSOR_PATH}")

        # Load threshold
        threshold = 0.5  # Default value
        if THRESHOLD_PATH.exists():
            with open(THRESHOLD_PATH) as f:
                threshold_data = json.load(f)
                threshold = threshold_data.get("optimal_threshold", 0.5)
            logger.info(f"✅ Threshold loaded: {threshold}")
        else:
            st.warning(f"⚠️ Threshold not found. Using default value: {threshold}")

        # Load configuration (optional)
        config = {}
        if CONFIG_PATH.exists():
            try:
                import yaml

                with open(CONFIG_PATH) as f:
                    config = yaml.safe_load(f)
                logger.info("✅ Configuration loaded")
            except ImportError:
                logger.warning("PyYAML not installed, skipping config file")
            except Exception as e:
                logger.warning(f"Could not load config: {e}")

        return model, preprocessor, threshold, config

    except Exception as e:
        st.error(f"❌ Error loading artifacts: {str(e)}")
        logger.exception("Error loading artifacts")
        return (None, None, None, None)  # type: ignore[return-value]


def validate_input_data(df: pd.DataFrame, preprocessor: Any) -> tuple[bool, list]:
    """
    Validate that the input data contains the required columns.

    Args:
        df: Input DataFrame.
        preprocessor: Trained preprocessor.

    Returns:
        Tuple of (is_valid, missing_columns).
    """
    try:
        # Retrieve the expected columns from the transformer
        # The ColumnTransformer stores them in the transformers attribute
        expected_columns = set()

        # The preprocessor may be a dict or an object
        if isinstance(preprocessor, dict):
            # Dictionary with the transformer stored inside
            transformer = preprocessor.get("transformer")
            if transformer and hasattr(transformer, "transformers_"):
                for name, trans, columns in transformer.transformers_:
                    if name != "remainder":
                        expected_columns.update(columns)
        elif hasattr(preprocessor, "transformer"):
            # Object exposing a transformer attribute
            for name, trans, columns in preprocessor.transformer.transformers_:
                if name != "remainder":
                    expected_columns.update(columns)
        elif hasattr(preprocessor, "transformers_"):
            # The transformer itself
            for name, trans, columns in preprocessor.transformers_:
                if name != "remainder":
                    expected_columns.update(columns)
        else:
            # Unable to determine the expected columns
            logger.warning("Could not determine expected columns from the preprocessor")
            return True, []  # Assume valid when validation is not possible

        # Check for missing columns
        input_columns = set(df.columns)
        missing_columns = expected_columns - input_columns

        return len(missing_columns) == 0, list(missing_columns)

    except Exception as e:
        logger.exception("Error validating input data")
        return False, [f"Validation error: {str(e)}"]


def get_unscheduled_observations(df: pd.DataFrame) -> pd.DataFrame:
    """
    Filter and return only the unscheduled observations from the dataset.

    Args:
        df: Full dataset DataFrame.

    Returns:
        DataFrame containing only unscheduled observations.
    """
    # Look for a column that indicates whether the observation was scheduled
    # Possible values include 'Scheduled', 'scheduled', 'planificada', etc.
    scheduled_col = None

    for col in df.columns:
        if col.lower() in ["scheduled", "planificada", "is_scheduled"]:
            scheduled_col = col
            break

    if scheduled_col is None:
        # If there is no explicit column, fall back to scheduled_period
        if "scheduled_period.start" in df.columns:
            # Unscheduled rows have NaN in scheduled_period.start
            return df[df["scheduled_period.start"].isna()].copy()
        else:
            # Return the full dataset if scheduling status cannot be determined
            st.warning("⚠️ Unable to determine which observations are scheduled")
            return df.copy()
    else:
        # Filter unscheduled observations (False, 0, No, etc.)
        if df[scheduled_col].dtype == bool:
            return df[~df[scheduled_col]].copy()
        elif df[scheduled_col].dtype in ["int64", "int32"]:
            return df[df[scheduled_col] == 0].copy()
        else:
            # String: 'No', 'False', etc.
            return df[df[scheduled_col].str.lower().isin(["no", "false", "0", "n"])].copy()


def _format_value_for_display(value: Any) -> str:
    """Convert any value into a Streamlit-friendly representation."""
    if isinstance(value, np.ndarray):
        value = value.tolist()
    if isinstance(value, set):
        value = sorted(value)
    if isinstance(value, (list, tuple)):
        try:
            return json.dumps(value, ensure_ascii=False)
        except TypeError:
            return str(value)
    if isinstance(value, dict):
        try:
            return json.dumps(value, ensure_ascii=False)
        except TypeError:
            return str(value)
    if isinstance(value, (pd.Timestamp, np.datetime64)):
        return pd.to_datetime(value).isoformat()
    if isinstance(value, (bytes, bytearray)):
        return value.decode("utf-8", errors="replace")
    if isinstance(value, np.generic):
        value = value.item()
    try:
        if pd.isna(value):
            return "NaN"
    except Exception:
        pass
    return str(value)


def _flatten_shap_values(shap_values: Any) -> np.ndarray:
    """Normalize SHAP outputs to a 1D vector."""
    if shap_values is None:
        return np.array([])
    if hasattr(shap_values, "values"):
        shap_values = shap_values.values
    if isinstance(shap_values, list):
        if len(shap_values) == 0:
            return np.array([])
        # Use the positive class when available
        shap_values = shap_values[1] if len(shap_values) > 1 else shap_values[0]
    shap_values_array = np.asarray(shap_values)
    if shap_values_array.ndim == 0:
        return np.array([float(shap_values_array)])
    if shap_values_array.ndim == 1:
        return shap_values_array  # type: ignore[return-value,no-any-return]
    # For 2D or higher outputs, take the first row
    return shap_values_array[0]  # type: ignore[return-value,no-any-return]


def _extract_base_value(base_value: Any) -> float:
    """Extract a scalar base value from various SHAP structures."""
    if hasattr(base_value, "values"):
        base_value = base_value.values
    if isinstance(base_value, (list, tuple)):
        if len(base_value) > 1:
            base_value = base_value[1]
        else:
            base_value = base_value[0]
    if isinstance(base_value, np.ndarray):
        if base_value.ndim == 0:
            base_value = base_value.item()
        else:
            flat = base_value.flatten()
            base_value = flat[1] if flat.size > 1 else flat[0]
    if isinstance(base_value, np.generic):
        base_value = base_value.item()
    try:
        return float(base_value)
    except (TypeError, ValueError):
        return 0.5


def _json_serializer(value: Any) -> Any:
    """Convert non-serializable objects into safe JSON representations."""
    if isinstance(value, (pd.Timestamp, np.datetime64, datetime)):
        return pd.to_datetime(value).isoformat()
    if isinstance(value, (np.integer, np.floating, np.bool_)):
        return value.item()
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, (pd.Series, pd.Index)):
        return value.tolist()
    if isinstance(value, set):
        return sorted(value)
    if isinstance(value, (bytes, bytearray)):
        return value.decode("utf-8", errors="replace")
    raise TypeError(f"Object of type {type(value).__name__} is not JSON serializable")


def make_dataframe_arrow_safe(df: pd.DataFrame) -> pd.DataFrame:
    """Return a copy of the DataFrame converted to string-safe values for Arrow."""
    return df.map(_format_value_for_display)


def analyze_observation(
    df: pd.DataFrame, model: Any, preprocessor: Any, threshold: float, config: dict
) -> dict[str, Any]:
    """Analyze an observation and generate a prediction and explanation."""
    try:
        # Transform data
        if hasattr(preprocessor, "transform"):
            X_transformed = preprocessor.transform(df)
        else:
            X_transformed = preprocessor["transformer"].transform(df)

        # Retrieve feature names
        if hasattr(preprocessor, "feature_names_out"):
            feature_names = preprocessor.feature_names_out
        elif "feature_names_out" in preprocessor:
            feature_names = preprocessor["feature_names_out"]
        else:
            feature_names = [f"feature_{i}" for i in range(X_transformed.shape[1])]

        # Prediction
        y_proba = model.predict_proba(X_transformed)[0, 1]
        y_pred = int(y_proba >= threshold)

        # Compute SHAP values
        shap_input = X_transformed
        if hasattr(shap_input, "toarray"):
            shap_input_dense = shap_input.toarray()
        else:
            shap_input_dense = np.asarray(shap_input)
        if shap_input_dense.ndim == 1:
            shap_input_dense = shap_input_dense.reshape(1, -1)

        try:
            if hasattr(model, "coef_") and not hasattr(model, "estimators_"):
                explainer = shap.LinearExplainer(model, shap_input_dense)
                shap_raw = explainer.shap_values(shap_input_dense)
                base_value = _extract_base_value(explainer.expected_value)
            else:
                explainer = shap.TreeExplainer(model)
                shap_raw = explainer.shap_values(shap_input_dense)
                base_value = _extract_base_value(explainer.expected_value)
            shap_values = _flatten_shap_values(shap_raw)
        except Exception:
            try:
                explainer = shap.Explainer(model, shap_input_dense)
                explanation = explainer(shap_input_dense)
                shap_values = _flatten_shap_values(explanation.values)
                base_value = _extract_base_value(explanation.base_values)
            except Exception as fallback_error:
                logger.warning(f"Could not compute SHAP values: {fallback_error}")
                shap_values = np.zeros(len(feature_names))
                base_value = 0.5

        if shap_values.size != len(feature_names):
            aligned = np.zeros(len(feature_names))
            limit = min(len(feature_names), shap_values.size)
            if limit > 0:
                aligned[:limit] = shap_values[:limit]
            shap_values = aligned

        # Build contribution dictionary
        feature_contributions = dict(zip(feature_names, shap_values))

        # Sort by absolute impact
        sorted_features = sorted(
            feature_contributions.items(), key=lambda x: abs(x[1]), reverse=True
        )

        # Generate recommendations
        recommendations = generate_recommendations(sorted_features, y_pred, df.iloc[0].to_dict())

        return {
            "probability": y_proba,
            "predicted_class": y_pred,
            "decision": "SCHEDULED" if y_pred == 1 else "NOT SCHEDULED",
            "threshold": threshold,
            "shap_values": shap_values,
            "base_value": base_value,
            "feature_names": feature_names,
            "feature_contributions": feature_contributions,
            "top_factors": sorted_features[:10],
            "recommendations": recommendations,
            "transformed_features": X_transformed[0],
        }

    except Exception:
        logger.exception("Error during observation analysis")
        raise


def generate_recommendations(
    sorted_features: list, prediction: int, original_data: dict
) -> list[str]:
    """Generate recommendations based on the most influential factors."""
    recommendations = []

    if prediction == 0:  # Not scheduled
        # Identify the main negative factors
        negative_factors = [(f, v) for f, v in sorted_features[:5] if v < 0]

        if negative_factors:
            recommendations.append("🔍 **Factors that hinder scheduling:**")

            for feature, shap_val in negative_factors:
                # Generate feature-specific guidance
                if "priority" in feature.lower():
                    recommendations.append(
                        f"  • **Low priority** (impact: {shap_val:.3f}): "
                        "Consider increasing the priority if the observation is critical."
                    )
                elif "visibility" in feature.lower() or "hours" in feature.lower():
                    recommendations.append(
                        f"  • **Limited visibility** (impact: {shap_val:.3f}): "
                        "Adjust the time window or check elevation constraints."
                    )
                elif "saturation" in feature.lower():
                    recommendations.append(
                        f"  • **High period saturation** (impact: {shap_val:.3f}): "
                        "Try scheduling in a less congested period."
                    )
                elif "duration" in feature.lower():
                    recommendations.append(
                        f"  • **Duration** (impact: {shap_val:.3f}): "
                        "Review whether the requested duration is realistic given the available visibility."
                    )
                else:
                    recommendations.append(f"  • **{feature}** (impact: {shap_val:.3f})")

        recommendations.append("\n💡 **Suggested actions:**")
        recommendations.append("  • Review the observation window and adjust dates if possible")
        recommendations.append("  • Verify that elevation/azimuth constraints are accurate")
        recommendations.append(
            "  • Consider increasing the priority if the observation is critical"
        )

    else:  # Scheduled
        recommendations.append("✅ **The observation is well positioned for scheduling**")
        recommendations.append("\n🎯 **Positive factors:**")

        positive_factors = [(f, v) for f, v in sorted_features[:3] if v > 0]
        for feature, shap_val in positive_factors:
            recommendations.append(f"  • {feature} (impact: {shap_val:.3f})")

    return recommendations


def plot_shap_waterfall(
    base_value: float, shap_values: np.ndarray, feature_names: list, top_n: int = 10
) -> plt.Figure:
    """Create a SHAP waterfall-style bar chart for the top factors."""
    # Order by absolute value
    sorted_idx = np.argsort(np.abs(shap_values))[::-1][:top_n]
    sorted_values = shap_values[sorted_idx]
    sorted_names = [feature_names[i] for i in sorted_idx]

    # Create figure
    fig, ax = plt.subplots(figsize=(10, 6))

    # Colors: red for negative, teal for positive
    colors = ["#ff6b6b" if v < 0 else "#4ecdc4" for v in sorted_values]

    # Build horizontal bars
    y_pos = np.arange(len(sorted_names))
    ax.barh(y_pos, sorted_values, color=colors, alpha=0.8)

    # Configure axes
    ax.set_yticks(y_pos)
    ax.set_yticklabels(sorted_names)
    ax.set_xlabel("Impact on Prediction (SHAP value)", fontsize=12)
    ax.set_title(f"Top {top_n} Most Influential Factors", fontsize=14, fontweight="bold")
    ax.axvline(x=0, color="black", linestyle="-", linewidth=0.8)

    # Add grid
    ax.grid(axis="x", alpha=0.3, linestyle="--")

    # Invert y-axis so the most important feature is on top
    ax.invert_yaxis()

    # Adjust layout
    plt.tight_layout()

    return fig


def plot_feature_impact(feature_contributions: dict[str, float], top_n: int = 10) -> plt.Figure:
    """Create a bar chart showing feature impact by SHAP value."""
    # Order by absolute impact
    sorted_items = sorted(feature_contributions.items(), key=lambda x: abs(x[1]), reverse=True)[
        :top_n
    ]

    features = [item[0] for item in sorted_items]
    values = [item[1] for item in sorted_items]

    # Build figure
    fig, ax = plt.subplots(figsize=(10, 6))

    # Colors by sign
    colors = ["#ff6b6b" if v < 0 else "#4ecdc4" for v in values]

    # Create bars
    bars = ax.barh(range(len(features)), values, color=colors, alpha=0.8)

    # Configure
    ax.set_yticks(range(len(features)))
    ax.set_yticklabels(features)
    ax.set_xlabel("Contribution to Prediction", fontsize=12)
    ax.set_title(f"Top {top_n} Most Influential Features", fontsize=14, fontweight="bold")
    ax.axvline(x=0, color="black", linestyle="-", linewidth=0.8)
    ax.grid(axis="x", alpha=0.3, linestyle="--")
    ax.invert_yaxis()

    # Annotate bar values
    for i, (bar, val) in enumerate(zip(bars, values)):
        x_pos = val + (0.01 if val > 0 else -0.01)
        ha = "left" if val > 0 else "right"
        ax.text(x_pos, i, f"{val:.3f}", va="center", ha=ha, fontsize=9)

    plt.tight_layout()

    return fig


def save_analysis_log(block_id: str, input_data: pd.DataFrame, result: dict[str, Any]) -> None:
    """Persist the analysis to a JSON log file."""
    try:
        # Ensure directory exists
        LOGS_DIR.mkdir(parents=True, exist_ok=True)

        # Build timestamp
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        # Assemble log payload
        log_data = {
            "timestamp": timestamp,
            "block_id": block_id,
            "input_data": input_data.to_dict("records")[0],
            "prediction": {
                "probability": float(result["probability"]),
                "decision": result["decision"],
                "threshold": float(result["threshold"]),
            },
            "top_factors": [
                {"feature": f, "shap_value": float(v)} for f, v in result["top_factors"]
            ],
        }

        # Write log file
        log_file = LOGS_DIR / f"analysis_{block_id}_{timestamp}.json"
        with open(log_file, "w") as f:
            json.dump(log_data, f, indent=2, ensure_ascii=False, default=_json_serializer)

        logger.info(f"Analysis log saved to {log_file}")

    except Exception:
        logger.exception("Error saving analysis log")


def render() -> None:
    """Render the unscheduled observation analysis page."""

    # Encabezado
    st.title("🔭 Unscheduled Observation Analysis")
    st.markdown(
        "Explore the factors that influenced the scheduler's decision "
        "and discover why an observation was not scheduled."
    )

    st.divider()

    # Cargar artefactos
    with st.spinner("Loading model and artifacts..."):
        model, preprocessor, threshold, config = load_artifacts()

    if model is None or preprocessor is None:
        st.stop()

    # Display model information
    with st.expander("ℹ️ Model information", expanded=False):
        col1, col2, col3 = st.columns(3)

        with col1:
            st.metric("Model type", type(model).__name__)

        with col2:
            st.metric("Optimal threshold", f"{threshold:.3f}")

        with col3:
            n_features = (
                len(preprocessor.get("feature_names_out", []))
                if isinstance(preprocessor, dict)
                else len(getattr(preprocessor, "feature_names_out", []))
            )
            st.metric("Features", n_features)

    st.divider()

    # Verify that data is loaded
    if not state.has_data():
        st.warning("⚠️ **No data loaded**")
        st.info("Please load a dataset from the landing page before using this feature.")
        st.stop()

    # Retrieve session data
    df_full = state.get_prepared_data()

    # Filtrar observaciones no planificadas
    df_unscheduled = get_unscheduled_observations(df_full)

    if len(df_unscheduled) == 0:
        st.warning("⚠️ **There are no unscheduled observations in the dataset**")
        st.info(
            "All observations in the current dataset were scheduled. "
            "Load a different dataset to analyze unscheduled blocks."
        )
        st.stop()

    # Search and selection section
    st.header("🔍 Search for an Unscheduled Observation")

    st.info(
        f"📊 Found **{len(df_unscheduled)}** unscheduled observations "
        f"in the dataset (out of {len(df_full)} total observations)."
    )

    # Crear selector
    col1, col2 = st.columns([2, 1])

    with col1:
        # Selection options
        search_method = st.radio(
            "Selection method:",
            ["By Block ID", "By Index", "Advanced Filters"],
            horizontal=True,
        )

    with col2:
        # Show quick statistics
        st.metric("Total Unscheduled", len(df_unscheduled))

    observation_df = None
    block_id = "unknown"

    if search_method == "By Block ID":
        st.markdown("**Search by observation block ID**")

        # Obtener lista de IDs disponibles
        id_col = None
        for col in df_unscheduled.columns:
            if "id" in col.lower() or "block" in col.lower():
                id_col = col
                break

        if id_col:
            available_ids = df_unscheduled[id_col].astype(str).tolist()

            selected_id = st.selectbox(
                f"Select a block ({id_col}):",
                options=available_ids,
                help="List of all available unscheduled blocks",
            )

            if selected_id:
                observation_df = df_unscheduled[
                    df_unscheduled[id_col].astype(str) == selected_id
                ].copy()
                block_id = selected_id

                st.success(f"✅ Block selected: **{block_id}**")
        else:
            st.error("❌ No ID column found in the dataset")

    elif search_method == "By Index":
        st.markdown("**Select by row number**")

        index_to_select = st.number_input(
            "Observation index (0-based):",
            min_value=0,
            max_value=len(df_unscheduled) - 1,
            value=0,
            help="Row number of the unscheduled observation",
        )

        observation_df = df_unscheduled.iloc[[index_to_select]].copy()

        # Try to get ID
        id_col = None
        for col in observation_df.columns:
            if "id" in col.lower() or "block" in col.lower():
                id_col = col
                break

        if id_col:
            block_id = str(observation_df[id_col].iloc[0])
        else:
            block_id = f"index_{index_to_select}"

        st.success(f"✅ Observation selected (index {index_to_select}): **{block_id}**")

    else:  # Advanced filters
        st.markdown("**Filter observations by criteria**")

        filter_col1, filter_col2 = st.columns(2)

        with filter_col1:
            # Priority filter
            if "priority" in df_unscheduled.columns:
                priority_min = float(df_unscheduled["priority"].min())
                priority_max = float(df_unscheduled["priority"].max())

                priority_range = st.slider(
                    "Priority range:",
                    min_value=priority_min,
                    max_value=priority_max,
                    value=(priority_min, priority_max),
                )

                df_filtered = df_unscheduled[
                    (df_unscheduled["priority"] >= priority_range[0])
                    & (df_unscheduled["priority"] <= priority_range[1])
                ]
            else:
                df_filtered = df_unscheduled

        with filter_col2:
            # Visibility filter
            if "total_visibility_hours" in df_unscheduled.columns:
                visibility_min = float(df_unscheduled["total_visibility_hours"].min())
                visibility_max = float(df_unscheduled["total_visibility_hours"].max())

                visibility_range = st.slider(
                    "Visibility hours:",
                    min_value=visibility_min,
                    max_value=visibility_max,
                    value=(visibility_min, visibility_max),
                )

                df_filtered = df_filtered[
                    (df_filtered["total_visibility_hours"] >= visibility_range[0])
                    & (df_filtered["total_visibility_hours"] <= visibility_range[1])
                ]

        st.info(f"📊 {len(df_filtered)} observations meet the criteria")

        if len(df_filtered) > 0:
            # Mostrar tabla con las observaciones filtradas
            display_cols = []
            for col in [
                "schedulingBlockId",
                "priority",
                "total_visibility_hours",
                "requestedDurationSec",
            ]:
                if col in df_filtered.columns:
                    display_cols.append(col)

            if display_cols:
                st.dataframe(df_filtered[display_cols].head(10), width="stretch", height=200)

            # Selector for the specific observation
            index_in_filtered = st.number_input(
                "Select the index from the table above:",
                min_value=0,
                max_value=len(df_filtered) - 1,
                value=0,
            )

            observation_df = df_filtered.iloc[[index_in_filtered]].copy()

            # Obtener ID
            id_col = None
            for col in observation_df.columns:
                if "id" in col.lower() or "block" in col.lower():
                    id_col = col
                    break

            if id_col:
                block_id = str(observation_df[id_col].iloc[0])
            else:
                block_id = f"filtered_{index_in_filtered}"

            st.success(f"✅ Observation selected: **{block_id}**")
        else:
            st.warning("⚠️ No observations match the criteria. Adjust the filters.")

    # Show preview of the selected data
    if observation_df is not None and len(observation_df) > 0:
        with st.expander("👁️ Preview of Selected Data", expanded=False):
            # Convert values to strings to avoid Arrow issues when transposing
            sanitized_df = make_dataframe_arrow_safe(observation_df.iloc[[0]])
            display_df = sanitized_df.iloc[0].reset_index()
            display_df.columns = ["feature", "value"]
            st.dataframe(display_df, width="stretch")

    # If data is available, proceed with the analysis
    if observation_df is not None and len(observation_df) > 0:

        st.divider()
        st.header("🔬 Analysis")

        # Apply feature engineering before validation
        with st.spinner("Preparing data with feature engineering..."):
            observation_df_engineered = prepare_observation_data(observation_df)

        # Show information about the generated features
        with st.expander("ℹ️ Generated Features", expanded=False):
            original_cols = set(observation_df.columns)
            engineered_cols = set(observation_df_engineered.columns)
            new_cols = engineered_cols - original_cols

            st.info(f"✅ Generated **{len(new_cols)}** additional features through engineering")

            if new_cols:
                st.markdown("**New features:**")
                # Display in columns for readability
                cols_list = sorted(list(new_cols))
                num_cols = 3
                cols = st.columns(num_cols)
                for i, col_name in enumerate(cols_list):
                    with cols[i % num_cols]:
                        st.text(f"• {col_name}")

        # Validate data
        is_valid, missing_cols = validate_input_data(observation_df_engineered, preprocessor)

        if not is_valid:
            st.error("❌ **Incomplete data**")
            st.markdown("**Missing columns:**")
            for col in missing_cols:
                st.markdown(f"  - `{col}`")
            st.info("💡 Ensure the file contains all columns required by the preprocessor.")
            st.stop()

        # Analysis button
        if st.button("🚀 Analyze Block", type="primary", width="stretch"):

            with st.spinner("Analyzing observation..."):
                try:
                    # Run analysis with the prepared data
                    result = analyze_observation(
                        observation_df_engineered.iloc[[0]],  # Analizar primera fila
                        model,
                        preprocessor,
                        threshold,
                        config,
                    )

                    # Save log (with original data for reference)
                    save_analysis_log(block_id, observation_df.iloc[[0]], result)

                    st.success("✅ Analysis complete")

                    # Mostrar resultados
                    st.divider()
                    st.header("📊 Results")

                    # Block information
                    st.subheader(f"Block: `{block_id}`")

                    # Key metrics
                    col1, col2, col3 = st.columns(3)

                    with col1:
                        prob_pct = result["probability"] * 100
                        st.metric(
                            "Scheduling probability",
                            f"{prob_pct:.1f}%",
                            delta=f"{prob_pct - threshold*100:.1f}% vs threshold",
                        )

                    with col2:
                        decision_emoji = "✅" if result["predicted_class"] == 1 else "❌"
                        st.metric("Model decision", f"{decision_emoji} {result['decision']}")

                    with col3:
                        st.metric("Decision threshold", f"{threshold:.3f}")

                    # Barra de progreso visual
                    st.progress(
                        result["probability"], text=f"Confidence: {result['probability']:.1%}"
                    )

                    st.divider()

                    # Textual explanation
                    st.subheader("💬 Natural-language explanation")

                    # Generar texto explicativo
                    explanation_text = f"""
**Decision:** {result['decision']}

**Scheduling probability:** {result['probability']:.1%}

The observation is classified as **{"schedulable" if result['predicted_class'] == 1 else "not schedulable"}**
with a confidence of {abs(result['probability'] - 0.5) * 200:.1f}%.
"""

                    st.markdown(explanation_text)

                    # Factores principales
                    st.markdown("**🔍 Top factors that influenced this decision:**")

                    for i, (feature, shap_val) in enumerate(result["top_factors"][:5], 1):
                        direction = "supports" if shap_val > 0 else "hinders"
                        emoji = "📈" if shap_val > 0 else "📉"
                        impact_level = (
                            "high"
                            if abs(shap_val) > 0.5
                            else "medium" if abs(shap_val) > 0.2 else "low"
                        )

                        st.markdown(
                            f"{i}. {emoji} **{feature}**: {direction} scheduling "
                            f"(impact {impact_level}: `{shap_val:+.3f}`)"
                        )

                    st.divider()

                    # SHAP visualization
                    st.subheader("📈 Visualization of influential factors")

                    # Create plot
                    fig = plot_feature_impact(result["feature_contributions"], top_n=10)
                    st.pyplot(fig)

                    st.caption(
                        "🔵 Positive values (blue) support scheduling. "
                        "🔴 Negative values (red) make scheduling harder."
                    )

                    st.divider()

                    # Recomendaciones
                    st.subheader("💡 Recommendations")

                    for rec in result["recommendations"]:
                        st.markdown(rec)

                    st.divider()

                    # Acciones adicionales
                    st.subheader("🔧 Additional actions")

                    col1, col2 = st.columns(2)

                    with col1:
                        if st.button("📥 Download JSON report"):
                            report = {
                                "block_id": block_id,
                                "prediction": result["decision"],
                                "probability": float(result["probability"]),
                                "top_factors": [
                                    {"feature": f, "impact": float(v)}
                                    for f, v in result["top_factors"]
                                ],
                                "recommendations": result["recommendations"],
                            }

                            st.download_button(
                                label="Download JSON",
                                data=json.dumps(report, indent=2),
                                file_name=f"analysis_{block_id}.json",
                                mime="application/json",
                            )

                    with col2:
                        st.info(
                            "🚧 More capabilities coming soon:\n- Compare with similar cases\n- Adjustment simulator\n- Drift monitoring"
                        )

                except Exception as e:
                    st.error(f"❌ Error during analysis: {str(e)}")
                    logger.exception("Analysis error")

                    with st.expander("🔍 Error details"):
                        st.code(str(e))

    # No explicit else: the search section already shows messages when nothing is selected


if __name__ == "__main__":
    render()
