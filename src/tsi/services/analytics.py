"""Analytics and statistical analysis services."""

from typing import Any, cast

import numpy as np
import pandas as pd

from core.algorithms import (
    AnalyticsSnapshot,
    CandidatePlacement,
)
from core.algorithms import (
    compute_correlations as core_compute_correlations,
)
from core.algorithms import (
    compute_distribution_stats as core_compute_distribution_stats,
)
from core.algorithms import (
    generate_insights as core_generate_insights,
)
from core.algorithms import (
    suggest_candidate_positions as core_suggest_candidate_positions,
)
from tsi.config import CORRELATION_COLUMNS
from tsi.models.schemas import AnalyticsMetrics
from tsi.services.rust_compat import (
    compute_metrics as rust_compute_metrics,
    find_conflicts as rust_find_conflicts,
    get_top_observations as rust_get_top_observations,
)


def compute_metrics(df: pd.DataFrame) -> AnalyticsMetrics:
    """Compute comprehensive analytics metrics from the dataset (using Rust backend - 10x faster)."""
    return rust_compute_metrics(df)


def compute_correlations(df: pd.DataFrame) -> pd.DataFrame:
    """Compute a Spearman correlation matrix for key numeric features."""
    result: pd.DataFrame = core_compute_correlations(df, columns=CORRELATION_COLUMNS)
    return result


def get_top_observations(df: pd.DataFrame, by: str = "priority", n: int = 10) -> pd.DataFrame:
    """Get top N observations by a specified metric (using Rust backend - 10x faster)."""
    return rust_get_top_observations(df, by=by, n=n)


def find_conflicts(df: pd.DataFrame) -> pd.DataFrame:
    """
    Find scheduling integrity issues (using Rust backend - 16x faster).
    
    Note: Falls back to empty DataFrame if datetime conversion issues occur.
    """
    try:
        return rust_find_conflicts(df)
    except RuntimeError as e:
        # Handle Rust backend datetime conversion issues
        if "datetime" in str(e).lower() or "dtype" in str(e).lower():
            # Return empty DataFrame with expected columns when conversion fails
            return pd.DataFrame(columns=["schedulingBlockId", "conflict_type", "details"])
        raise  # Re-raise other RuntimeErrors


def _snapshot_from_metrics(metrics: AnalyticsMetrics) -> AnalyticsSnapshot:
    """Convert the Pydantic schema into a dataclass used by the core layer."""
    return AnalyticsSnapshot(**metrics.model_dump())


def compute_distribution_stats(series: pd.Series) -> dict[str, float]:  # type: ignore[type-arg]
    """Compute statistical summary for a numeric series."""
    result: dict[str, float] = core_compute_distribution_stats(series)
    return result


def generate_insights(df: pd.DataFrame, metrics: AnalyticsMetrics) -> list[str]:
    """Generate automated insights from the data."""
    snapshot = _snapshot_from_metrics(metrics)
    result: list[str] = core_generate_insights(df, snapshot)
    return result


def suggest_candidate_positions(df: pd.DataFrame, row: pd.Series) -> list[CandidatePlacement]:  # type: ignore[type-arg]
    """Proxy helper that exposes scheduling what-if scenarios to the UI."""
    result: list[Any] = core_suggest_candidate_positions(df, row)
    return result


