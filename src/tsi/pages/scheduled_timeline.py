"""Scheduled Timeline - Monthly view of scheduled observations."""

import pandas as pd
import plotly.graph_objects as go
import streamlit as st

from core.time import format_datetime_utc
from tsi import state
from tsi.components.toolbar import (
    render_number_input,
    render_priority_filter,
    render_reset_filters_button,
)


def render() -> None:
    """Render the Scheduled Timeline page."""
    st.title("🗂️ Scheduled Timeline")

    st.markdown(
        """
        Monthly chronological view of scheduled observations. Each row represents a month
        and the horizontal axis shows the days (1-31) of that specific month, avoiding visual
        overflow. Colors represent the priority of each block (darker = higher priority).
        """
    )

    df = state.get_prepared_data()
    dark_periods_df = state.get_dark_periods()

    if df is None:
        st.warning("No data loaded. Please return to the landing page.")
        return

    # Check if there are any scheduled observations
    if "scheduled_flag" not in df.columns or not df["scheduled_flag"].any():
        st.warning("There are no scheduled observations in the dataset.")
        return

    # Filter only scheduled observations with valid datetime fields
    scheduled_df = df[
        (df["scheduled_flag"])
        & (df["scheduled_start_dt"].notna())
        & (df["scheduled_stop_dt"].notna())
    ].copy()

    if len(scheduled_df) == 0:
        st.warning("There are no scheduled observations with valid dates.")
        return

    # Add auxiliary columns for monthly grouping
    # Remove timezone info before converting to Period to avoid warnings
    scheduled_df["scheduled_month"] = (
        scheduled_df["scheduled_start_dt"].dt.tz_localize(None).dt.to_period("M")
    )
    scheduled_df["scheduled_month_label"] = scheduled_df["scheduled_start_dt"].dt.strftime("%Y-%m")
    scheduled_df["duration_hours"] = (
        scheduled_df["scheduled_stop_dt"] - scheduled_df["scheduled_start_dt"]
    ).dt.total_seconds() / 3600.0

    # Calculate priority range from scheduled data
    if "priority" in scheduled_df.columns:
        priority_values = scheduled_df["priority"].dropna()
        if not priority_values.empty:
            priority_min = float(priority_values.min())
            priority_max = float(priority_values.max())
            if priority_min == priority_max:
                priority_max = priority_min + 1.0
        else:
            priority_min, priority_max = 0.0, 10.0
    else:
        priority_min, priority_max = 0.0, 10.0

    # Get unique months sorted chronologically
    all_months = sorted(scheduled_df["scheduled_month_label"].unique())

    # Simple filters - use default values
    priority_range = (priority_min, priority_max)
    selected_months = all_months
    show_short_blocks = False
    duration_threshold = 0.0

    # Apply filters
    filtered_df = scheduled_df[
        (scheduled_df["priority"] >= priority_range[0])
        & (scheduled_df["priority"] <= priority_range[1])
    ]

    if selected_months:
        filtered_df = filtered_df[filtered_df["scheduled_month_label"].isin(selected_months)]

    if show_short_blocks and duration_threshold > 0:
        filtered_df = filtered_df[filtered_df["duration_hours"] >= duration_threshold]

    if len(filtered_df) == 0:
        st.warning("There are no scheduled observations for the selected criteria.")
        return

    # Build the monthly timeline figure
    # Always include dark periods if available - users can toggle them in the plot legend
    filtered_dark_periods = None
    if dark_periods_df is not None and not dark_periods_df.empty:
        filtered_dark_periods = dark_periods_df.copy()
        if selected_months:
            filtered_dark_periods = filtered_dark_periods[
                filtered_dark_periods["months"].apply(
                    lambda month_list: any(month in selected_months for month in month_list)
                )
            ]
        if filtered_dark_periods.empty:
            filtered_dark_periods = None

    fig = build_monthly_timeline(filtered_df, priority_range, filtered_dark_periods)

    # Display the chart with configuration to limit pan range
    # Disable pan on X axis to prevent moving beyond day boundaries
    # Users can still zoom in/out
    config = {
        "displayModeBar": True,
        "displaylogo": False,
        "modeBarButtonsToRemove": ["lasso2d", "select2d"],
        "scrollZoom": True,  # Enable zoom with scroll wheel
    }
    st.plotly_chart(fig, width='stretch', config=config)

    # Show dark period summary if available
    if filtered_dark_periods is not None:
        st.markdown("---")
        st.subheader("ℹ️ Observable periods information")

        dark_count = len(filtered_dark_periods)
        total_dark_hours = filtered_dark_periods["duration_hours"].sum()
        min_dark = filtered_dark_periods["start_dt"].min()
        max_dark = filtered_dark_periods["stop_dt"].max()

        st.caption(
            f"Detected {dark_count:,} dark/nocturnal OBSERVABLE periods (total {total_dark_hours:,.1f} h). "
            f"The chart shows DAYTIME periods (non-observable) in light yellow."
            f" Time range: {format_datetime_utc(min_dark)} → {format_datetime_utc(max_dark)}."
        )

        dark_display = filtered_dark_periods.copy()
        dark_display["Start"] = dark_display["start_dt"].dt.strftime("%Y-%m-%d %H:%M")
        dark_display["End"] = dark_display["stop_dt"].dt.strftime("%Y-%m-%d %H:%M")
        dark_display = dark_display[["Start", "End", "duration_hours", "months"]]
        dark_display = dark_display.rename(
            columns={"duration_hours": "Duration (h)", "months": "Months"}
        )

        st.dataframe(
            dark_display,
            width='stretch',
            hide_index=True,
            height=min(300, 60 + 24 * min(len(dark_display), 8)),
        )

    # Add detailed information table
    st.markdown("---")
    st.subheader("📊 Observation Details")

    # Prepare data for display with more comprehensive information
    display_columns = [
        "schedulingBlockId",
        "scheduled_month_label",
        "priority",
        "duration_hours",
        "scheduled_start_dt",
        "scheduled_stop_dt",
    ]

    # Add optional columns if they exist
    optional_columns = [
        "raInDeg",
        "decInDeg",
        "requested_hours",
        "total_visibility_hours",
        "num_visibility_periods",
    ]

    for col in optional_columns:
        if col in filtered_df.columns:
            display_columns.append(col)

    display_df = filtered_df[display_columns].copy()

    # Add day information
    display_df["start_day"] = display_df["scheduled_start_dt"].dt.day
    display_df["end_day"] = display_df["scheduled_stop_dt"].dt.day
    display_df["start_time"] = display_df["scheduled_start_dt"].dt.strftime("%H:%M")
    display_df["end_time"] = display_df["scheduled_stop_dt"].dt.strftime("%H:%M")

    # Rename columns for better display
    column_renames = {
        "schedulingBlockId": "Block ID",
        "scheduled_month_label": "Month",
        "priority": "Priority",
        "duration_hours": "Duration (h)",
        "scheduled_start_dt": "Start Date",
        "scheduled_stop_dt": "End Date",
        "start_day": "Day",
        "end_day": "End Day",
        "start_time": "Start Time",
        "end_time": "End Time",
        "raInDeg": "RA (°)",
        "decInDeg": "Dec (°)",
        "requested_hours": "Requested (h)",
        "total_visibility_hours": "Total Visibility (h)",
        "num_visibility_periods": "# Vis. Periods",
    }

    display_df = display_df.rename(columns=column_renames)

    # Reorder columns
    base_columns = [
        "Block ID",
        "Month",
        "Day",
        "Start Time",
        "End Time",
        "Priority",
        "Duration (h)",
    ]

    # Add optional columns that exist
    extra_columns = []
    for original, renamed in column_renames.items():
        if renamed not in base_columns and renamed in display_df.columns:
            extra_columns.append(renamed)

    display_df = display_df[base_columns + extra_columns]

    # Sort by month and day
    display_df = display_df.sort_values(["Month", "Day"])

    # Add search/filter capability
    col_search1, col_search2, col_search3 = st.columns(3)

    with col_search1:
        search_id = st.text_input(
            "🔍 Search by ID",
            key="timeline_search_id",
            placeholder="e.g., SB001",
        )

    with col_search2:
        search_month = st.selectbox(
            "📅 Month",
            options=["All"] + sorted(filtered_df["scheduled_month_label"].unique().tolist()),
            key="timeline_search_month",
        )

    with col_search3:
        min_priority_filter = st.number_input(
            "⭐ Minimum priority",
            min_value=0.0,
            max_value=10.0,
            value=0.0,
            step=0.5,
            key="timeline_min_priority_filter",
        )

    # Apply search filters
    filtered_display = display_df.copy()

    if search_id:
        filtered_display = filtered_display[
            filtered_display["Block ID"].astype(str).str.contains(search_id, case=False, na=False)
        ]

    if search_month != "All":
        filtered_display = filtered_display[filtered_display["Month"] == search_month]

    if min_priority_filter > 0:
        filtered_display = filtered_display[filtered_display["Priority"] >= min_priority_filter]

    # Display count
    st.caption(f"Showing {len(filtered_display):,} of {len(display_df):,} observations")

    # Display the table with formatting
    st.dataframe(
        filtered_display.style.format(
            {
                "Priority": "{:.2f}",
                "Duration (h)": "{:.2f}",
                "RA (°)": "{:.2f}",
                "Dec (°)": "{:.2f}",
                "Requested (h)": "{:.2f}",
                "Total Visibility (h)": "{:.2f}",
            },
            na_rep="-",
        ),
        width="stretch",
        height=400,
        hide_index=True,
    )

    # Add download button
    csv = filtered_display.to_csv(index=False).encode("utf-8")
    st.download_button(
        label="📥 Download table as CSV",
        data=csv,
        file_name=f"scheduled_timeline_{pd.Timestamp.now().strftime('%Y%m%d_%H%M%S')}.csv",
        mime="text/csv",
        key="timeline_download_csv",
    )

    # Display key metrics
    st.markdown("---")
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Scheduled blocks", f"{len(filtered_df):,}")

    with col2:
        total_hours = filtered_df["duration_hours"].sum()
        st.metric("Total hours", f"{total_hours:,.1f}")

    with col3:
        avg_duration = filtered_df["duration_hours"].mean()
        st.metric("Average duration", f"{avg_duration:.2f} h")

    with col4:
        num_months = filtered_df["scheduled_month_label"].nunique()
        st.metric("Months covered", f"{num_months}")

    # Date range info
    min_date = filtered_df["scheduled_start_dt"].min()
    max_date = filtered_df["scheduled_stop_dt"].max()
    st.caption(f"**Time range:** {format_datetime_utc(min_date)} → {format_datetime_utc(max_date)}")


