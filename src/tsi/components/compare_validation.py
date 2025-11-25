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
) -> tuple[set, set, set, set]:
    """
    Validate that both schedules have the same blocks and display discrepancies.
    
    Args:
        current_df: Current schedule DataFrame
        comparison_df: Comparison schedule DataFrame
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule
        
    Returns:
        Tuple of (only_in_current, only_in_comparison, common_ids_current, common_ids_comparison)
        where common_ids_current uses current df's ID types and common_ids_comparison uses comparison df's ID types
    """
    # Get block IDs and convert to strings for robust comparison
    # This handles mixed int/string types and prevents false mismatches
    current_ids_raw = current_df["schedulingBlockId"].dropna().unique()
    comparison_ids_raw = comparison_df["schedulingBlockId"].dropna().unique()
    
    # Convert to strings and strip whitespace for comparison
    current_ids_str = {str(x).strip() for x in current_ids_raw}
    comparison_ids_str = {str(x).strip() for x in comparison_ids_raw}
    
    # Find differences using string comparison
    only_in_current_str = current_ids_str - comparison_ids_str
    only_in_comparison_str = comparison_ids_str - current_ids_str
    common_ids_str = current_ids_str & comparison_ids_str
    
    # Map back to original values for filtering DataFrames
    # Create mapping from string representation to original value
    current_id_map = {str(x).strip(): x for x in current_ids_raw}
    comparison_id_map = {str(x).strip(): x for x in comparison_ids_raw}
    
    # Convert string sets back to original types for DataFrame filtering
    only_in_current = {current_id_map[s] for s in only_in_current_str}
    only_in_comparison = {comparison_id_map[s] for s in only_in_comparison_str}
    
    # For common IDs, create separate sets with the correct type for each DataFrame
    common_ids_current = {current_id_map[s] for s in common_ids_str}
    common_ids_comparison = {comparison_id_map[s] for s in common_ids_str}
    
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
        
        st.info(f"**Common blocks:** {len(common_ids_current)} blocks will be used for comparison")
        
        add_vertical_space(1)
        st.divider()
    
    return only_in_current, only_in_comparison, common_ids_current, common_ids_comparison


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
    # Ensure schedulingBlockId has the same type in both dataframes before merging
    current_subset = current_common[["schedulingBlockId", "scheduled_flag", "priority"]].copy()
    comparison_subset = comparison_common[["schedulingBlockId", "scheduled_flag", "priority"]].copy()
    
    # Convert both to string for consistent merging (handles int64/object mismatch)
    current_subset["schedulingBlockId"] = current_subset["schedulingBlockId"].astype(str)
    comparison_subset["schedulingBlockId"] = comparison_subset["schedulingBlockId"].astype(str)
    
    # Merge on block ID to compare scheduling status
    merged = pd.merge(
        current_subset,
        comparison_subset,
        on="schedulingBlockId",
        suffixes=("_current", "_comparison"),
    )
    
    # Convert to boolean to handle both boolean and integer types
    merged["scheduled_flag_current"] = merged["scheduled_flag_current"].astype(bool)
    merged["scheduled_flag_comparison"] = merged["scheduled_flag_comparison"].astype(bool)
    
    # Find blocks with changed scheduling status
    newly_scheduled = merged[
        (~merged["scheduled_flag_current"]) & (merged["scheduled_flag_comparison"])
    ]
    newly_unscheduled = merged[
        (merged["scheduled_flag_current"]) & (~merged["scheduled_flag_comparison"])
    ]
    
    return newly_scheduled, newly_unscheduled
