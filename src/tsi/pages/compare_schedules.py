"""Schedule Comparison page for comparing two schedules."""

from __future__ import annotations

import streamlit as st
import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots

from tsi import state
from core.loaders import load_schedule_from_json
from tsi.services import prepare_dataframe
from tsi.theme import add_vertical_space
from tsi.config import PLOT_HEIGHT


def render() -> None:
    """Render the Schedule Comparison page."""
    st.title("⚖️ Compare Schedules")

    st.markdown(
        """
        Compare the current schedule with a newly uploaded one. 
        View differences in scheduled blocks, priority distributions, and planned time.
        """
    )

    # Get current schedule
    current_df = state.get_prepared_data()

    if current_df is None:
        st.warning("⚠️ No base schedule loaded. Please load a schedule from the landing page first.")
        return

    # Display current schedule info
    st.subheader("📊 Current Schedule")
    
    current_filename = state.get_data_filename()
    if current_filename:
        st.info(f"**Loaded:** {current_filename}")
    
    col1, col2, col3 = st.columns(3)
    with col1:
        st.metric("Total Blocks", f"{len(current_df):,}")
    with col2:
        scheduled_count = current_df["scheduled_flag"].sum()
        st.metric("Scheduled", f"{int(scheduled_count):,}")
    with col3:
        st.metric("Mean Priority", f"{current_df['priority'].mean():.2f}")

    add_vertical_space(2)
    st.divider()

    # Upload comparison schedule
    st.subheader("📤 Upload Comparison Schedule")
    
    uploaded_json = st.file_uploader(
        "Choose a schedule.json file to compare",
        type=["json"],
        help="Upload a schedule.json file to compare with the current schedule",
        key="comparison_json_uploader",
    )

    # Optional visibility file for comparison schedule
    with st.expander("🔍 Add visibility data for comparison schedule (optional)", expanded=False):
        uploaded_visibility = st.file_uploader(
            "Choose possible_periods.json (optional)",
            type=["json"],
            help="Optional: upload visibility/possible periods data for the comparison schedule",
            key="comparison_visibility_uploader",
        )

    if uploaded_json is not None:
        # Get comparison schedule from session state if already processed
        comparison_df = state.get_comparison_schedule()
        
        # Track if we need to reprocess
        file_token = f"{uploaded_json.name}:{uploaded_json.size}"
        last_token = st.session_state.get("comparison_file_token")
        
        if comparison_df is None or last_token != file_token:
            # Load and process the comparison schedule
            try:
                with st.spinner("Loading and processing comparison schedule..."):
                    result = load_schedule_from_json(
                        uploaded_json, 
                        uploaded_visibility if uploaded_visibility else None,
                        validate=True
                    )
                    
                    # Get the raw dataframe
                    comparison_df = result.dataframe
                    
                    # Apply the same preparation transformations as the main schedule
                    # This adds scheduled_start_dt, scheduled_stop_dt, and other derived columns
                    comparison_df = prepare_dataframe(comparison_df)
                    
                    # Convert visibility lists to strings for compatibility
                    if "visibility" in comparison_df.columns:
                        comparison_df["visibility"] = comparison_df["visibility"].apply(str)
                    
                    # Show processing stats only if there are warnings
                    if result.validation.warnings:
                        with st.expander("⚠️ Processing warnings", expanded=False):
                            for warning in result.validation.warnings:
                                st.warning(f"  - {warning}")
                    
                    # Store in session state
                    state.set_comparison_schedule(comparison_df)
                    st.session_state["comparison_file_token"] = file_token
                    st.session_state["comparison_filename"] = uploaded_json.name.replace(".json", "")
                    
                    # Show success message only during initial load
                    st.success(f"✅ Processed {len(comparison_df)} scheduling blocks")
                    
            except Exception as e:
                st.error(f"❌ Error loading comparison schedule: {str(e)}")
                st.exception(e)
                return

        # Display comparison if we have both schedules
        if comparison_df is not None:
            add_vertical_space(2)
            st.divider()
            
            # Show comparison schedule info
            st.subheader("📊 Comparison Schedule")
            
            comparison_filename = st.session_state.get("comparison_filename", "Comparison Schedule")
            st.info(f"**Loaded:** {comparison_filename}")
            
            col1, col2, col3 = st.columns(3)
            with col1:
                st.metric("Total Blocks", f"{len(comparison_df):,}")
            with col2:
                comp_scheduled_count = comparison_df["scheduled_flag"].sum()
                st.metric("Scheduled", f"{int(comp_scheduled_count):,}")
            with col3:
                st.metric("Mean Priority", f"{comparison_df['priority'].mean():.2f}")

            add_vertical_space(2)
            st.divider()

            # Validate and compare
            _validate_and_compare(current_df, comparison_df, current_filename or "Current", comparison_filename)