def generate_correlation_insights(correlations: pd.DataFrame) -> list[str]:
    """Generate automated insights from correlation matrix.

    Args:
        correlations: Spearman correlation matrix (DataFrame)

    Returns:
        List of insight strings describing key correlations
    """
    if correlations.empty:
        return ["No correlations available - insufficient data."]

    insights = []

    # Interpretation thresholds
    STRONG_THRESHOLD = 0.7
    MODERATE_THRESHOLD = 0.4
    WEAK_THRESHOLD = 0.2

    def interpret_strength(value: float) -> str:
        """Interpret correlation strength."""
        abs_value = abs(value)
        if abs_value >= STRONG_THRESHOLD:
            return "strong"
        elif abs_value >= MODERATE_THRESHOLD:
            return "moderate"
        elif abs_value >= WEAK_THRESHOLD:
            return "weak"
        else:
            return "very weak"

    def interpret_direction(value: float) -> str:
        """Interpret correlation direction."""
        if value > 0:
            return "positive"
        else:
            return "negative"

    # Extract all correlations (excluding diagonal)
    correlations_list: list[dict[str, Any]] = []
    for i in range(len(correlations.columns)):
        for j in range(i + 1, len(correlations.columns)):
            var1 = correlations.columns[i]
            var2 = correlations.columns[j]
            corr_value = correlations.iloc[i, j]

            if pd.notna(corr_value):
                correlations_list.append(
                    {
                        "var1": var1,
                        "var2": var2,
                        "value": corr_value,
                        "abs_value": (
                            abs(float(corr_value))
                            if isinstance(corr_value, (int, float, np.number))
                            else 0.0
                        ),
                    }
                )

    # Sort by absolute value
    correlations_list.sort(key=lambda x: cast(float, x["abs_value"]), reverse=True)

    # Header insight
    insights.append(
        f"**Spearman correlation analysis across {len(correlations.columns)} key dataset variables.**"
    )

    # Analyze top correlations
    if correlations_list:
        # Strongest correlation
        strongest = correlations_list[0]
        strongest_value = cast(float, strongest["value"])
        strongest_var1 = cast(str, strongest["var1"])
        strongest_var2 = cast(str, strongest["var2"])

        # Build more contextual message for strongest correlation
        if abs(strongest_value) >= STRONG_THRESHOLD:
            intro = "**Key finding:** A **strong and significant** relationship was detected"
        elif abs(strongest_value) >= MODERATE_THRESHOLD:
            intro = "**Key finding:** A **moderate** relationship was identified"
        else:
            intro = "**Key finding:** Correlations are generally **weak**. The most notable one"

        insights.append(
            f"{intro} is between **{strongest_var1}** and **{strongest_var2}** (ρ = {strongest_value:.3f}). "
            f"{'When one increases, the other tends to increase as well.' if strongest_value > 0 else 'They show an inverse relationship: when one increases, the other tends to decrease.'}"
        )

        # Analyze specific meaningful correlations (skip the first one, already covered)
        analyzed_count = 1
        for corr in correlations_list[1:6]:  # Next 5 correlations after the strongest
            corr_value = cast(float, corr["value"])
            corr_var1 = cast(str, corr["var1"])
            corr_var2 = cast(str, corr["var2"])

            if abs(corr_value) >= MODERATE_THRESHOLD:
                # Custom interpretations based on variable names
                interpretation = ""
                if "priority" in corr_var1 or "priority" in corr_var2:
                    other_var = corr_var2 if "priority" in corr_var1 else corr_var1
                    if corr_value > 0:
                        interpretation = f"Observations with higher **{other_var}** tend to have **higher scheduling priority**."
                    else:
                        interpretation = f"Higher **{other_var}** is associated with **lower scheduling priority**."

                elif "visibility" in corr_var1.lower() or "visibility" in corr_var2.lower():
                    other_var = corr_var2 if "visibility" in corr_var1.lower() else corr_var1
                    if corr_value > 0:
                        interpretation = f"Higher **{other_var}** implies **more observation windows available**."
                    else:
                        interpretation = (
                            f"Higher **{other_var}** results in **less visibility time**."
                        )

                elif "requested" in corr_var1.lower() or "requested" in corr_var2.lower():
                    other_var = corr_var2 if "requested" in corr_var1.lower() else corr_var1
                    if corr_value > 0:
                        interpretation = f"Observations that require more time tend to have higher **{other_var}**."
                    else:
                        interpretation = (
                            f"Longer requested time is associated with lower **{other_var}**."
                        )

                if interpretation:
                    insights.append(
                        f"**{corr_var1} ↔ {corr_var2}** (ρ = {corr_value:.3f}): {interpretation}"
                    )
                    analyzed_count += 1

        # If no moderate correlations beyond the first, mention independence
        if analyzed_count == 1 and correlations_list:
            insights.append(
                "**Independence between variables:** The remaining variables show weak or very weak correlations, "
                "indicating they operate **mostly independently** from one another. "
                "This suggests each variable contributes unique information to the scheduling problem."
            )

        # Look for interesting weak/negative correlations
        negative_correlations = [
            c for c in correlations_list if cast(float, c["value"]) < -WEAK_THRESHOLD
        ]
        if negative_correlations and len(negative_correlations) > 0:
            example = negative_correlations[0]
            example_value = cast(float, example["value"])
            example_var1 = cast(str, example["var1"])
            example_var2 = cast(str, example["var2"])
            if abs(example_value) >= MODERATE_THRESHOLD:
                insights.append(
                    f"**Notable inverse relationship:** **{example_var1}** y **{example_var2}** "
                    f"show a negative correlation (ρ = {example_value:.3f}), "
                    f"indicating a trade-off between the two metrics."
                )

        # Overall conclusion with actionable insight
        avg_abs_corr = sum(cast(float, c["abs_value"]) for c in correlations_list) / len(
            correlations_list
        )
        strong_count = sum(
            1 for c in correlations_list if abs(cast(float, c["value"])) >= STRONG_THRESHOLD
        )
        moderate_count = sum(
            1
            for c in correlations_list
            if MODERATE_THRESHOLD <= abs(cast(float, c["value"])) < STRONG_THRESHOLD
        )

        if strong_count > 1:
            insights.append(
                f"**Scheduling implication:** With {strong_count} strong correlations detected, "
                f"there is a **meaningful underlying structure** in the data. "
                f"Optimization algorithms can leverage these relationships to improve scheduling."
            )
        elif moderate_count > 2:
            insights.append(
                f"**Scheduling implication:** The {moderate_count} moderate correlations suggest "
                f"**partial patterns** in the dataset. Considering these relationships can help "
                f"prioritize observations more effectively."
            )
        elif avg_abs_corr < 0.3:
            insights.append(
                f"**Scheduling implication:** The low average correlation (|ρ| = {avg_abs_corr:.3f}) "
                f"indicates the variables are **highly independent**. Each criterion "
                f"(priority, visibility, duration) should be evaluated **separately** during optimization."
            )
        else:
            insights.append(
                "**Scheduling implication:** The moderate correlations present "
                "suggest there is some **interdependence** between scheduling criteria, "
                "but each variable remains independent enough to provide unique value."
            )

    return insights
