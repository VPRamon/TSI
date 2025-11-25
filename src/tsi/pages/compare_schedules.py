"""Schedule Comparison page for comparing two schedules."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.components.compare_upload import render_file_upload
from tsi.components.compare_validation import (
    validate_and_display_discrepancies,
    compute_scheduling_changes,
)
from tsi.components.compare_tables import render_comparison_tables
from tsi.plots.compare_plots import (
    create_changes_plot,
    create_priority_distribution_plot,
    create_scheduling_status_plot,
    create_time_distribution_plot,
)
from tsi.theme import add_vertical_space


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

    # Handle file upload and loading
    comparison_df = render_file_upload()

    if comparison_df is not None:
        # Get schedule names
        current_filename = state.get_data_filename()
        comparison_filename = st.session_state.get("comparison_filename", "Comparison Schedule")

        # Validate and compare
        _display_comparison(
            current_df,
            comparison_df,
            current_filename or "Current",
            comparison_filename,
        )


def _display_comparison(
    current_df,
    comparison_df,
    current_name: str,
    comparison_name: str,
) -> None:
    """
    Validate and display comparison between two schedules.

    Args:
        current_df: Current schedule DataFrame
        comparison_df: Comparison schedule DataFrame
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule
    """
    st.header("🔍 Schedule Comparison")

    # Validate and display discrepancies
    only_in_current, only_in_comparison, common_ids = validate_and_display_discrepancies(
        current_df, comparison_df, current_name, comparison_name
    )

    if len(common_ids) == 0:
        st.error("❌ No common blocks found. Cannot perform comparison.")
        return

    # Filter to common blocks
    current_common = current_df[current_df["schedulingBlockId"].isin(common_ids)]
    comparison_common = comparison_df[comparison_df["schedulingBlockId"].isin(common_ids)]

    # Filter to scheduled blocks
    current_scheduled = current_common[current_common["scheduled_flag"] == 1]
    comparison_scheduled = comparison_common[comparison_common["scheduled_flag"] == 1]

    # Find scheduling changes
    newly_scheduled, newly_unscheduled = compute_scheduling_changes(current_common, comparison_common)

    # Display comparison tables
    render_comparison_tables(
        current_scheduled=current_scheduled,
        comparison_scheduled=comparison_scheduled,
        current_common=current_common,
        comparison_common=comparison_common,
        newly_scheduled=newly_scheduled,
        newly_unscheduled=newly_unscheduled,
        current_name=current_name,
        comparison_name=comparison_name,
    )

    add_vertical_space(2)
    st.divider()

    # Display visualizations
    _display_comparison_plots(
        current_scheduled=current_scheduled,
        comparison_scheduled=comparison_scheduled,
        current_common=current_common,
        comparison_common=comparison_common,
        newly_scheduled=newly_scheduled,
        newly_unscheduled=newly_unscheduled,
        current_name=current_name,
        comparison_name=comparison_name,
    )


def _display_comparison_plots(
    current_scheduled,
    comparison_scheduled,
    current_common,
    comparison_common,
    newly_scheduled,
    newly_unscheduled,
    current_name: str,
    comparison_name: str,
) -> None:
    """Display comparison visualizations."""
    st.header("📊 Comparison Visualizations")

    # Row 1: Priority Distribution and Scheduling Status side by side
    col1, col2 = st.columns(2)

    with col1:
        st.subheader("Priority Distribution Comparison")
        fig_priority = create_priority_distribution_plot(
            current_scheduled, comparison_scheduled, current_name, comparison_name
        )
        st.plotly_chart(fig_priority, use_container_width=True)

    with col2:
        st.subheader("Scheduling Status Breakdown")
        fig_status = create_scheduling_status_plot(
            current_common, comparison_common, current_name, comparison_name
        )
        st.plotly_chart(fig_status, use_container_width=True)

    add_vertical_space(1)

    # Plot 3: Changes Flow
    if len(newly_scheduled) > 0 or len(newly_unscheduled) > 0:
        st.subheader("Scheduling Changes")
        fig_changes = create_changes_plot(newly_scheduled, newly_unscheduled)
        st.plotly_chart(fig_changes, use_container_width=True)

    # Plot 4: Time comparison (if available)
    has_time_data = (
        "requested_hours" in current_scheduled.columns
        and "requested_hours" in comparison_scheduled.columns
    )
    if has_time_data:
        add_vertical_space(1)
        st.subheader("Planned Time Distribution")
        fig_time = create_time_distribution_plot(
            current_scheduled, comparison_scheduled, current_name, comparison_name
        )
        st.plotly_chart(fig_time, use_container_width=True)

