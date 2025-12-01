"""Validation Report page - Display data quality and scheduling issues."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.components.validation.validation_summary import render_summary_metrics, render_criticality_stats
from tsi.components.validation.validation_issues import render_unified_validation_table
from tsi.services.database import get_validation_report_data


def render() -> None:
    """Render the Validation Report page."""
    st.title("⚠️ Validation Report")

    st.markdown(
        """
        Review data quality issues, impossible scheduling blocks, and validation warnings
        to understand potential problems in the uploaded schedule.
        """
    )

    schedule_id = state.get_schedule_id()

    if schedule_id is None:
        st.info("Load a schedule from the database to view the validation report.")
        return

    schedule_id = int(schedule_id)

    try:
        with st.spinner("Loading validation data..."):
            validation_data = get_validation_report_data(schedule_id=schedule_id)
    except Exception as exc:
        st.error(f"Failed to load validation data from the backend: {exc}")
        st.exception(exc)
        return

    # Summary metrics at the top
    render_summary_metrics(validation_data)
    st.divider()

    # If everything is valid, show success message
    total_issues = (
        len(validation_data.get("impossible_blocks", [])) +
        len(validation_data.get("validation_errors", [])) +
        len(validation_data.get("validation_warnings", []))
    )
    
    if total_issues == 0:
        st.success("✅ All validation checks passed! No issues found in the schedule data.")
        return

    # Show criticality statistics
    render_criticality_stats(validation_data)
    st.divider()

    # Show unified validation table
    render_unified_validation_table(validation_data)
