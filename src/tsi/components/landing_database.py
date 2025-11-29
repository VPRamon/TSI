"""Landing page database components."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.services.loaders import prepare_dataframe
from tsi.services.database import (
    fetch_dark_periods_db,
    fetch_schedule_db,
    list_schedules_db,
)


def render_database_section() -> None:
    """Render the database loading section."""
    st.markdown("### 📊 Load from Database")
    st.markdown("Select a previously uploaded schedule")

    try:
        schedules = list_schedules_db()
        
        if not schedules:
            st.info("No schedules available. Upload a schedule first!")
        else:
            # Create options for selectbox
            schedule_options = {
                f"{s['schedule_name']} (ID: {s['schedule_id']})": s['schedule_id']
                for s in schedules
            }
            
            selected_option = st.selectbox(
                "Choose a schedule",
                options=list(schedule_options.keys()),
                key="schedule_selector",
            )
            
            if selected_option and st.button("Load Schedule", type="primary", key="load_db_btn"):
                schedule_id = schedule_options[selected_option]
                # Find the schedule info
                schedule_info = next(s for s in schedules if s['schedule_id'] == schedule_id)
                load_schedule_from_db(schedule_id, schedule_info['schedule_name'])
    
    except Exception as e:
        st.error(f"Failed to list schedules: {e}")


def load_schedule_from_db(schedule_id: int, schedule_name: str) -> None:
    """
    Load a schedule from the database.

    Args:
        schedule_id: Database schedule ID
        schedule_name: Name of the schedule
    """
    try:
        with st.spinner("Loading schedule from database..."):
            # Fetch schedule data
            raw_df = fetch_schedule_db(schedule_id=schedule_id)

            # Prepare and enrich
            prepared_df = prepare_dataframe(raw_df)

            # Store in session state
            state.set_prepared_data(prepared_df)
            state.set_schedule_id(schedule_id)
            state.set_schedule_name(schedule_name)
            state.set_data_filename(schedule_name)
            st.session_state[state.KEY_DATA_SOURCE] = "database"

            # Load dark periods for this schedule (with global fallback)
            try:
                dark_periods_df = fetch_dark_periods_db(schedule_id)
                if not dark_periods_df.empty:
                    state.set_dark_periods(dark_periods_df)
            except Exception as e:
                st.warning(f"Could not load dark periods: {e}")

            # Auto-navigate to first page
            state.set_current_page("Sky Map")

            # Force rerun to navigate away from landing page
            st.rerun()

    except Exception as e:
        st.error(f"❌ Error loading schedule: {str(e)}")
        st.exception(e)
