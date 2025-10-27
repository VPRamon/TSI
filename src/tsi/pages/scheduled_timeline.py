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

    # Sidebar filters
    with st.sidebar:
        st.header("Timeline Filters")

        # Priority range filter
        stored_range = state.get_priority_range()
        if (
            stored_range is None
            or stored_range[0] < priority_min
            or stored_range[1] > priority_max
            or stored_range == (0.0, 10.0)
        ):
            default_range = (priority_min, priority_max)
        else:
            default_range = stored_range

        priority_range = render_priority_filter(
            "timeline_priority_range",
            min_value=priority_min,
            max_value=priority_max,
            default=default_range,
        )
        state.set_priority_range(priority_range)

        # Month selector
        st.markdown("**Months to display**")
        selected_months = st.multiselect(
            "Select months",
            options=all_months,
            default=all_months,
            key="timeline_month_filter",
            help="Filter by specific months to focus on specific periods",
        )

        # Duration threshold filter
        st.markdown("**Duration filter**")
        show_short_blocks = st.checkbox(
            "Filter short blocks",
            value=False,
            key="timeline_filter_short",
            help="Enable to hide blocks with duration less than the threshold",
        )

        duration_threshold = 0.0
        if show_short_blocks:
            duration_threshold = render_number_input(
                "Minimum duration (hours)",
                min_value=0,
                max_value=100,
                default=1,
                key="timeline_duration_threshold",
            )

        st.divider()

        # Instructions
        st.info(
            """
            **💡 How to read the timeline:**
            - **Y axis (vertical)** = Months of the year
            - **X axis (horizontal)** = Days of the month (1-31)
            - Each row shows only the observations for that month
            - Horizontal blocks = scheduled observations
            - Color = priority (Viridis scale)
            - You can zoom/pan to explore details
            - Day range is limited to 1-31 (invalid days won't be shown)
            """
        )

        st.divider()

        if render_reset_filters_button():
            state.reset_filters()
            st.rerun()

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
    fig = build_monthly_timeline(filtered_df, priority_range)

    # Display the chart with configuration to limit pan range
    # Disable pan on X axis to prevent moving beyond day boundaries
    # Users can still zoom in/out
    config = {
        "displayModeBar": True,
        "displaylogo": False,
        "modeBarButtonsToRemove": ["lasso2d", "select2d"],
        "scrollZoom": True,  # Enable zoom with scroll wheel
    }
    st.plotly_chart(fig, use_container_width=True, config=config)

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


def build_monthly_timeline(df: pd.DataFrame, priority_range: tuple[float, float]) -> go.Figure:
    """
    Build a Plotly figure showing scheduled observations grouped by month.

    Each month is represented as a horizontal row, with observations displayed
    as bars within that month's time range (days 1-31). This prevents overflow
    by constraining each row to its month's temporal boundaries.

    Args:
        df: DataFrame with scheduled observations and monthly grouping columns
        priority_range: (min, max) priority values for color normalization

    Returns:
        Plotly Figure object
    """
    # Get sorted months for Y-axis ordering
    ordered_months = sorted(df["scheduled_month_label"].unique())
    num_months = len(ordered_months)

    # Create figure
    fig = go.Figure()

    # Normalize priority for color mapping
    priority_min, priority_max = priority_range
    if priority_max <= priority_min:
        priority_max = priority_min + 1.0

    # Create a mapping from month_label to numeric position
    month_to_position = {month: idx for idx, month in enumerate(ordered_months)}

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
