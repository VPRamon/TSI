"""
Visualization functions for scheduling trends analysis.

Supports Altair (default) and Plotly for interactive charts.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Literal

import pandas as pd

if TYPE_CHECKING:
    from altair import Chart
    from plotly.graph_objects import Figure

try:
    import altair as alt

    ALTAIR_AVAILABLE = True
except ImportError:
    ALTAIR_AVAILABLE = False
    alt = None  # type: ignore[assignment]

try:
    import plotly.graph_objects as go

    PLOTLY_AVAILABLE = True
except ImportError:
    PLOTLY_AVAILABLE = False
    go = None  # type: ignore[assignment]


def bar_rate_by_priority(
    df: pd.DataFrame,
    priority_col: str = "priority",
    rate_col: str = "scheduled_rate",
    n_col: str = "n",
    library: Literal["altair", "plotly"] = "altair",
    title: str = "Scheduling rate by priority",
) -> Chart | Figure:
    """
    Create bar chart of scheduling rate by priority.

    Args:
        df: DataFrame with rates by priority
        priority_col: Priority column
        rate_col: Rate column (0-1)
        n_col: Sample size column
        library: 'altair' or 'plotly'
        title: Chart title

    Returns:
        Altair or Plotly chart

    Raises:
        ValueError: If requested library is not available
    """
    if library == "altair":
        if not ALTAIR_AVAILABLE:
            raise ValueError("Altair is not installed")

        # Convert rate to percentage
        df_plot = df.copy()
        df_plot["rate_pct"] = df_plot[rate_col] * 100

        chart = (
            alt.Chart(df_plot)
            .mark_bar()
            .encode(
                x=alt.X(f"{priority_col}:Q", title="Priority"),
                y=alt.Y("rate_pct:Q", title="% Scheduled", scale=alt.Scale(domain=[0, 100])),
                tooltip=[
                    alt.Tooltip(f"{priority_col}:Q", title="Priority"),
                    alt.Tooltip("rate_pct:Q", title="% Scheduled", format=".1f"),
                    alt.Tooltip(f"{n_col}:Q", title="Observations"),
                ],
                color=alt.Color("rate_pct:Q", scale=alt.Scale(scheme="viridis"), legend=None),
            )
            .properties(title=title, width=600, height=400)
        )

        return chart

    elif library == "plotly":
        if not PLOTLY_AVAILABLE:
            raise ValueError("Plotly is not installed")

        df_plot = df.copy()
        df_plot["rate_pct"] = df_plot[rate_col] * 100

        fig = go.Figure(
            data=[
                go.Bar(
                    x=df_plot[priority_col],
                    y=df_plot["rate_pct"],
                    text=df_plot["rate_pct"].apply(lambda x: f"{x:.1f}%"),
                    textposition="outside",
                    marker=dict(
                        color=df_plot["rate_pct"],
                        colorscale="Viridis",
                        showscale=False,
                    ),
                    hovertemplate=(
                        "<b>Priority:</b> %{x}<br>"
                        "<b>% Scheduled:</b> %{y:.1f}%<br>"
                        "<b>Observations:</b> %{customdata[0]}<br>"
                        "<extra></extra>"
                    ),
                    customdata=df_plot[[n_col]].values,
                )
            ]
        )

        fig.update_layout(
            title=title,
            xaxis_title="Priority",
            yaxis_title="% Scheduled",
            yaxis=dict(range=[0, 105]),
            template="plotly_white",
        )

        return fig

    else:
        raise ValueError(f"Unsupported library: {library}")


def loess_trend(
    df: pd.DataFrame,
    x_col: str = "x",
    y_col: str = "y_smoothed",
    n_col: str = "n_samples",
    library: Literal["altair", "plotly"] = "altair",
    title: str = "Smoothed trend",
    x_label: str = "X",
    y_label: str = "Scheduling rate",
) -> Chart | Figure:
    """
    Create smoothed trend chart (LOESS or equivalent).

    Args:
        df: DataFrame with smoothed data
        x_col: X column
        y_col: Smoothed Y column
        n_col: Sample size column
        library: 'altair' or 'plotly'
        title: Chart title
        x_label: X axis label
        y_label: Y axis label

    Returns:
        Altair or Plotly chart
    """
    if library == "altair":
        if not ALTAIR_AVAILABLE:
            raise ValueError("Altair is not installed")

        df_plot = df.copy()
        df_plot["rate_pct"] = df_plot[y_col] * 100

        chart = (
            alt.Chart(df_plot)
            .mark_line(strokeWidth=3, color="#1f77b4")
            .encode(
                x=alt.X(f"{x_col}:Q", title=x_label),
                y=alt.Y("rate_pct:Q", title=y_label + " (%)", scale=alt.Scale(domain=[0, 100])),
                tooltip=[
                    alt.Tooltip(f"{x_col}:Q", title=x_label, format=".2f"),
                    alt.Tooltip("rate_pct:Q", title=y_label + " (%)", format=".1f"),
                    alt.Tooltip(f"{n_col}:Q", title="Sample"),
                ],
            )
            .properties(title=title, width=600, height=400)
        )

        return chart

    elif library == "plotly":
        if not PLOTLY_AVAILABLE:
            raise ValueError("Plotly is not installed")

        df_plot = df.copy()
        df_plot["rate_pct"] = df_plot[y_col] * 100

        fig = go.Figure(
            data=[
                go.Scatter(
                    x=df_plot[x_col],
                    y=df_plot["rate_pct"],
                    mode="lines",
                    line=dict(color="#1f77b4", width=3),
                    hovertemplate=(
                        f"<b>{x_label}:</b> %{{x:.2f}}<br>"
                        f"<b>{y_label}:</b> %{{y:.1f}}%<br>"
                        "<b>Sample:</b> %{customdata[0]}<br>"
                        "<extra></extra>"
                    ),
                    customdata=df_plot[[n_col]].values,
                )
            ]
        )

        fig.update_layout(
            title=title,
            xaxis_title=x_label,
            yaxis_title=y_label + " (%)",
            yaxis=dict(range=[0, 105]),
            template="plotly_white",
        )

        return fig

    else:
        raise ValueError(f"Unsupported library: {library}")


def heatmap_visibility_priority(
    df: pd.DataFrame,
    visibility_col: str = "total_visibility_hours",
    priority_col: str = "priority",
    target_col: str = "scheduled_flag",
    n_bins_vis: int = 10,
    n_bins_priority: int = 10,
    library: Literal["altair", "plotly"] = "altair",
    title: str = "Heatmap: Visibility × Priority",
) -> Chart | Figure:
    """
    Create 2D heatmap of scheduling rate.

    Args:
        df: DataFrame with data
        visibility_col: Visibility column
        priority_col: Priority column
        target_col: Target column (0/1)
        n_bins_vis: Bins for visibility
        n_bins_priority: Bins for priority
        library: 'altair' or 'plotly'
        title: Chart title

    Returns:
        Altair or Plotly chart
    """
    # Create bins
    df_temp = df.copy()
    df_temp["vis_bin"] = pd.cut(df_temp[visibility_col], bins=n_bins_vis)
    df_temp["priority_bin"] = pd.cut(df_temp[priority_col], bins=n_bins_priority)

    # Aggregate
    heatmap_data = (
        df_temp.groupby(["vis_bin", "priority_bin"], observed=False)
        .agg(
            scheduled_rate=(target_col, lambda x: x.astype(int).mean()),
            n=(target_col, "count"),
            vis_mid=(visibility_col, "mean"),
            priority_mid=(priority_col, "mean"),
        )
        .reset_index()
    )

    # Remove empty bins
    heatmap_data = heatmap_data[heatmap_data["n"] > 0]

    if library == "altair":
        if not ALTAIR_AVAILABLE:
            raise ValueError("Altair is not installed")

        heatmap_data["rate_pct"] = heatmap_data["scheduled_rate"] * 100

        chart = (
            alt.Chart(heatmap_data)
            .mark_rect()
            .encode(
                x=alt.X("vis_mid:Q", title="Visibility (hours)", bin=alt.Bin(maxbins=n_bins_vis)),
                y=alt.Y(
                    "priority_mid:Q",
                    title="Priority",
                    bin=alt.Bin(maxbins=n_bins_priority),
                ),
                color=alt.Color(
                    "rate_pct:Q",
                    scale=alt.Scale(scheme="viridis", domain=[0, 100]),
                    legend=alt.Legend(title="% Scheduled"),
                ),
                tooltip=[
                    alt.Tooltip("vis_mid:Q", title="Visibility", format=".1f"),
                    alt.Tooltip("priority_mid:Q", title="Priority", format=".1f"),
                    alt.Tooltip("rate_pct:Q", title="% Scheduled", format=".1f"),
                    alt.Tooltip("n:Q", title="Observations"),
                ],
            )
            .properties(title=title, width=500, height=400)
        )

        return chart

    elif library == "plotly":
        if not PLOTLY_AVAILABLE:
            raise ValueError("Plotly is not installed")

        # Create pivot matrix for heatmap
        pivot = heatmap_data.pivot_table(
            index="priority_mid",
            columns="vis_mid",
            values="scheduled_rate",
            aggfunc="mean",
        )

        pivot_pct = pivot * 100

        fig = go.Figure(
            data=go.Heatmap(
                z=pivot_pct.values,
                x=pivot_pct.columns,
                y=pivot_pct.index,
                colorscale="Viridis",
                colorbar=dict(title="% Scheduled"),
                hovertemplate=(
                    "<b>Visibility:</b> %{x:.1f} hours<br>"
                    "<b>Priority:</b> %{y:.1f}<br>"
                    "<b>% Scheduled:</b> %{z:.1f}%<br>"
                    "<extra></extra>"
                ),
            )
        )

        fig.update_layout(
            title=title,
            xaxis_title="Visibility (hours)",
            yaxis_title="Priority",
            template="plotly_white",
        )

        return fig

    else:
        raise ValueError(f"Unsupported library: {library}")


def pred_curve_vs_visibility(
    df_pred: pd.DataFrame,
    visibility_col: str = "total_visibility_hours",
    priority_col: str = "priority",
    prob_col: str = "scheduled_prob",
    library: Literal["altair", "plotly"] = "altair",
    title: str = "Estimated probability vs Visibility",
    fixed_time: float | None = None,
) -> Chart | Figure:
    """
    Create chart of estimated probability vs visibility, colored by priority.

    Args:
        df_pred: DataFrame with predictions
        visibility_col: Visibility column
        priority_col: Priority column
        prob_col: Estimated probability column
        library: 'altair' or 'plotly'
        title: Chart title
        fixed_time: Fixed requested_time value (to show in title)

    Returns:
        Altair or Plotly chart
    """
    if fixed_time is not None:
        title = f"{title} (requested time = {fixed_time:.1f}h)"

    if library == "altair":
        if not ALTAIR_AVAILABLE:
            raise ValueError("Altair is not installed")

        df_plot = df_pred.copy()
        df_plot["prob_pct"] = df_plot[prob_col] * 100

        # Convert priority to string for coloring
        df_plot[f"{priority_col}_str"] = df_plot[priority_col].astype(str)

        chart = (
            alt.Chart(df_plot)
            .mark_line(strokeWidth=2)
            .encode(
                x=alt.X(f"{visibility_col}:Q", title="Visibility (hours)"),
                y=alt.Y(
                    "prob_pct:Q",
                    title="Scheduling probability (%)",
                    scale=alt.Scale(domain=[0, 100]),
                ),
                color=alt.Color(
                    f"{priority_col}_str:N",
                    title="Priority",
                    scale=alt.Scale(scheme="category10"),
                ),
                tooltip=[
                    alt.Tooltip(f"{visibility_col}:Q", title="Visibility", format=".2f"),
                    alt.Tooltip("prob_pct:Q", title="Prob. (%)", format=".1f"),
                    alt.Tooltip(f"{priority_col}:Q", title="Priority"),
                ],
            )
            .properties(title=title, width=600, height=400)
        )

        return chart

    elif library == "plotly":
        if not PLOTLY_AVAILABLE:
            raise ValueError("Plotly is not installed")

        df_plot = df_pred.copy()
        df_plot["prob_pct"] = df_plot[prob_col] * 100

        fig = go.Figure()

        # Line for each priority level
        for priority in sorted(df_plot[priority_col].unique()):
            df_subset = df_plot[df_plot[priority_col] == priority]

            fig.add_trace(
                go.Scatter(
                    x=df_subset[visibility_col],
                    y=df_subset["prob_pct"],
                    mode="lines",
                    name=f"Priority {priority}",
                    line=dict(width=2),
                    hovertemplate=(
                        "<b>Visibility:</b> %{x:.2f} hours<br>"
                        "<b>Probability:</b> %{y:.1f}%<br>"
                        f"<b>Priority:</b> {priority}<br>"
                        "<extra></extra>"
                    ),
                )
            )

        fig.update_layout(
            title=title,
            xaxis_title="Visibility (hours)",
            yaxis_title="Scheduling probability (%)",
            yaxis=dict(range=[0, 105]),
            template="plotly_white",
            legend=dict(title="Priority"),
        )

        return fig

    else:
        raise ValueError(f"Unsupported library: {library}")
