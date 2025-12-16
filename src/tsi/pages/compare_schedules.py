"""Schedule Comparison page for comparing two schedules."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi import state
from tsi.components.compare.compare_tables import render_comparison_tables
from tsi.components.compare.compare_upload import render_file_upload
from tsi.plots.compare_plots import (
    create_changes_plot,
    create_priority_distribution_plot,
    create_scheduling_status_plot,
    create_time_distribution_plot,
)
from tsi.services import database as db
from tsi.theme import add_vertical_space


def render() -> None:
    """Render the Schedule Comparison page."""
    st.title("⚖️ Compare Schedules")

    st.markdown(
        """
        Compare the current schedule with another one from the database or upload a new one.
        View differences in scheduled blocks, priority distributions, and planned time.
        """
    )

    # Get current schedule information
    current_schedule_id = state.get_schedule_id()
    current_name = state.get_schedule_name() or state.get_data_filename() or "Current"

    # Handle comparison schedule selection (database or file upload)
    # The upload component handles storing uploaded files to the database
    comparison_schedule_id, comparison_name, _ = render_file_upload()

    if comparison_schedule_id is None:
        st.info("👆 Select a schedule from the database or upload a file to compare")
        return

    # Get comparison data from Rust backend
    try:
        with st.spinner("Computing comparison..."):
            compare_data = db.get_compare_data(
                current_schedule_id=int(current_schedule_id),
                comparison_schedule_id=int(comparison_schedule_id),
                current_name=current_name,
                comparison_name=comparison_name or "Comparison",
            )
    except Exception as e:
        st.error(f"Failed to compute comparison: {e}")
        st.exception(e)
        return

    # Display comparison results
    _display_comparison(compare_data)


def _display_comparison(compare_data) -> None:
    """
    Display comparison between two schedules using pre-computed CompareData.

    Args:
        compare_data: CompareData object from Rust backend with pre-computed statistics
    """
    st.header("🔍 Schedule Comparison")

    # Display discrepancies if any
    if len(compare_data.only_in_current) > 0 or len(compare_data.only_in_comparison) > 0:
        st.error("⚠️ **Discrepancy Warning!** The schedules contain different sets of blocks.")

        col1, col2 = st.columns(2)

        with col1:
            if len(compare_data.only_in_current) > 0:
                st.warning(f"**Blocks only in {compare_data.current_name}:** {len(compare_data.only_in_current)}")
                with st.expander(f"View {len(compare_data.only_in_current)} blocks", expanded=False):
                    st.dataframe(
                        pd.DataFrame({"schedulingBlockId": sorted(compare_data.only_in_current)}),
                        hide_index=True,
                        height=200,
                        use_container_width=True,
                    )

        with col2:
            if len(compare_data.only_in_comparison) > 0:
                st.warning(f"**Blocks only in {compare_data.comparison_name}:** {len(compare_data.only_in_comparison)}")
                with st.expander(f"View {len(compare_data.only_in_comparison)} blocks", expanded=False):
                    st.dataframe(
                        pd.DataFrame({"schedulingBlockId": sorted(compare_data.only_in_comparison)}),
                        hide_index=True,
                        height=200,
                        use_container_width=True,
                    )

        st.info(f"**Common blocks:** {len(compare_data.common_ids)} blocks will be used for comparison")

        add_vertical_space(1)
        st.divider()

    if len(compare_data.common_ids) == 0:
        st.error("❌ No common blocks found. Cannot perform comparison.")
        return

    # Convert CompareData to DataFrames for components
    current_df = _convert_blocks_to_df(compare_data.current_blocks)
    comparison_df = _convert_blocks_to_df(compare_data.comparison_blocks)

    # Filter to common blocks
    current_common = current_df[current_df["schedulingBlockId"].isin(compare_data.common_ids)]
    comparison_common = comparison_df[comparison_df["schedulingBlockId"].isin(compare_data.common_ids)]

    # Filter to scheduled blocks
    current_scheduled = current_common[current_common["scheduled_flag"] == 1]
    comparison_scheduled = comparison_common[comparison_common["scheduled_flag"] == 1]

    # Convert scheduling changes to DataFrames
    newly_scheduled = pd.DataFrame([
        {
            "schedulingBlockId": change.scheduling_block_id,
            "priority_current": change.priority,
        }
        for change in compare_data.scheduling_changes
        if change.change_type == "newly_scheduled"
    ])

    newly_unscheduled = pd.DataFrame([
        {
            "schedulingBlockId": change.scheduling_block_id,
            "priority_current": change.priority,
        }
        for change in compare_data.scheduling_changes
        if change.change_type == "newly_unscheduled"
    ])

    # Display comparison tables
    render_comparison_tables(
        current_scheduled=current_scheduled,
        comparison_scheduled=comparison_scheduled,
        current_common=current_common,
        comparison_common=comparison_common,
        newly_scheduled=newly_scheduled,
        newly_unscheduled=newly_unscheduled,
        current_name=compare_data.current_name,
        comparison_name=compare_data.comparison_name,
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
        current_name=compare_data.current_name,
        comparison_name=compare_data.comparison_name,
    )


def _convert_blocks_to_df(blocks) -> pd.DataFrame:
    """Convert list of CompareBlock objects to pandas DataFrame."""
    data = []
    for block in blocks:
        data.append({
            "schedulingBlockId": block.scheduling_block_id,
            "priority": block.priority,
            "scheduled_flag": 1 if block.scheduled else 0,
            "requested_hours": block.requested_hours,
        })
    return pd.DataFrame(data)


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
