"""Insights page table display components."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
import streamlit as st

from tsi.components.data.data_preview import render_conflicts_table, render_data_preview

if TYPE_CHECKING:
    from typing import Any


def render_top_observations(
    top_priority: list[Any],
    top_visibility: list[Any],
) -> None:
    """
    Display top observations in tabbed layout.

    Args:
        top_priority: Top observations by priority (list of TopObservation objects)
        top_visibility: Top observations by visibility hours (list of TopObservation objects)
    """
    st.header("🏆 Top Observations")

    tab1, tab2 = st.tabs(["By Priority", "By Visibility Hours"])

    with tab1:
        if top_priority:
            # Convert Rust TopObservation objects to DataFrame
            df = pd.DataFrame(
                [
                    {
                        "scheduling_block_id": obs.scheduling_block_id,
                        "priority": obs.priority,
                        "total_visibility_hours": obs.total_visibility_hours,
                        "requested_hours": obs.requested_hours,
                        "scheduled": obs.scheduled,
                    }
                    for obs in top_priority
                ]
            )
            render_data_preview(
                df,
                max_rows=10,
                title="Top 10 by Priority",
            )
        else:
            st.info("No data available")

    with tab2:
        if top_visibility:
            # Convert Rust TopObservation objects to DataFrame
            df = pd.DataFrame(
                [
                    {
                        "scheduling_block_id": obs.scheduling_block_id,
                        "priority": obs.priority,
                        "total_visibility_hours": obs.total_visibility_hours,
                        "requested_hours": obs.requested_hours,
                        "scheduled": obs.scheduled,
                    }
                    for obs in top_visibility
                ]
            )
            render_data_preview(
                df,
                max_rows=10,
                title="Top 10 by Total Visibility Hours",
            )
        else:
            st.info("No data available")


def render_integrity_checks(conflicts: list[Any]) -> None:
    """
    Display scheduling integrity checks and conflict information.

    Args:
        conflicts: List of ConflictRecord objects
    """
    st.header("🔎 Scheduling Integrity")

    # Convert Rust ConflictRecord objects to DataFrame
    if conflicts:
        conflicts_df = pd.DataFrame(
            [
                {
                    "block_id_1": conflict.block_id_1,
                    "block_id_2": conflict.block_id_2,
                    "start_time_1": conflict.start_time_1,
                    "stop_time_1": conflict.stop_time_1,
                    "start_time_2": conflict.start_time_2,
                    "stop_time_2": conflict.stop_time_2,
                    "overlap_hours": conflict.overlap_hours,
                }
                for conflict in conflicts
            ]
        )
    else:
        conflicts_df = pd.DataFrame()

    render_conflicts_table(conflicts_df)

    if not conflicts_df.empty:
        st.warning(
            f"""
            **⚠️ {len(conflicts_df)} scheduling conflicts detected!**

            These observations have overlapping scheduled times, which may indicate:
            - Double-booking of telescope time
            - Scheduling algorithm errors
            - Data integrity issues

            Review the conflict details above and adjust schedules accordingly.
            """
        )
    else:
        st.success(
            "✅ No scheduling conflicts detected. All scheduled observations are properly sequenced."
        )
