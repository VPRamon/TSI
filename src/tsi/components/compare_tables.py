"""Compare schedules page table display components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.services.compare_processing import calculate_observation_gaps
from tsi.theme import add_vertical_space


def render_comparison_tables(
    current_scheduled: pd.DataFrame,
    comparison_scheduled: pd.DataFrame,
    current_common: pd.DataFrame,
    comparison_common: pd.DataFrame,
    newly_scheduled: pd.DataFrame,
    newly_unscheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> None:
    """
    Display compact comparison tables for metrics.
    
    Args:
        current_scheduled: Current schedule's scheduled observations
        comparison_scheduled: Comparison schedule's scheduled observations
        current_common: Current schedule with common blocks
        comparison_common: Comparison schedule with common blocks
        newly_scheduled: Newly scheduled blocks
        newly_unscheduled: Newly unscheduled blocks
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule
    """
    st.subheader("📊 Summary Tables")
    
    # Add custom styling for the tables
    table_style = """
    <style>
        .comparison-table {
            width: 100%;
            border-collapse: collapse;
            margin: 1rem 0;
        }
        .comparison-table th {
            background-color: #1e1e1e;
            color: #e0e0e0;
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid #404040;
            font-weight: 600;
        }
        .comparison-table td {
            padding: 0.75rem;
            border-bottom: 1px solid #2a2a2a;
            color: #fafafa;
        }
        .comparison-table tr:hover {
            background-color: #252525;
        }
    </style>
    """
    st.markdown(table_style, unsafe_allow_html=True)
    
    # Build metrics table
    metrics_df = _build_metrics_table(
        current_scheduled,
        comparison_scheduled,
        newly_scheduled,
        newly_unscheduled,
        current_name,
        comparison_name,
    )
    
    # Build time metrics table if available
    has_time_data = (
        "requested_hours" in current_scheduled.columns
        and "requested_hours" in comparison_scheduled.columns
    )
    
    if has_time_data:
        time_df = _build_time_metrics_table(
            current_scheduled,
            comparison_scheduled,
            current_name,
            comparison_name,
        )
        
        # Display both tables side by side
        col1, col2 = st.columns(2)
        
        with col1:
            st.markdown("**Priority & Scheduling Metrics**")
            st.markdown(
                metrics_df.to_html(escape=False, index=False, classes="comparison-table"),
                unsafe_allow_html=True,
            )
        
        with col2:
            st.markdown("**Time Metrics**")
            st.markdown(
                time_df.to_html(escape=False, index=False, classes="comparison-table"),
                unsafe_allow_html=True,
            )
    else:
        # If no time data, just show the metrics table alone
        st.markdown(
            metrics_df.to_html(escape=False, index=False, classes="comparison-table"),
            unsafe_allow_html=True,
        )
    
    # Show expandable details for changes
    add_vertical_space(1)
    _render_change_details(
        newly_scheduled,
        newly_unscheduled,
        current_common,
        comparison_common,
    )


def _format_with_delta(
    value: str,
    delta: float,
    is_count: bool = False,
    inverse_colors: bool = False,
) -> str:
    """
    Format a value with a colored delta percentage label.
    
    Args:
        value: The value to display
        delta: The change from the previous value
        is_count: Whether this is a count metric
        inverse_colors: If True, positive changes are red (bad)
        
    Returns:
        HTML formatted string with delta badge
    """
    if delta == 0:
        return value
    
    # Calculate percentage change
    base_value = float(value.replace(",", ""))
    if base_value == 0:
        pct_change = 0.0
    else:
        pct_change = (delta / base_value) * 100
    
    # Choose color based on sign
    if inverse_colors:
        color = "#d62728" if delta > 0 else "#2ca02c"
    else:
        color = "#2ca02c" if delta > 0 else "#d62728"
    sign = "+" if delta > 0 else ""
    
    delta_label = f'<span style="background-color: {color}; color: white; padding: 2px 6px; border-radius: 3px; margin-left: 8px; font-size: 0.85em; font-weight: bold;">{sign}{pct_change:.1f}%</span>'
    
    return f"{value} {delta_label}"


def _build_metrics_table(
    current_scheduled: pd.DataFrame,
    comparison_scheduled: pd.DataFrame,
    newly_scheduled: pd.DataFrame,
    newly_unscheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> pd.DataFrame:
    """Build priority and scheduling metrics table."""
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
            _format_with_delta(f"{comparison_count:,}", delta_count, is_count=True),
            _format_with_delta(f"{comp_total_priority:.2f}", delta_total_priority),
            _format_with_delta(f"{comp_mean_priority:.2f}", delta_mean_priority),
            _format_with_delta(f"{comp_median_priority:.2f}", delta_median_priority),
            f"{len(newly_scheduled):,}",
            f"{len(newly_unscheduled):,}",
        ],
    }
    
    return pd.DataFrame(metrics_data)


def _build_time_metrics_table(
    current_scheduled: pd.DataFrame,
    comparison_scheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> pd.DataFrame:
    """Build time metrics table with gap statistics."""
    current_count = len(current_scheduled)
    comparison_count = len(comparison_scheduled)
    
    current_total_time = current_scheduled["requested_hours"].sum() if current_count > 0 else 0
    current_mean_time = current_scheduled["requested_hours"].mean() if current_count > 0 else 0
    current_median_time = current_scheduled["requested_hours"].median() if current_count > 0 else 0
    
    comp_total_time = comparison_scheduled["requested_hours"].sum() if comparison_count > 0 else 0
    comp_mean_time = comparison_scheduled["requested_hours"].mean() if comparison_count > 0 else 0
    comp_median_time = comparison_scheduled["requested_hours"].median() if comparison_count > 0 else 0
    
    # Calculate gaps
    current_gaps_count, current_mean_gap, current_median_gap = calculate_observation_gaps(
        current_scheduled
    )
    comp_gaps_count, comp_mean_gap, comp_median_gap = calculate_observation_gaps(
        comparison_scheduled
    )
    
    # Calculate deltas
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
            _format_with_delta(f"{comp_total_time:.2f}", delta_total_time),
            _format_with_delta(f"{comp_mean_time:.2f}", delta_mean_time),
            _format_with_delta(f"{comp_median_time:.2f}", delta_median_time),
            _format_with_delta(f"{comp_gaps_count:,}", delta_gaps, is_count=True, inverse_colors=True),
            _format_with_delta(f"{comp_mean_gap:.2f}", delta_mean_gap),
            _format_with_delta(f"{comp_median_gap:.2f}", delta_median_gap),
        ],
    }
    
    return pd.DataFrame(time_data)


def _render_change_details(
    newly_scheduled: pd.DataFrame,
    newly_unscheduled: pd.DataFrame,
    current_common: pd.DataFrame,
    comparison_common: pd.DataFrame,
) -> None:
    """Render expandable details for scheduling changes."""
    col1, col2 = st.columns(2)
    
    with col1:
        if len(newly_scheduled) > 0:
            with st.expander(f"📋 View {len(newly_scheduled)} newly scheduled blocks"):
                display_df = newly_scheduled[["schedulingBlockId", "priority_current"]].copy()
                
                comparison_info = comparison_common[
                    ["schedulingBlockId", "targetName", "scheduled_period.start", "scheduled_period.stop"]
                ].copy()
                
                display_df = display_df.merge(comparison_info, on="schedulingBlockId", how="left")
                display_df = display_df.rename(columns={"priority_current": "priority"})
                display_df = display_df[[
                    "schedulingBlockId", "targetName", "priority",
                    "scheduled_period.start", "scheduled_period.stop"
                ]]
                
                display_df = display_df.rename(columns={
                    "schedulingBlockId": "Block ID",
                    "targetName": "Target Name",
                    "priority": "Priority",
                    "scheduled_period.start": "Start (MJD)",
                    "scheduled_period.stop": "Stop (MJD)",
                })
                
                st.dataframe(display_df, hide_index=True, height=200, use_container_width=True)
        else:
            with st.expander("📋 View 0 newly scheduled blocks", expanded=False):
                st.info("No blocks were newly scheduled in the comparison schedule.")
    
    with col2:
        if len(newly_unscheduled) > 0:
            with st.expander(f"📋 View {len(newly_unscheduled)} newly unscheduled blocks"):
                display_df = newly_unscheduled[["schedulingBlockId", "priority_current"]].copy()
                
                current_info = current_common[["schedulingBlockId", "targetName"]].copy()
                
                display_df = display_df.merge(current_info, on="schedulingBlockId", how="left")
                display_df = display_df.rename(columns={"priority_current": "priority"})
                display_df = display_df[["schedulingBlockId", "targetName", "priority"]]
                
                display_df = display_df.rename(columns={
                    "schedulingBlockId": "Block ID",
                    "targetName": "Target Name",
                    "priority": "Priority",
                })
                
                st.dataframe(display_df, hide_index=True, height=200, use_container_width=True)
        else:
            with st.expander("📋 View 0 newly unscheduled blocks", expanded=False):
                st.info("No blocks were removed in the comparison schedule.")
