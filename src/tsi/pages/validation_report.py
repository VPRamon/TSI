"""Validation Report page - Display data quality and scheduling issues."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.components.validation.validation_issues import render_unified_validation_table
from tsi.components.validation.validation_summary import (
    render_criticality_stats,
    render_summary_metrics,
)
from tsi.services import database as db


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

    try:
        with st.spinner("Loading validation data..."):
            validation_data = db.get_validation_report_data(schedule_id=schedule_id)
    except Exception as exc:
        st.error(f"Failed to load validation data from the backend: {exc}")
        st.exception(exc)
        return

    render_summary_metrics(validation_data)
    st.divider()

    # If everything is valid, show success message
    total_issues = (
        len(validation_data.get("impossible_blocks", []))
        + len(validation_data.get("validation_errors", []))
        + len(validation_data.get("validation_warnings", []))
    )

    if total_issues == 0:
        st.success("✅ All validation checks passed! No issues found in the schedule data.")
        st.info(
            "ℹ️ All blocks in this schedule passed validation. "
            "No impossible blocks were found, and all data is clean."
        )
        return

    # Show info about filtering
    impossible_count = len(validation_data.get("impossible_blocks", []))
    if impossible_count > 0:
        st.warning(
            f"⚠️ **{impossible_count} impossible block(s) found** and automatically excluded from analytics pages. "
            "These blocks have zero visibility or insufficient visibility for the requested observation duration."
        )

    # Show criticality statistics
    render_criticality_stats(validation_data)
    st.divider()

    # Show unified validation table
    render_unified_validation_table(validation_data)
