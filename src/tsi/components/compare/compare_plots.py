"""Compare schedules page plotting components."""

from __future__ import annotations

import streamlit as st

from tsi.plots.compare_plots import (
    create_changes_plot,
    create_priority_distribution_plot,
    create_scheduling_status_plot,
    create_time_distribution_plot,
)


def render_comparison_plots(compare_data) -> None:
    """
    Display comparison visualizations from CompareData.

    Args:
        compare_data: CompareData object from Rust backend with pre-computed statistics
    """
    st.header("📊 Comparison Visualizations")

    # Extract scheduled blocks from common blocks
    current_scheduled = [
        b for b in compare_data.current_blocks 
        if b.scheduled and b.scheduling_block_id in compare_data.common_ids
    ]
    comparison_scheduled = [
        b for b in compare_data.comparison_blocks 
        if b.scheduled and b.scheduling_block_id in compare_data.common_ids
    ]
    
    # Get common blocks
    current_common = [
        b for b in compare_data.current_blocks 
        if b.scheduling_block_id in compare_data.common_ids
    ]
    comparison_common = [
        b for b in compare_data.comparison_blocks 
        if b.scheduling_block_id in compare_data.common_ids
    ]
    
    # Extract priority data for plotting
    current_priorities = [b.priority for b in current_scheduled]
    comparison_priorities = [b.priority for b in comparison_scheduled]

    # Row 1: Priority Distribution and Scheduling Status side by side
    col1, col2 = st.columns(2)

    with col1:
        st.subheader("Priority Distribution Comparison")
        fig_priority = create_priority_distribution_plot(
            current_priorities,
            comparison_priorities,
            compare_data.current_name,
            compare_data.comparison_name,
        )
        st.plotly_chart(fig_priority, use_container_width=True)
    
    with col2:
        st.subheader("Scheduling Status Breakdown")
        
        # Calculate counts
        current_scheduled_count = sum(1 for b in current_common if b.scheduled)
        current_unscheduled_count = len(current_common) - current_scheduled_count
        comp_scheduled_count = sum(1 for b in comparison_common if b.scheduled)
        comp_unscheduled_count = len(comparison_common) - comp_scheduled_count
        
        fig_status = create_scheduling_status_plot(
            current_scheduled_count,
            current_unscheduled_count,
            comp_scheduled_count,
            comp_unscheduled_count,
            compare_data.current_name,
            compare_data.comparison_name,
        )
        st.plotly_chart(fig_status, use_container_width=True)

    # Plot 3: Changes Flow
    newly_scheduled_count = sum(
        1 for c in compare_data.scheduling_changes if c.change_type == "newly_scheduled"
    )
    newly_unscheduled_count = sum(
        1 for c in compare_data.scheduling_changes if c.change_type == "newly_unscheduled"
    )
    
    if newly_scheduled_count > 0 or newly_unscheduled_count > 0:
        st.subheader("Scheduling Changes")
        fig_changes = create_changes_plot(newly_scheduled_count, newly_unscheduled_count)
        st.plotly_chart(fig_changes, use_container_width=True)

    # Plot 4: Time comparison (if available)
    has_time_data = any(b.requested_hours > 0 for b in current_scheduled + comparison_scheduled)
    
    if has_time_data:
        st.subheader("Planned Time Distribution")
        current_times = [b.requested_hours for b in current_scheduled]
        comparison_times = [b.requested_hours for b in comparison_scheduled]
        
        fig_time = create_time_distribution_plot(
            current_times,
            comparison_times,
            compare_data.current_name,
            compare_data.comparison_name,
        )
        st.plotly_chart(fig_time, use_container_width=True)