def build_monthly_timeline(
    df: pd.DataFrame,
    priority_range: tuple[float, float],
    dark_periods: pd.DataFrame | None = None,
) -> go.Figure:
    """
    Build a Plotly figure showing scheduled observations grouped by month.

    Each month is represented as a horizontal row, with observations displayed
    as bars within that month's time range (days 1-31). This prevents overflow
    by constraining each row to its month's temporal boundaries.

    Args:
        df: DataFrame with scheduled observations and monthly grouping columns
        priority_range: (min, max) priority values for color normalization
        dark_periods: Optional DataFrame with dark period windows to overlay

    Returns:
        Plotly Figure object
    """
    # Get sorted months for Y-axis ordering
    scheduled_months = set(df["scheduled_month_label"].unique())
    dark_months: set[str] = set()
    if dark_periods is not None and "months" in dark_periods.columns:
        for month_list in dark_periods["months"]:
            if isinstance(month_list, list):
                dark_months.update(month_list)

    ordered_months = sorted(scheduled_months | dark_months)
    num_months = len(ordered_months)

    # Create figure
    fig = go.Figure()

    # Normalize priority for color mapping
    priority_min, priority_max = priority_range
    if priority_max <= priority_min:
        priority_max = priority_min + 1.0

    # Create a mapping from month_label to numeric position
    month_to_position = {month: idx for idx, month in enumerate(ordered_months)}

    # Add both dark and light period overlays - users can toggle them in the legend
    if dark_periods is not None and not dark_periods.empty:
        # Add light periods (inverted from dark periods) to show non-observable times
        light_segments = _invert_to_light_periods(dark_periods)
        
        light_traces_added = 0
        for month_label, seg_start, seg_stop in light_segments:
            if month_label not in month_to_position:
                continue

            y_pos = month_to_position[month_label]
            y_bottom = y_pos - 0.04  # Narrower band (was 0.45 before)
            y_top = y_pos + 0.04

            x_start = _datetime_to_day_fraction(seg_start)
            x_stop = _datetime_to_day_fraction(seg_stop)
            if x_stop <= x_start:
                continue

            duration_hours = (seg_stop - seg_start).total_seconds() / 3600.0

            # Mark LIGHT periods (daytime, non-observable) with light yellow
            fig.add_trace(
                go.Scatter(
                    name="Daytime periods (non-observable)",
                    x=[x_start, x_stop, x_stop, x_start, x_start],
                    y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                    fill="toself",
                    fillcolor="rgba(255, 230, 180, 0.15)",  # Very soft yellow
                    line=dict(color="rgba(200, 180, 140, 0.3)", width=1.5),  # Subtle outline
                    hovertemplate=(
                        "<b>☀️ Daytime period (non-observable)</b><br><br>"
                        f"<b>Month:</b> {month_label}<br>"
                        f"<b>Start:</b> {format_datetime_utc(seg_start)}<br>"
                        f"<b>End:</b> {format_datetime_utc(seg_stop)}<br>"
                        f"<b>Duration:</b> {duration_hours:.2f} hours"
                        "<extra></extra>"
                    ),
                    legendgroup="light_periods",
                    showlegend=light_traces_added == 0,
                    mode="lines",
                )
            )
            light_traces_added += 1
        
        # Add dark periods (observable times) with dark blue
        dark_segments = _split_dark_periods_by_month(dark_periods)
        
        dark_traces_added = 0
        for month_label, seg_start, seg_stop in dark_segments:
            if month_label not in month_to_position:
                continue

            y_pos = month_to_position[month_label]
            y_bottom = y_pos - 0.04  # Narrower band (consistent with daytime periods)
            y_top = y_pos + 0.04

            x_start = _datetime_to_day_fraction(seg_start)
            x_stop = _datetime_to_day_fraction(seg_stop)
            if x_stop <= x_start:
                continue

            duration_hours = (seg_stop - seg_start).total_seconds() / 3600.0

            # Mark DARK periods (nighttime, observable) with dark blue
            fig.add_trace(
                go.Scatter(
                    name="Nighttime periods (observable)",
                    x=[x_start, x_stop, x_stop, x_start, x_start],
                    y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                    fill="toself",
                    fillcolor="rgba(80, 100, 140, 0.15)",  # Very soft bluish gray
                    line=dict(color="rgba(100, 120, 160, 0.3)", width=1.5),  # Subtle outline
                    hovertemplate=(
                        "<b>🌙 Nighttime period (observable)</b><br><br>"
                        f"<b>Month:</b> {month_label}<br>"
                        f"<b>Start:</b> {format_datetime_utc(seg_start)}<br>"
                        f"<b>End:</b> {format_datetime_utc(seg_stop)}<br>"
                        f"<b>Duration:</b> {duration_hours:.2f} hours"
                        "<extra></extra>"
                    ),
                    legendgroup="dark_periods",
                    showlegend=dark_traces_added == 0,
                    mode="lines",
                )
            )
            dark_traces_added += 1

    # Iterate through each observation and add as a filled rectangle
    # Key: convert datetime to day-of-month (1-31) for X axis
    # Handle observations that span multiple months by splitting them
    for _, row in df.iterrows():
        block_id = row["schedulingBlockId"]
        priority = row["priority"]
        start_dt = row["scheduled_start_dt"]
        stop_dt = row["scheduled_stop_dt"]
        duration_hours = row["duration_hours"]
        month_label = row["scheduled_month_label"]

        # Normalize priority to [0, 1] for colorscale
        normalized_priority = (priority - priority_min) / (priority_max - priority_min)

        # Check if observation spans multiple months
        # Remove timezone info before converting to Period to avoid warnings
        start_month = start_dt.tz_localize(None).to_period("M")
        stop_month = stop_dt.tz_localize(None).to_period("M")

        if start_month != stop_month:
            # Split observation across months
            # Part 1: From start to end of start month
            end_of_start_month = start_dt.tz_localize(None).to_period("M").to_timestamp(how="end")

            # Add segment for the start month
            start_month_label = start_dt.strftime("%Y-%m")
            if start_month_label in month_to_position:
                y_pos = month_to_position[start_month_label]
                y_bottom = y_pos - 0.4
                y_top = y_pos + 0.4

                start_day = start_dt.day + start_dt.hour / 24.0 + start_dt.minute / 1440.0
                # End at midnight of the last day of the month
                end_day = end_of_start_month.day + 23.0 / 24.0 + 59.0 / 1440.0

                fig.add_trace(
                    go.Scatter(
                        name=f"Block {block_id} (part 1)",
                        x=[start_day, end_day, end_day, start_day, start_day],
                        y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                        fill="toself",
                        fillcolor=f"rgba({_viridis_color(normalized_priority)}, 0.85)",
                        line=dict(
                            color=f"rgba({_viridis_color(normalized_priority)}, 1.0)",
                            width=1,
                        ),
                        hovertemplate=(
                            f"<b>📡 Block {block_id}</b> (crosses months)<br><br>"
                            f"<b>Month:</b> {start_month_label}<br>"
                            f"<b>Days:</b> {start_dt.day} → {end_of_start_month.day}<br>"
                            f"<b>Priority:</b> {priority:.2f}<br>"
                            f"<b>Total duration:</b> {duration_hours:.2f} hours<br><br>"
                            f"<b>Start:</b> {format_datetime_utc(start_dt)}<br>"
                            f"<b>Total end:</b> {format_datetime_utc(stop_dt)}<br>"
                            "<extra></extra>"
                        ),
                        showlegend=False,
                        mode="lines",
                        customdata=[[block_id, priority, duration_hours, start_month_label]],
                    )
                )

            # Part 2: From start of stop month to actual stop
            stop_month_label = stop_dt.strftime("%Y-%m")
            if stop_month_label in month_to_position:
                y_pos = month_to_position[stop_month_label]
                y_bottom = y_pos - 0.4
                y_top = y_pos + 0.4

                # Start at day 1
                start_day = 1.0
                stop_day = stop_dt.day + stop_dt.hour / 24.0 + stop_dt.minute / 1440.0

                fig.add_trace(
                    go.Scatter(
                        name=f"Block {block_id} (part 2)",
                        x=[start_day, stop_day, stop_day, start_day, start_day],
                        y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                        fill="toself",
                        fillcolor=f"rgba({_viridis_color(normalized_priority)}, 0.85)",
                        line=dict(
                            color=f"rgba({_viridis_color(normalized_priority)}, 1.0)",
                            width=1,
                        ),
                        hovertemplate=(
                            f"<b>📡 Block {block_id}</b> (crosses months)<br><br>"
                            f"<b>Month:</b> {stop_month_label}<br>"
                            f"<b>Days:</b> 1 → {stop_dt.day}<br>"
                            f"<b>Priority:</b> {priority:.2f}<br>"
                            f"<b>Total duration:</b> {duration_hours:.2f} hours<br><br>"
                            f"<b>Total start:</b> {format_datetime_utc(start_dt)}<br>"
                            f"<b>End:</b> {format_datetime_utc(stop_dt)}<br>"
                            "<extra></extra>"
                        ),
                        showlegend=False,
                        mode="lines",
                        customdata=[[block_id, priority, duration_hours, stop_month_label]],
                    )
                )
        else:
            # Observation within a single month - original logic
            y_pos = month_to_position[month_label]
            y_bottom = y_pos - 0.4
            y_top = y_pos + 0.4

            # Convert datetime to day-of-month with fractional part for time
            # Day 1 = 0:00 of first day, Day 1.5 = 12:00 of first day
            start_day = start_dt.day + start_dt.hour / 24.0 + start_dt.minute / 1440.0
            stop_day = stop_dt.day + stop_dt.hour / 24.0 + stop_dt.minute / 1440.0

            # Create a filled rectangle using Scatter with fill
            fig.add_trace(
                go.Scatter(
                    name=f"Block {block_id}",  # Named trace instead of generic "Trace"
                    x=[start_day, stop_day, stop_day, start_day, start_day],
                    y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                    fill="toself",
                    fillcolor=f"rgba({_viridis_color(normalized_priority)}, 0.85)",
                    line=dict(
                        color=f"rgba({_viridis_color(normalized_priority)}, 1.0)",
                        width=1,
                    ),
                    hovertemplate=(
                        f"<b>📡 Block {block_id}</b><br><br>"
                        f"<b>Month:</b> {month_label}<br>"
                        f"<b>Days:</b> {start_dt.day} → {stop_dt.day}<br>"
                        f"<b>Priority:</b> {priority:.2f}<br>"
                        f"<b>Duration:</b> {duration_hours:.2f} hours<br><br>"
                        f"<b>Start:</b> {format_datetime_utc(start_dt)}<br>"
                        f"<b>End:</b> {format_datetime_utc(stop_dt)}<br>"
                        "<extra></extra>"
                    ),
                    showlegend=False,
                    mode="lines",
                    # Add custom data for potential click events
                    customdata=[[block_id, priority, duration_hours, month_label]],
                )
            )

    # Add a dummy trace for the colorbar legend
    # This creates a visual reference for priority colors
    fig.add_trace(
        go.Scatter(
            x=[None],
            y=[None],
            mode="markers",
            marker=dict(
                colorscale="Viridis",
                cmin=priority_min,
                cmax=priority_max,
                colorbar=dict(
                    title="Priority",
                    thickness=15,
                    len=0.7,
                ),
                showscale=True,
            ),
            hoverinfo="skip",
            showlegend=False,
        )
    )

    # Calculate dynamic height based on number of months
    height = max(600, num_months * 80)

    # Update layout for chronological ordering and dark theme
    fig.update_layout(
        title=f"Scheduled Timeline by Month ({len(df):,} observations)",
        xaxis=dict(
            title="Day of month",
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
            range=[0.5, 31.5],  # Days 1-31 with padding, constrained range
            tickmode="linear",
            tick0=1,
            dtick=1,
            tickformat="d",
            rangemode="normal",
            autorange=False,
            # Allow zoom but we'll add min/max range constraints
            minallowed=0.5,  # Don't allow viewing before day 0.5
            maxallowed=31.5,  # Don't allow viewing after day 31.5
        ),
        yaxis=dict(
            title="Scheduled month",
            tickmode="array",
            tickvals=list(range(num_months)),
            ticktext=ordered_months,
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
            range=[-0.5, num_months - 0.5],  # Add padding around months
        ),
        height=height,
        margin=dict(l=100, r=120, t=80, b=80),
        hovermode="closest",
        dragmode="pan",  # Restore pan mode for better UX
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    # Allow zoom/pan but enforce absolute min/max boundaries
    fig.update_xaxes(
        range=[0.5, 31.5],
        minallowed=0.5,  # Absolute minimum visible value
        maxallowed=31.5,  # Absolute maximum visible value
    )

    # Y axis can be panned/zoomed freely
    fig.update_yaxes(
        fixedrange=False,
    )

    return fig


def _viridis_color(normalized_value: float) -> str:
    """
    Convert a normalized value [0, 1] to an RGB string using Viridis colorscale approximation.

    Args:
        normalized_value: Value between 0 and 1

    Returns:
        RGB values as comma-separated string (e.g., "68, 1, 84")
    """
    # Simplified Viridis colorscale (5 key points)
    # Format: (position, (R, G, B))
    viridis_colors = [
        (0.0, (68, 1, 84)),
        (0.25, (59, 82, 139)),
        (0.5, (33, 145, 140)),
        (0.75, (94, 201, 98)),
        (1.0, (253, 231, 37)),
    ]

    # Clamp value to [0, 1]
    normalized_value = max(0.0, min(1.0, normalized_value))

    # Find the two colors to interpolate between
    for i in range(len(viridis_colors) - 1):
        pos1, color1 = viridis_colors[i]
        pos2, color2 = viridis_colors[i + 1]

        if pos1 <= normalized_value <= pos2:
            # Linear interpolation
            t = (normalized_value - pos1) / (pos2 - pos1) if pos2 != pos1 else 0
            r = int(color1[0] + t * (color2[0] - color1[0]))
            g = int(color1[1] + t * (color2[1] - color1[1]))
            b = int(color1[2] + t * (color2[2] - color1[2]))
            return f"{r}, {g}, {b}"

    # Fallback (should not reach here)
    return "68, 1, 84"


def _split_dark_periods_by_month(
    dark_df: pd.DataFrame,
) -> list[tuple[str, pd.Timestamp, pd.Timestamp]]:
    """Split each dark period by month boundaries for plotting."""

    segments: list[tuple[str, pd.Timestamp, pd.Timestamp]] = []
    for _, row in dark_df.iterrows():
        start_dt = row.get("start_dt")
        stop_dt = row.get("stop_dt")

        if start_dt is None or stop_dt is None:
            continue
        if pd.isna(start_dt) or pd.isna(stop_dt):
            continue

        for month_label, seg_start, seg_stop in _iter_month_segments(start_dt, stop_dt):
            if seg_stop <= seg_start:
                continue
            segments.append((month_label, seg_start, seg_stop))

    return segments


def _invert_to_light_periods(
    dark_df: pd.DataFrame,
) -> list[tuple[str, pd.Timestamp, pd.Timestamp]]:
    """
    Invert dark (observable) periods to obtain light (non-observable) periods.

    Dark periods represent the nighttime windows when observations are possible.
    This function computes the gaps between dark periods, which correspond to the
    daytime windows when observations are NOT possible.

    Returns:
        List of tuples: (month_label, start, stop) for each light period
    """
    if dark_df.empty:
        return []
    
    # Sort by start time
    dark_df_sorted = dark_df.sort_values('start_dt').reset_index(drop=True)
    
    light_segments: list[tuple[str, pd.Timestamp, pd.Timestamp]] = []
    
    # For each month, find the gaps between dark periods
    all_months = set()
    for month_list in dark_df_sorted['months']:
        all_months.update(month_list)
    
    for month_label in sorted(all_months):
        # Get all dark periods for this month
        month_dark = dark_df_sorted[
            dark_df_sorted['months'].apply(lambda x: month_label in x)
        ].copy()
        
        if month_dark.empty:
            continue
        
        # Get month boundaries
        year, month = map(int, month_label.split('-'))
        month_start = pd.Timestamp(year=year, month=month, day=1, hour=0, minute=0, tz='UTC')
        
        # Compute the first instant of the next month (serves as month end boundary)
        if month == 12:
            month_end = pd.Timestamp(year=year + 1, month=1, day=1, hour=0, minute=0, tz='UTC')
        else:
            month_end = pd.Timestamp(year=year, month=month + 1, day=1, hour=0, minute=0, tz='UTC')
        
        # Collect all dark periods of the month in order
        dark_periods_in_month = []
        for _, row in month_dark.iterrows():
            start = max(row['start_dt'], month_start)
            stop = min(row['stop_dt'], month_end)
            if start < stop:
                dark_periods_in_month.append((start, stop))
        
        # Sort by start time
        dark_periods_in_month.sort(key=lambda x: x[0])
        
        if not dark_periods_in_month:
            continue
        
        # Create light periods between dark periods
        # 1. Gap from the start of the month to the first dark period
        first_dark_start = dark_periods_in_month[0][0]
        if month_start < first_dark_start:
            light_segments.append((month_label, month_start, first_dark_start))
        
        # 2. Gaps between consecutive dark periods
        for i in range(len(dark_periods_in_month) - 1):
            current_dark_end = dark_periods_in_month[i][1]
            next_dark_start = dark_periods_in_month[i + 1][0]
            
            if current_dark_end < next_dark_start:
                light_segments.append((month_label, current_dark_end, next_dark_start))
        
        # 3. Gap from the last dark period to the end of the month
        last_dark_end = dark_periods_in_month[-1][1]
        if last_dark_end < month_end:
            light_segments.append((month_label, last_dark_end, month_end))
    
    return light_segments


def _iter_month_segments(
    start_dt: pd.Timestamp, stop_dt: pd.Timestamp
) -> list[tuple[str, pd.Timestamp, pd.Timestamp]]:
    """Yield (month, start, stop) segments clipped to month boundaries."""

    segments: list[tuple[str, pd.Timestamp, pd.Timestamp]] = []
    current_start = start_dt

    while current_start < stop_dt:
        month_label = current_start.strftime("%Y-%m")
        next_month_start = _start_of_next_month(current_start)
        month_end = min(stop_dt, next_month_start - pd.Timedelta(seconds=1))

        segments.append((month_label, current_start, month_end))

        if month_end >= stop_dt:
            break

        current_start = next_month_start

    return segments


def _start_of_next_month(dt: pd.Timestamp) -> pd.Timestamp:
    """Return the first instant of the month following ``dt`` preserving timezone."""

    tz = dt.tzinfo or "UTC"
    year = dt.year
    month = dt.month

    if month == 12:
        return pd.Timestamp(year=year + 1, month=1, day=1, tz=tz)

    return pd.Timestamp(year=year, month=month + 1, day=1, tz=tz)


def _datetime_to_day_fraction(dt: pd.Timestamp) -> float:
    """Convert a timestamp into day-of-month with fractional component."""

    base = dt.day
    base += dt.hour / 24.0
    base += dt.minute / 1440.0
    base += dt.second / 86_400.0
    base += dt.microsecond / 86_400_000_000.0
    return base
