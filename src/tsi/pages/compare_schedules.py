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
from tsi.services.database import get_compare_data, compute_compare_data
from tsi.services.data.loaders import prepare_dataframe
from tsi.theme import add_vertical_space


def _convert_compare_blocks_to_df(blocks) -> pd.DataFrame:
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


def render() -> None:
    """Render the Schedule Comparison page."""
    st.title("⚖️ Compare Schedules")

    st.markdown(
        """
        Compare the current schedule with another one from the database or upload a new one.
        View differences in scheduled blocks, priority distributions, and planned time.
        """
    )

    # Get current schedule ID and name
    current_schedule_id = state.get_schedule_id()
    current_name = state.get_schedule_name() or state.get_data_filename() or "Current"

    if current_schedule_id is None:
        st.warning("⚠️ No base schedule loaded. Please load a schedule from the landing page first.")
        return

    # Handle comparison schedule selection (database or file upload)
    comparison_schedule_id, comparison_name, comparison_df = render_file_upload()

    if comparison_schedule_id is None and comparison_df is None:
        st.info("👆 Select a schedule from the database or upload a file to compare")
        return

    # Get comparison data from Rust backend
    try:
        with st.spinner("Computing comparison..."):
            if comparison_schedule_id is not None:
                # Both schedules are in the database - use Rust backend directly
                compare_data = get_compare_data(
                    current_schedule_id=int(current_schedule_id),
                    comparison_schedule_id=int(comparison_schedule_id),
                    current_name=current_name,
                    comparison_name=comparison_name or "Comparison",
                )
            elif comparison_df is not None:
                # Comparison schedule is from file upload - we need to convert it to CompareBlocks
                # and use compute_compare_data
                try:
                    import tsi_rust
                except ImportError:
                    st.error("Rust backend not available. Cannot compute comparison.")
                    return
                
                # Fetch current schedule blocks from database
                from tsi.services.database import _rust_call
                current_blocks = _rust_call("py_fetch_compare_blocks", int(current_schedule_id))
                
                # Convert comparison DataFrame to CompareBlock format
                comparison_blocks = []
                for _, row in comparison_df.iterrows():
                    # Create a CompareBlock-like dict that can be passed to Rust
                    # We'll need to create these manually since we can't instantiate CompareBlock directly
                    block_dict = {
                        "scheduling_block_id": str(row["schedulingBlockId"]),
                        "priority": float(row.get("priority", 0.0)),
                        "scheduled": bool(row.get("scheduled_flag", False)),
                        "requested_hours": float(row.get("requested_hours", 0.0)),
                    }
                    comparison_blocks.append(block_dict)
                
                # For now, fall back to pandas-based comparison if file upload
                # TODO: Add proper file-to-CompareBlock conversion in Rust
                st.warning("File upload comparison is not yet fully optimized. Using legacy comparison.")
                _display_comparison_legacy(
                    current_df=state.get_prepared_data(),
                    comparison_df=comparison_df,
                    current_name=current_name,
                    comparison_name=comparison_name or "Uploaded Schedule",
                )
                return
            else:
                return
    
    except Exception as e:
        st.error(f"Failed to compute comparison: {e}")
        st.exception(e)
        return

    # Convert CompareData to DataFrames for compatibility with existing components
    current_df = _convert_compare_blocks_to_df(compare_data.current_blocks)
    comparison_df_converted = _convert_compare_blocks_to_df(compare_data.comparison_blocks)

    # Display comparison
    _display_comparison(
        current_df=current_df,
        comparison_df=comparison_df_converted,
        compare_data=compare_data,
    )


def _display_comparison(
    current_df: pd.DataFrame,
    comparison_df: pd.DataFrame,
    compare_data,
) -> None:
    """
    Display comparison between two schedules using pre-computed CompareData.

    Args:
        current_df: Current schedule DataFrame (converted from CompareBlocks)
        comparison_df: Comparison schedule DataFrame (converted from CompareBlocks)
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
                        width='stretch',
                    )

        with col2:
            if len(compare_data.only_in_comparison) > 0:
                st.warning(f"**Blocks only in {compare_data.comparison_name}:** {len(compare_data.only_in_comparison)}")
                with st.expander(f"View {len(compare_data.only_in_comparison)} blocks", expanded=False):
                    st.dataframe(
                        pd.DataFrame({"schedulingBlockId": sorted(compare_data.only_in_comparison)}),
                        hide_index=True,
                        height=200,
                        width='stretch',
                    )

        st.info(f"**Common blocks:** {len(compare_data.common_ids)} blocks will be used for comparison")

        add_vertical_space(1)
        st.divider()

    if len(compare_data.common_ids) == 0:
        st.error("❌ No common blocks found. Cannot perform comparison.")
        return

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
        st.plotly_chart(fig_priority, width='stretch')
    
    with col2:
        st.subheader("Scheduling Status Breakdown")
        fig_status = create_scheduling_status_plot(
            current_common, comparison_common, current_name, comparison_name
        )
        st.plotly_chart(fig_status, width='stretch')
    
    add_vertical_space(1)

    # Plot 3: Changes Flow
    if len(newly_scheduled) > 0 or len(newly_unscheduled) > 0:
        st.subheader("Scheduling Changes")
        fig_changes = create_changes_plot(newly_scheduled, newly_unscheduled)
        st.plotly_chart(fig_changes, width='stretch')

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
        st.plotly_chart(fig_time, width='stretch')


def _display_comparison_legacy(
    current_df: pd.DataFrame,
    comparison_df: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> None:
    """
    Legacy comparison display for file uploads (pandas-based).
    This function uses the old validation logic for file-based comparisons.
    
    Args:
        current_df: Current schedule DataFrame
        comparison_df: Comparison schedule DataFrame
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule
    """
    from tsi.components.compare.compare_validation import (
        compute_scheduling_changes,
        validate_and_display_discrepancies,
    )
    
    st.header("🔍 Schedule Comparison")

    # Validate and display discrepancies
    only_in_current, only_in_comparison, common_ids_current, common_ids_comparison = (
        validate_and_display_discrepancies(current_df, comparison_df, current_name, comparison_name)
    )

    if len(common_ids_current) == 0:
        st.error("❌ No common blocks found. Cannot perform comparison.")
        return

    # Filter to common blocks - use the appropriate ID set for each dataframe
    current_common = current_df[current_df["schedulingBlockId"].isin(common_ids_current)]
    comparison_common = comparison_df[
        comparison_df["schedulingBlockId"].isin(common_ids_comparison)
    ]

    # Filter to scheduled blocks (handle both boolean and integer types)
    current_scheduled = current_common[current_common["scheduled_flag"].astype(bool)]
    comparison_scheduled = comparison_common[comparison_common["scheduled_flag"].astype(bool)]

    # Find scheduling changes
    newly_scheduled, newly_unscheduled = compute_scheduling_changes(
        current_common, comparison_common
    )

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
