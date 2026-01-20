"""Timeline (Gantt-style) plotting for visibility and scheduling."""

import plotly.graph_objects as go

from tsi.config import PLOT_HEIGHT
from tsi.plots.plot_theme import PlotTheme, apply_theme, get_colorscale_config


def build_visibility_histogram_from_bins(
    bins: list[dict],
    total_blocks: int,
    bin_duration_minutes: float,
) -> go.Figure:
    """
    Build histogram from pre-computed backend bins.

    This function creates a Plotly visualization from bins computed by the Rust backend,
    eliminating the need to load and process the full dataframe in Python.

    Args:
        bins: List of dicts with keys 'bin_start_unix', 'bin_end_unix', 'count'
        total_blocks: Total number of blocks in the dataset (for title)
        bin_duration_minutes: Duration of each bin in minutes (for title)

    Returns:
        Plotly Figure object with histogram
    """
    if len(bins) == 0:
        fig = go.Figure()
        apply_theme(fig, show_legend=False)
        fig.update_layout(title="No bins to display")
        return fig

    # Convert Unix timestamps to datetimes for plotting
    from datetime import datetime, timezone

    bin_starts = []
    bin_counts = []
    bin_widths = []

    for bin_data in bins:
        bin_start_unix = bin_data["bin_start_unix"]
        bin_end_unix = bin_data["bin_end_unix"]
        count = bin_data["count"]

        # Use bin start time as x position
        start_dt = datetime.fromtimestamp(bin_start_unix, tz=timezone.utc)
        bin_starts.append(start_dt)
        bin_counts.append(count)

        # Calculate bin width in milliseconds for Plotly
        bin_width_ms = (bin_end_unix - bin_start_unix) * 1000
        bin_widths.append(bin_width_ms)

    # Get colorscale configuration
    colorscale_config = get_colorscale_config(
        colorscale="Viridis",
        colorbar_title="Number of<br>Visible Blocks",
    )

    # Create the histogram using bar chart with explicit width
    fig = go.Figure()

    fig.add_trace(
        go.Bar(
            x=bin_starts,
            y=bin_counts,
            width=bin_widths,
            name="Visible Targets",
            marker=dict(
                color=bin_counts,
                colorscale=colorscale_config["colorscale"],
                colorbar=colorscale_config["colorbar"],
                line=dict(
                    width=0.5, color="rgba(255, 255, 255, 0.15)"
                ),  # Subtle border between bars
            ),
            hovertemplate=(
                "<b>%{y} visible blocks</b><br>" "Time: %{x|%Y-%m-%d %H:%M}<br>" "<extra></extra>"
            ),
        )
    )

    # Human-readable bin duration for title
    if bin_duration_minutes >= 24 * 60:
        duration_label = f"{bin_duration_minutes / (24 * 60):.1f} day(s)"
    elif bin_duration_minutes >= 60:
        duration_label = f"{bin_duration_minutes / 60:.1f} hour(s)"
    else:
        duration_label = f"{bin_duration_minutes:.1f} minute(s)"

    num_bins = len(bins)

    # Apply standard theme with no legend (colorbar serves as legend)
    apply_theme(
        fig,
        height=PLOT_HEIGHT,
        margin=dict(l=80, r=80, t=100, b=80),
        show_legend=False,
    )

    fig.update_layout(
        title=(
            "Target Visibility Over Time "
            f"({total_blocks:,} total blocks, {num_bins} bins, ~{duration_label} per bin)"
        ),
        xaxis=dict(
            title="Observation Period (UTC)",
            showgrid=True,
            gridcolor=PlotTheme.GRID_COLOR,
            type="date",
        ),
        yaxis=dict(
            title="Number of Visible Blocks",
            showgrid=True,
            gridcolor=PlotTheme.GRID_COLOR,
        ),
        bargap=0,  # No gap between bars
        hovermode="x unified",
    )

    return fig
