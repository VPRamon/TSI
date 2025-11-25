"""Compare schedules page validation components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.theme import add_vertical_space


def validate_and_display_discrepancies(
    current_df: pd.DataFrame,
    comparison_df: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> tuple[set[str], set[str], set[str]]:
    """
    Validate that both schedules have the same blocks and display discrepancies.
    
    Args:
        current_df: Current schedule DataFrame
        comparison_df: Comparison schedule DataFrame
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule
        
    Returns:
        Tuple of (only_in_current, only_in_comparison, common_ids)
    """
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
    
    return only_in_current, only_in_comparison, common_ids


def compute_scheduling_changes(
    current_common: pd.DataFrame,
    comparison_common: pd.DataFrame,
) -> tuple[pd.DataFrame, pd.DataFrame]:
    """
    Compute blocks with changed scheduling status.
    
    Args:
        current_common: Current schedule with common blocks
        comparison_common: Comparison schedule with common blocks
        
    Returns:
        Tuple of (newly_scheduled, newly_unscheduled) DataFrames
    """
    # Merge on block ID to compare scheduling status
    merged = pd.merge(
        current_common[["schedulingBlockId", "scheduled_flag", "priority"]],
        comparison_common[["schedulingBlockId", "scheduled_flag", "priority"]],
        on="schedulingBlockId",
        suffixes=("_current", "_comparison"),
    )
    
    # Find blocks with changed scheduling status
    newly_scheduled = merged[
        (merged["scheduled_flag_current"] == 0) & (merged["scheduled_flag_comparison"] == 1)
    ]
    newly_unscheduled = merged[
        (merged["scheduled_flag_current"] == 1) & (merged["scheduled_flag_comparison"] == 0)
    ]
    
    return newly_scheduled, newly_unscheduled
