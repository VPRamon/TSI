"""Landing page upload components."""

from __future__ import annotations

from typing import Any

import streamlit as st

from tsi.components.landing.landing_database import load_schedule_from_db
from tsi.services import database as db


def render_upload_section() -> None:
    """Render the upload section."""
    st.markdown("### 📤 Upload New Schedule")
    st.markdown("Upload a schedule.json file to store in database")

    uploaded_json = st.file_uploader(
        "Choose a schedule.json file",
        type=["json"],
        help="Upload a raw schedule.json file - will be processed and stored",
        key="json_uploader",
    )

    # Optional: also allow visibility file
    with st.expander("🔍 Add visibility data (optional)", expanded=False):
        uploaded_visibility = st.file_uploader(
            "Choose possible_periods.json (optional)",
            type=["json"],
            help="Optional: upload visibility/possible periods data",
            key="visibility_uploader",
        )

    # Schedule name input
    schedule_name_input = st.text_input(
        "Schedule name",
        placeholder="Enter a name for this schedule",
        key="schedule_name_input",
    )

    if uploaded_json is not None:
        if st.button("Upload", type="primary", key="upload_btn"):
            if not schedule_name_input:
                st.error("Please provide a schedule name")
            else:
                visibility_file = uploaded_visibility if uploaded_visibility is not None else None
                upload_schedule(
                    uploaded_json,
                    schedule_name_input,
                    visibility_file,
                )


def upload_schedule(
    schedule_file: Any,
    schedule_name: str,
    visibility_file: Any | None = None,
) -> None:
    """
    Upload and store a new schedule in the database.

    Args:
        schedule_file: Uploaded schedule.json file
        schedule_name: Name for the schedule
        visibility_file: Optional visibility file
    """
    try:
        with st.spinner("Uploading and processing schedule..."):
            # Read file contents
            schedule_content = schedule_file.read()
            if isinstance(schedule_content, bytes):
                schedule_content = schedule_content.decode("utf-8")

            visibility_content = None
            if visibility_file is not None:
                visibility_content = visibility_file.read()
                if isinstance(visibility_content, bytes):
                    visibility_content = visibility_content.decode("utf-8")

            # Store in database (preprocesses automatically)
            schedule_id = db.store_schedule_db(
                schedule_name=schedule_name,
                schedule_json=schedule_content,
                visibility_json=visibility_content,
            )

            st.success(f"✅ Schedule stored successfully (ID: {schedule_id})")

            # Now load it
            load_schedule_from_db(schedule_id, schedule_name)

    except Exception as e:
        st.error(f"❌ Error uploading schedule: {str(e)}")
        st.exception(e)
