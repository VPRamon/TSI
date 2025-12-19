"""Compare schedules page file upload and database selection components."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.services import database as db


def render_file_upload() -> tuple[int | None, str | None, None]:
    """
    Render file upload or database selection UI for comparison schedule.

    Returns:
        Tuple of (schedule_id, schedule_name, None):
        - If database or file upload: (schedule_id, schedule_name, None)
        - If nothing selected: (None, None, None)
    """
    st.subheader("Select Comparison Schedule")

    # Create tabs for database vs file upload
    tab_db, tab_file = st.tabs(["📊 From Database", "📁 Upload File"])

    with tab_db:
        comparison_id, comparison_name = _render_database_selection()
        if comparison_id is not None:
            return (comparison_id, comparison_name, None)

    with tab_file:
        comparison_id, comparison_name = _render_file_upload_section()
        if comparison_id is not None:
            return (comparison_id, comparison_name, None)

    return (None, None, None)


def _render_database_selection() -> tuple[int | None, str | None]:
    """
    Render the database selection section for comparison schedule.

    Returns:
        Tuple of (schedule_id, schedule_name) if selected, (None, None) otherwise
    """
    try:
        schedules = db.list_schedules_db()

        if not schedules:
            st.info("No schedules available in the database.")
            return (None, None)

        # Get current schedule ID to filter it out
        current_schedule_id = state.get_schedule_id()

        # Filter out the current schedule
        available_schedules = [s for s in schedules if s["schedule_id"] != current_schedule_id]

        if not available_schedules:
            st.info("No other schedules available for comparison.")
            return (None, None)

        # Create options for selectbox
        schedule_options = {
            f"{s['schedule_name']} (ID: {s['schedule_id']})": (s["schedule_id"], s["schedule_name"])
            for s in available_schedules
        }

        selected_option = st.selectbox(
            "Choose a schedule to compare",
            options=list(schedule_options.keys()),
            key="comparison_schedule_selector",
            help="Select a schedule from the database to compare with the current schedule",
        )

        if selected_option:
            schedule_id, schedule_name = schedule_options[selected_option]

            # Store in session state
            if st.session_state.get("comparison_schedule_id") != schedule_id:
                st.session_state["comparison_schedule_id"] = schedule_id
                st.session_state["comparison_filename"] = schedule_name
                st.session_state["comparison_source"] = "database"
                st.rerun()

            return (schedule_id, schedule_name)

    except Exception as e:
        st.error(f"Failed to list schedules: {e}")
        return (None, None)

    return (None, None)


def _render_file_upload_section() -> tuple[int | None, str | None]:
    """
    Render the file upload section for comparison schedule.

    When a file is uploaded, it is stored in the database and the schedule_id is returned.

    Returns:
        Tuple of (schedule_id, schedule_name) if uploaded, (None, None) otherwise
    """
    from tsi.services import database as db

    uploaded_json = st.file_uploader(
        "Choose a schedule.json file to compare",
        type=["json"],
        help="Upload a schedule.json file to compare with the current schedule",
        key="comparison_json_uploader",
    )

    with st.expander("🔍 Add visibility data for comparison schedule (optional)", expanded=False):
        uploaded_visibility = st.file_uploader(
            "Choose possible_periods.json (optional)",
            type=["json"],
            help="Optional: upload visibility/possible periods data for the comparison schedule",
            key="comparison_visibility_uploader",
        )

    if uploaded_json is None:
        return (None, None)

    # Generate a file token to track changes
    vis_token = ""
    if uploaded_visibility is not None:
        vis_token = f":{uploaded_visibility.name}:{uploaded_visibility.size}"
    file_token = f"{uploaded_json.name}:{uploaded_json.size}{vis_token}"
    st.session_state.get("comparison_file_token")

    # Check if we already processed this file
    if st.session_state.get("comparison_file_token") == file_token:
        schedule_id = st.session_state.get("comparison_schedule_id")
        schedule_name = st.session_state.get("comparison_filename")
        if schedule_id is not None:
            return (schedule_id, schedule_name)

    # Process and store the new file
    try:
        with st.spinner("Uploading and processing comparison schedule..."):
            # Read file contents
            schedule_content_raw = uploaded_json.read()
            schedule_content: str
            if isinstance(schedule_content_raw, bytes):
                schedule_content = schedule_content_raw.decode("utf-8")
            else:
                schedule_content = schedule_content_raw
            uploaded_json.seek(0)  # Reset for potential re-read

            visibility_content: str | None = None
            if uploaded_visibility is not None:
                visibility_content_raw = uploaded_visibility.read()
                if isinstance(visibility_content_raw, bytes):
                    visibility_content = visibility_content_raw.decode("utf-8")
                else:
                    visibility_content = visibility_content_raw
                uploaded_visibility.seek(0)

            # Store in database (preprocesses automatically)
            schedule_name = uploaded_json.name.replace(".json", "") + "_comparison"
            metadata = db.store_schedule_db(
                schedule_name=schedule_name,
                schedule_json=schedule_content,
                visibility_json=visibility_content,
            )

            schedule_id = metadata["schedule_id"]

            # Store in session state
            st.session_state["comparison_file_token"] = file_token
            st.session_state["comparison_schedule_id"] = schedule_id
            st.session_state["comparison_filename"] = schedule_name
            st.session_state["comparison_source"] = "file"

            st.success(f"✅ Schedule uploaded successfully (ID: {schedule_id})")

            return (schedule_id, schedule_name)

    except Exception as e:
        st.error(f"❌ Error uploading comparison schedule: {str(e)}")
        st.exception(e)
        return (None, None)
