"""Schedule Comparison page for comparing two schedules."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi import state
from tsi.components.compare.compare_plots import render_comparison_plots
from tsi.components.compare.compare_tables import render_comparison_tables
from tsi.components.compare.compare_upload import render_file_upload
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
                st.warning(
                    f"**Blocks only in {compare_data.current_name}:** {len(compare_data.only_in_current)}"
                )
                with st.expander(
                    f"View {len(compare_data.only_in_current)} blocks", expanded=False
                ):
                    st.dataframe(
                        pd.DataFrame({"schedulingBlockId": sorted(compare_data.only_in_current)}),
                        hide_index=True,
                        height=200,
                        use_container_width=True,
                    )

        with col2:
            if len(compare_data.only_in_comparison) > 0:
                st.warning(
                    f"**Blocks only in {compare_data.comparison_name}:** {len(compare_data.only_in_comparison)}"
                )
                with st.expander(
                    f"View {len(compare_data.only_in_comparison)} blocks", expanded=False
                ):
                    st.dataframe(
                        pd.DataFrame(
                            {"schedulingBlockId": sorted(compare_data.only_in_comparison)}
                        ),
                        hide_index=True,
                        height=200,
                        use_container_width=True,
                    )

        st.info(
            f"**Common blocks:** {len(compare_data.common_ids)} blocks will be used for comparison"
        )

        add_vertical_space(1)
        st.divider()

    if len(compare_data.common_ids) == 0:
        st.error("❌ No common blocks found. Cannot perform comparison.")
        return

    # Display comparison tables
    render_comparison_tables(compare_data)

    st.divider()

    # Display visualizations
    render_comparison_plots(compare_data)
