"""Compare schedules page table display components."""

from __future__ import annotations

from typing import Any

import pandas as pd
import streamlit as st

from tsi.theme import add_vertical_space


def render_comparison_tables(compare_data: Any) -> None:
    """
    Display compact comparison tables for metrics.

    Args:
        compare_data: CompareData object from Rust backend with pre-computed statistics
    """
    st.subheader("📊 Summary Tables")

    # Extract data from CompareData
    current_scheduled = [
        b
        for b in compare_data.current_blocks
        if b.scheduled and b.scheduling_block_id in compare_data.common_ids
    ]
    comparison_scheduled = [
        b
        for b in compare_data.comparison_blocks
        if b.scheduled and b.scheduling_block_id in compare_data.common_ids
    ]

    newly_scheduled = [
        c for c in compare_data.scheduling_changes if c.change_type == "newly_scheduled"
    ]
    newly_unscheduled = [
        c for c in compare_data.scheduling_changes if c.change_type == "newly_unscheduled"
    ]

    newly_scheduled = [
        c for c in compare_data.scheduling_changes if c.change_type == "newly_scheduled"
    ]
    newly_unscheduled = [
        c for c in compare_data.scheduling_changes if c.change_type == "newly_unscheduled"
    ]

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
        compare_data.current_name,
        compare_data.comparison_name,
    )

    # Check if we have time data
    has_time_data = any(b.requested_hours > 0 for b in current_scheduled + comparison_scheduled)

    if has_time_data:
        time_df = _build_time_metrics_table(
            current_scheduled,
            comparison_scheduled,
            compare_data.current_name,
            compare_data.comparison_name,
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
        compare_data.current_blocks,
        compare_data.comparison_blocks,
        compare_data.common_ids,
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
    current_scheduled: list,
    comparison_scheduled: list,
    newly_scheduled: list,
    newly_unscheduled: list,
    current_name: str,
    comparison_name: str,
) -> pd.DataFrame:
    """Build priority and scheduling metrics table from CompareBlock lists."""
    current_count = len(current_scheduled)
    comparison_count = len(comparison_scheduled)

    current_total_priority = sum(b.priority for b in current_scheduled) if current_count > 0 else 0
    current_mean_priority = current_total_priority / current_count if current_count > 0 else 0
    current_priorities = (
        sorted([b.priority for b in current_scheduled]) if current_count > 0 else [0]
    )
    current_median_priority = (
        current_priorities[len(current_priorities) // 2] if current_count > 0 else 0
    )

    comp_total_priority = (
        sum(b.priority for b in comparison_scheduled) if comparison_count > 0 else 0
    )
    comp_mean_priority = comp_total_priority / comparison_count if comparison_count > 0 else 0
    comp_priorities = (
        sorted([b.priority for b in comparison_scheduled]) if comparison_count > 0 else [0]
    )
    comp_median_priority = comp_priorities[len(comp_priorities) // 2] if comparison_count > 0 else 0

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
    current_scheduled: list,
    comparison_scheduled: list,
    current_name: str,
    comparison_name: str,
) -> pd.DataFrame:
    """Build time metrics table from CompareBlock lists."""
    current_count = len(current_scheduled)
    comparison_count = len(comparison_scheduled)

    current_total_time = (
        sum(b.requested_hours for b in current_scheduled) if current_count > 0 else 0
    )
    current_mean_time = current_total_time / current_count if current_count > 0 else 0
    current_times = (
        sorted([b.requested_hours for b in current_scheduled]) if current_count > 0 else [0]
    )
    current_median_time = current_times[len(current_times) // 2] if current_count > 0 else 0

    comp_total_time = (
        sum(b.requested_hours for b in comparison_scheduled) if comparison_count > 0 else 0
    )
    comp_mean_time = comp_total_time / comparison_count if comparison_count > 0 else 0
    comp_times = (
        sorted([b.requested_hours for b in comparison_scheduled]) if comparison_count > 0 else [0]
    )
    comp_median_time = comp_times[len(comp_times) // 2] if comparison_count > 0 else 0

    # For gaps, we need scheduled blocks - skip for now as it requires additional data
    # These would need scheduled time periods which aren't in CompareBlock
    current_gaps_count = 0
    current_mean_gap = 0.0
    current_median_gap = 0.0
    comp_gaps_count = 0
    comp_mean_gap = 0.0
    comp_median_gap = 0.0

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
            _format_with_delta(
                f"{comp_gaps_count:,}", delta_gaps, is_count=True, inverse_colors=True
            ),
            _format_with_delta(f"{comp_mean_gap:.2f}", delta_mean_gap),
            _format_with_delta(f"{comp_median_gap:.2f}", delta_median_gap),
        ],
    }

    return pd.DataFrame(time_data)


def _render_change_details(
    newly_scheduled: list,
    newly_unscheduled: list,
    current_blocks: list,
    comparison_blocks: list,
    common_ids: list,
) -> None:
    """Render expandable details for scheduling changes."""
    col1, col2 = st.columns(2)

    with col1:
        if len(newly_scheduled) > 0:
            with st.expander(f"📋 View {len(newly_scheduled)} newly scheduled blocks"):
                # Convert to DataFrame for display
                display_data = []
                for change in newly_scheduled:
                    display_data.append(
                        {
                            "Block ID": change.scheduling_block_id,
                            "Priority": f"{change.priority:.2f}",
                        }
                    )
                display_df = pd.DataFrame(display_data)
                st.dataframe(display_df, hide_index=True, height=200, use_container_width=True)
        else:
            with st.expander("📋 View 0 newly scheduled blocks", expanded=False):
                st.info("No blocks were newly scheduled in the comparison schedule.")

    with col2:
        if len(newly_unscheduled) > 0:
            with st.expander(f"📋 View {len(newly_unscheduled)} newly unscheduled blocks"):
                # Convert to DataFrame for display
                display_data = []
                for change in newly_unscheduled:
                    display_data.append(
                        {
                            "Block ID": change.scheduling_block_id,
                            "Priority": f"{change.priority:.2f}",
                        }
                    )
                display_df = pd.DataFrame(display_data)
                st.dataframe(display_df, hide_index=True, height=200, use_container_width=True)
        else:
            with st.expander("📋 View 0 newly unscheduled blocks", expanded=False):
                st.info("No blocks were removed in the comparison schedule.")