def _validate_and_compare(
    current_df: pd.DataFrame,
    comparison_df: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> None:
    """
    Validate that both schedules have the same blocks and display comparison metrics.
    
    Args:
        current_df: Current schedule DataFrame
        comparison_df: Comparison schedule DataFrame
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule
    """
    st.header("🔍 Schedule Comparison")
    
    # Get block IDs from both schedules
    current_ids = set(current_df["schedulingBlockId"].unique())
    comparison_ids = set(comparison_df["schedulingBlockId"].unique())
    
    # Find differences
    only_in_current = current_ids - comparison_ids
    only_in_comparison = comparison_ids - current_ids
    common_ids = current_ids & comparison_ids
    
    # Only display validation section if there are discrepancies
    if only_in_current or only_in_comparison:
        st.error("⚠️ **Discrepancy Warning!** The schedules contain different sets of blocks.")
        
        col1, col2 = st.columns(2)
        
        with col1:
            if only_in_current:
                st.warning(f"**Blocks only in {current_name}:** {len(only_in_current)}")
                with st.expander(f"View {len(only_in_current)} blocks", expanded=False):
                    st.dataframe(
                        pd.DataFrame({"schedulingBlockId": sorted(only_in_current)}),
                        hide_index=True,
                        height=200,
                    )
        
        with col2:
            if only_in_comparison:
                st.warning(f"**Blocks only in {comparison_name}:** {len(only_in_comparison)}")
                with st.expander(f"View {len(only_in_comparison)} blocks", expanded=False):
                    st.dataframe(
                        pd.DataFrame({"schedulingBlockId": sorted(only_in_comparison)}),
                        hide_index=True,
                        height=200,
                    )
        
        st.info(f"**Common blocks:** {len(common_ids)} blocks will be used for comparison")
        
        add_vertical_space(1)
        st.divider()
    
    # Filter to common blocks for fair comparison
    current_common = current_df[current_df["schedulingBlockId"].isin(common_ids)]
    comparison_common = comparison_df[comparison_df["schedulingBlockId"].isin(common_ids)]
    
    if len(common_ids) == 0:
        st.error("❌ No common blocks found. Cannot perform comparison.")
        return
    
    # Filter to scheduled blocks for both schedules
    current_scheduled = current_common[current_common["scheduled_flag"] == 1]
    comparison_scheduled = comparison_common[comparison_common["scheduled_flag"] == 1]
    
    # Merge on block ID to compare scheduling status
    merged = pd.merge(
        current_common[["schedulingBlockId", "scheduled_flag", "priority"]],
        comparison_common[["schedulingBlockId", "scheduled_flag", "priority"]],
        on="schedulingBlockId",
        suffixes=("_current", "_comparison")
    )
    
    # Find blocks with changed scheduling status
    newly_scheduled = merged[
        (merged["scheduled_flag_current"] == 0) & (merged["scheduled_flag_comparison"] == 1)
    ]
    newly_unscheduled = merged[
        (merged["scheduled_flag_current"] == 1) & (merged["scheduled_flag_comparison"] == 0)
    ]
    
    # Build comparison tables
    _display_comparison_tables(
        current_scheduled, 
        comparison_scheduled, 
        current_common,
        comparison_common,
        newly_scheduled,
        newly_unscheduled,
        current_name, 
        comparison_name
    )
    
    add_vertical_space(2)
    st.divider()
    
    # Visualization section
    _display_comparison_plots(
        current_scheduled,
        comparison_scheduled,
        current_common,
        comparison_common,
        newly_scheduled,
        newly_unscheduled,
        current_name,
        comparison_name
    )


def _calculate_observation_gaps(df: pd.DataFrame, schedule_name: str = "") -> tuple[int, float, float]:
    """
    Calculate gaps statistics between scheduled observations.
    
    A gap is defined as any time period between consecutive scheduled observations.
    
    Args:
        df: DataFrame with scheduled observations containing scheduled_start_dt and scheduled_stop_dt
        schedule_name: Name of the schedule for debugging
    
    Returns:
        Tuple of (num_gaps, mean_gap_hours, median_gap_hours)
    """
    if len(df) <= 1:  # Need at least 2 observations to have a gap
        return 0, 0.0, 0.0
    
    # Check if we have the necessary datetime columns
    if "scheduled_start_dt" not in df.columns or "scheduled_stop_dt" not in df.columns:
        return 0, 0.0, 0.0
    
    # Filter out rows with null datetime values
    valid_df = df.dropna(subset=["scheduled_start_dt", "scheduled_stop_dt"]).copy()
    
    if len(valid_df) <= 1:
        return 0, 0.0, 0.0
    
    # Sort by start time
    sorted_df = valid_df.sort_values("scheduled_start_dt").reset_index(drop=True)
    
    # Calculate gaps and their durations
    gaps = 0
    gap_durations = []  # in hours
    
    for i in range(len(sorted_df) - 1):
        current_end = sorted_df.iloc[i]["scheduled_stop_dt"]
        next_start = sorted_df.iloc[i + 1]["scheduled_start_dt"]
        
        # If there's a gap between observations (even 1 second counts)
        if next_start > current_end:
            gaps += 1
            gap_duration_hours = (next_start - current_end).total_seconds() / 3600
            gap_durations.append(gap_duration_hours)
    
    # Calculate mean and median
    mean_gap = sum(gap_durations) / len(gap_durations) if gap_durations else 0.0
    median_gap = sorted(gap_durations)[len(gap_durations) // 2] if gap_durations else 0.0
    
    return gaps, mean_gap, median_gap


def _display_comparison_tables(
    current_scheduled: pd.DataFrame,
    comparison_scheduled: pd.DataFrame,
    current_common: pd.DataFrame,
    comparison_common: pd.DataFrame,
    newly_scheduled: pd.DataFrame,
    newly_unscheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> None:
    """Display compact comparison tables for metrics."""
    st.subheader("📊 Summary Tables")
    
    # Add custom styling for the tables (once, at the beginning)
    table_style = """
    <style>
        .comparison-table {
            width: 100%;
            border-collapse: collapse;
            margin: 1rem 0;
        }
        .comparison-table th {
            background-color: #262730;
            color: #fafafa;
            padding: 0.75rem;
            text-align: left;
            border-bottom: 2px solid #4da6ff;
            font-weight: 600;
        }
        .comparison-table td {
            padding: 0.75rem;
            border-bottom: 1px solid #3d3d4d;
            color: #fafafa;
        }
        .comparison-table tr:hover {
            background-color: #1a1d24;
        }
    </style>
    """
    st.markdown(table_style, unsafe_allow_html=True)
    
    # Table 1: Priority and Scheduling Metrics
    current_count = len(current_scheduled)
    comparison_count = len(comparison_scheduled)
    
    current_total_priority = current_scheduled["priority"].sum() if current_count > 0 else 0
    current_mean_priority = current_scheduled["priority"].mean() if current_count > 0 else 0
    current_median_priority = current_scheduled["priority"].median() if current_count > 0 else 0
    
    comp_total_priority = comparison_scheduled["priority"].sum() if comparison_count > 0 else 0
    comp_mean_priority = comparison_scheduled["priority"].mean() if comparison_count > 0 else 0
    comp_median_priority = comparison_scheduled["priority"].median() if comparison_count > 0 else 0
    
    # Calculate deltas
    delta_count = comparison_count - current_count
    delta_total_priority = comp_total_priority - current_total_priority
    delta_mean_priority = comp_mean_priority - current_mean_priority
    delta_median_priority = comp_median_priority - current_median_priority
    
    # Helper function to format value with delta label
    def format_with_delta(value: str, delta: float, is_count: bool = False, inverse_colors: bool = False) -> str:
        """Format a value with a colored delta percentage label.
        
        Args:
            value: The value to display
            delta: The change from the previous value
            is_count: Whether this is a count metric
            inverse_colors: If True, positive changes are red (bad) and negative are green (good)
        """
        if delta == 0:
            return value
        
        # Calculate percentage change (avoiding division by zero)
        base_value = float(value.replace(",", ""))
        if base_value == 0:
            pct_change = 0
        else:
            pct_change = (delta / base_value) * 100
        
        # Choose color based on sign (inverse logic for metrics where increase is bad)
        if inverse_colors:
            color = "#d62728" if delta > 0 else "#2ca02c"  # red for positive, green for negative
        else:
            color = "#2ca02c" if delta > 0 else "#d62728"  # green for positive, red for negative
        sign = "+" if delta > 0 else ""
        
        # Format the delta label
        delta_label = f'<span style="background-color: {color}; color: white; padding: 2px 6px; border-radius: 3px; margin-left: 8px; font-size: 0.85em; font-weight: bold;">{sign}{pct_change:.1f}%</span>'
        
        return f"{value} {delta_label}"
    
    metrics_data = {
        "Metric": [
            "Scheduled Blocks",
            "Total Priority Sum",
            "Mean Priority",
            "Median Priority",
            "Newly Scheduled",
            "Newly Unscheduled",
        ],
        current_name: [
            f"{current_count:,}",
            f"{current_total_priority:.2f}",
            f"{current_mean_priority:.2f}",
            f"{current_median_priority:.2f}",
            "-",
            "-",
        ],
        comparison_name: [
            format_with_delta(f"{comparison_count:,}", delta_count, is_count=True),
            format_with_delta(f"{comp_total_priority:.2f}", delta_total_priority),
            format_with_delta(f"{comp_mean_priority:.2f}", delta_mean_priority),
            format_with_delta(f"{comp_median_priority:.2f}", delta_median_priority),
            f"{len(newly_scheduled):,}",
            f"{len(newly_unscheduled):,}",
        ],
    }
    
    metrics_df = pd.DataFrame(metrics_data)
    
    # Table 2: Time Metrics (if available)
    has_time_data = "requested_hours" in current_scheduled.columns and "requested_hours" in comparison_scheduled.columns
    
    if has_time_data:
        current_total_time = current_scheduled["requested_hours"].sum() if current_count > 0 else 0
        current_mean_time = current_scheduled["requested_hours"].mean() if current_count > 0 else 0
        current_median_time = current_scheduled["requested_hours"].median() if current_count > 0 else 0
        
        comp_total_time = comparison_scheduled["requested_hours"].sum() if comparison_count > 0 else 0
        comp_mean_time = comparison_scheduled["requested_hours"].mean() if comparison_count > 0 else 0
        comp_median_time = comparison_scheduled["requested_hours"].median() if comparison_count > 0 else 0
        
        # Calculate gaps between observations
        current_gaps_count, current_mean_gap, current_median_gap = _calculate_observation_gaps(current_scheduled, current_name)
        comp_gaps_count, comp_mean_gap, comp_median_gap = _calculate_observation_gaps(comparison_scheduled, comparison_name)
        
        # Calculate time deltas
        delta_total_time = comp_total_time - current_total_time
        delta_mean_time = comp_mean_time - current_mean_time
        delta_median_time = comp_median_time - current_median_time
        delta_gaps = comp_gaps_count - current_gaps_count
        delta_mean_gap = comp_mean_gap - current_mean_gap
        delta_median_gap = comp_median_gap - current_median_gap
        
        time_data = {
            "Metric": [
                "Total Planned Time (hrs)",
                "Mean Planned Time (hrs)",
                "Median Planned Time (hrs)",
                "Gaps Between Observations",
                "Mean Gap Duration (hrs)",
                "Median Gap Duration (hrs)",
            ],
            current_name: [
                f"{current_total_time:.2f}",
                f"{current_mean_time:.2f}",
                f"{current_median_time:.2f}",
                f"{current_gaps_count:,}",
                f"{current_mean_gap:.2f}",
                f"{current_median_gap:.2f}",
            ],
            comparison_name: [
                format_with_delta(f"{comp_total_time:.2f}", delta_total_time),
                format_with_delta(f"{comp_mean_time:.2f}", delta_mean_time),
                format_with_delta(f"{comp_median_time:.2f}", delta_median_time),
                format_with_delta(f"{comp_gaps_count:,}", delta_gaps, is_count=True, inverse_colors=True),
                format_with_delta(f"{comp_mean_gap:.2f}", delta_mean_gap),
                format_with_delta(f"{comp_median_gap:.2f}", delta_median_gap),
            ],
        }
        
        time_df = pd.DataFrame(time_data)
        
        # Display both tables side by side
        col1, col2 = st.columns(2)
        
        with col1:
            st.markdown("**Priority & Scheduling Metrics**")
            st.markdown(metrics_df.to_html(escape=False, index=False, classes="comparison-table"), unsafe_allow_html=True)
        
        with col2:
            st.markdown("**Time Metrics**")
            st.markdown(time_df.to_html(escape=False, index=False, classes="comparison-table"), unsafe_allow_html=True)
    else:
        # If no time data, just show the metrics table alone
        st.markdown(metrics_df.to_html(escape=False, index=False, classes="comparison-table"), unsafe_allow_html=True)
    
    # Show expandable details for changes
    if len(newly_scheduled) > 0 or len(newly_unscheduled) > 0:
        add_vertical_space(1)
        
        col1, col2 = st.columns(2)
        
        with col1:
            if len(newly_scheduled) > 0:
                with st.expander(f"📋 View {len(newly_scheduled)} newly scheduled blocks"):
                    display_df = newly_scheduled[["schedulingBlockId", "priority_current"]].rename(
                        columns={"priority_current": "priority"}
                    )
                    st.dataframe(display_df, hide_index=True, height=200, width="stretch")
        
        with col2:
            if len(newly_unscheduled) > 0:
                with st.expander(f"📋 View {len(newly_unscheduled)} newly unscheduled blocks"):
                    display_df = newly_unscheduled[["schedulingBlockId", "priority_current"]].rename(
                        columns={"priority_current": "priority"}
                    )
                    st.dataframe(display_df, hide_index=True, height=200, width="stretch")


def _display_comparison_plots(
    current_scheduled: pd.DataFrame,
    comparison_scheduled: pd.DataFrame,
    current_common: pd.DataFrame,
    comparison_common: pd.DataFrame,
    newly_scheduled: pd.DataFrame,
    newly_unscheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> None:
    """Display comparison visualizations."""
    st.header("� Comparison Visualizations")
    
    # Plot 1: Priority Distribution Comparison
    st.subheader("Priority Distribution Comparison")
    fig_priority = _create_priority_distribution_plot(
        current_scheduled, comparison_scheduled, current_name, comparison_name
    )
    st.plotly_chart(fig_priority, width="stretch")
    
    add_vertical_space(1)
    
    # Plot 2: Scheduling Status Breakdown
    st.subheader("Scheduling Status Breakdown")
    fig_status = _create_scheduling_status_plot(
        current_common, comparison_common, current_name, comparison_name
    )
    st.plotly_chart(fig_status, width="stretch")
    
    add_vertical_space(1)
    
    # Plot 3: Changes Flow (Sankey-style or bar chart)
    if len(newly_scheduled) > 0 or len(newly_unscheduled) > 0:
        st.subheader("Scheduling Changes")
        fig_changes = _create_changes_plot(
            newly_scheduled, newly_unscheduled, current_name, comparison_name
        )
        st.plotly_chart(fig_changes, width="stretch")
    
    # Plot 4: Time comparison (if available)
    has_time_data = "requested_hours" in current_scheduled.columns and "requested_hours" in comparison_scheduled.columns
    if has_time_data:
        add_vertical_space(1)
        st.subheader("Planned Time Distribution")
        fig_time = _create_time_distribution_plot(
            current_scheduled, comparison_scheduled, current_name, comparison_name
        )
        st.plotly_chart(fig_time, width="stretch")


def _create_priority_distribution_plot(
    current_scheduled: pd.DataFrame,
    comparison_scheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """Create overlaid histogram of priority distributions."""
    fig = go.Figure()
    
    # Current schedule
    fig.add_trace(go.Histogram(
        x=current_scheduled["priority"],
        name=current_name,
        opacity=0.7,
        marker=dict(color="#1f77b4"),
        nbinsx=30,
    ))
    
    # Comparison schedule
    fig.add_trace(go.Histogram(
        x=comparison_scheduled["priority"],
        name=comparison_name,
        opacity=0.7,
        marker=dict(color="#ff7f0e"),
        nbinsx=30,
    ))
    
    fig.update_layout(
        barmode='overlay',
        xaxis_title="Priority",
        yaxis_title="Count",
        height=PLOT_HEIGHT - 100,
        legend=dict(orientation="h", yanchor="bottom", y=1.02, xanchor="right", x=1),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )
    
    return fig


def _create_scheduling_status_plot(
    current_common: pd.DataFrame,
    comparison_common: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """Create grouped bar chart of scheduling status."""
    current_scheduled = (current_common["scheduled_flag"] == 1).sum()
    current_unscheduled = (current_common["scheduled_flag"] == 0).sum()
    
    comp_scheduled = (comparison_common["scheduled_flag"] == 1).sum()
    comp_unscheduled = (comparison_common["scheduled_flag"] == 0).sum()
    
    fig = go.Figure()
    
    fig.add_trace(go.Bar(
        name=current_name,
        x=["Scheduled", "Unscheduled"],
        y=[current_scheduled, current_unscheduled],
        marker=dict(color="#1f77b4"),
        text=[f"{current_scheduled:,}", f"{current_unscheduled:,}"],
        textposition="auto",
    ))
    
    fig.add_trace(go.Bar(
        name=comparison_name,
        x=["Scheduled", "Unscheduled"],
        y=[comp_scheduled, comp_unscheduled],
        marker=dict(color="#ff7f0e"),
        text=[f"{comp_scheduled:,}", f"{comp_unscheduled:,}"],
        textposition="auto",
    ))
    
    fig.update_layout(
        barmode='group',
        yaxis_title="Number of Blocks",
        height=PLOT_HEIGHT - 150,
        legend=dict(orientation="h", yanchor="bottom", y=1.02, xanchor="right", x=1),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )
    
    return fig


def _create_changes_plot(
    newly_scheduled: pd.DataFrame,
    newly_unscheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """Create visualization of scheduling changes."""
    # Priority distribution of changed blocks
    fig = make_subplots(
        rows=1, cols=2,
        subplot_titles=("Newly Scheduled Blocks", "Newly Unscheduled Blocks"),
        specs=[[{"type": "histogram"}, {"type": "histogram"}]]
    )
    
    if len(newly_scheduled) > 0:
        fig.add_trace(
            go.Histogram(
                x=newly_scheduled["priority_current"],
                name="Newly Scheduled",
                marker=dict(color="#2ca02c"),
                nbinsx=20,
                showlegend=False,
            ),
            row=1, col=1
        )
    
    if len(newly_unscheduled) > 0:
        fig.add_trace(
            go.Histogram(
                x=newly_unscheduled["priority_current"],
                name="Newly Unscheduled",
                marker=dict(color="#d62728"),
                nbinsx=20,
                showlegend=False,
            ),
            row=1, col=2
        )
    
    fig.update_xaxes(title_text="Priority", row=1, col=1)
    fig.update_xaxes(title_text="Priority", row=1, col=2)
    fig.update_yaxes(title_text="Count", row=1, col=1)
    fig.update_yaxes(title_text="Count", row=1, col=2)
    
    fig.update_layout(
        height=PLOT_HEIGHT - 100,
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )
    
    return fig


def _create_time_distribution_plot(
    current_scheduled: pd.DataFrame,
    comparison_scheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """Create box plot comparison of requested time distributions."""
    fig = go.Figure()
    
    fig.add_trace(go.Box(
        y=current_scheduled["requested_hours"],
        name=current_name,
        marker=dict(color="#1f77b4"),
        boxmean='sd',
    ))
    
    fig.add_trace(go.Box(
        y=comparison_scheduled["requested_hours"],
        name=comparison_name,
        marker=dict(color="#ff7f0e"),
        boxmean='sd',
    ))
    
    fig.update_layout(
        yaxis_title="Requested Hours",
        height=PLOT_HEIGHT - 150,
        showlegend=True,
        legend=dict(orientation="h", yanchor="bottom", y=1.02, xanchor="right", x=1),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )
    
    return fig
