"""Insights page table display components."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
import streamlit as st

from tsi.components.data.data_preview import render_data_preview

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
