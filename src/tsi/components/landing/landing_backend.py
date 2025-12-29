"""Landing page backend components."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.services import backend_client


def render_backend_section() -> None:
    """Render the backend loading section."""
    st.markdown("### 📊 Load from Backend")
    st.markdown("Select a previously uploaded schedule")

    try:
        schedules = backend_client.list_schedules()

        if not schedules:
            st.info("No schedules available. Upload a schedule first!")
        else:
            schedule_options = {f"{s.name} (ID: {s.id})": s for s in schedules}

            selected_option = st.selectbox(
                "Choose a schedule",
                options=list(schedule_options.keys()),
                key="schedule_selector",
            )

            if selected_option and st.button("Load Schedule", type="primary", key="load_backend_btn"):
                selected_schedule = schedule_options[selected_option]
                load_schedule_from_backend(selected_schedule)

    except Exception as e:
        st.error(f"Failed to list schedules: {e}")


def load_schedule_from_backend(schedule: backend_client.ScheduleSummary) -> None:
    """Persist the selected backend schedule and navigate into the app."""
    try:
        with st.spinner("Loading schedule from backend..."):
            state.set_schedule_ref(schedule.ref)
            state.set_schedule_name(schedule.name)
            state.set_data_filename(schedule.name)
            st.session_state[state.KEY_DATA_SOURCE] = "backend"

            state.set_current_page("Validation")
            st.rerun()

    except Exception as e:
        st.error(f"❌ Error loading schedule: {str(e)}")
        st.exception(e)
