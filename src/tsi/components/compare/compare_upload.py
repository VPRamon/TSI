"""Compare schedules page file upload and database selection components."""

from __future__ import annotations

import json
from typing import cast

import pandas as pd
import streamlit as st

from tsi import state
from tsi.services.data.loaders import prepare_dataframe
from tsi.services.database import list_schedules_db


def render_file_upload() -> tuple[int | None, str | None, pd.DataFrame | None]:
    """
    Render file upload or database selection UI for comparison schedule.

    Returns:
        Tuple of (schedule_id, schedule_name, dataframe):
        - If database: (schedule_id, schedule_name, None)
        - If file upload: (None, filename, dataframe)
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
        comparison_df, comparison_filename = _render_file_upload_section()
        if comparison_df is not None:
            return (None, comparison_filename, comparison_df)
    
    return (None, None, None)


def _render_database_selection() -> tuple[int | None, str | None]:
    """
    Render the database selection section for comparison schedule.
    
    Returns:
        Tuple of (schedule_id, schedule_name) if selected, (None, None) otherwise
    """
    try:
        schedules = list_schedules_db()
        
        if not schedules:
            st.info("No schedules available in the database.")
            return (None, None)
        
        # Get current schedule ID to filter it out
        current_schedule_id = state.get_schedule_id()
        
        # Filter out the current schedule
        available_schedules = [s for s in schedules if s['schedule_id'] != current_schedule_id]
        
        if not available_schedules:
            st.info("No other schedules available for comparison.")
            return (None, None)
        
        # Create options for selectbox
        schedule_options = {
            f"{s['schedule_name']} (ID: {s['schedule_id']})": (s['schedule_id'], s['schedule_name'])
            for s in available_schedules
        }
        
        selected_option = st.selectbox(
            "Choose a schedule to compare",
            options=list(schedule_options.keys()),
            key="comparison_schedule_selector",
            help="Select a schedule from the database to compare with the current schedule"
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


def _render_file_upload_section() -> tuple[pd.DataFrame | None, str | None]:
    """
    Render the file upload section for comparison schedule.
    
    Returns:
        Tuple of (dataframe, filename) if uploaded, (None, None) otherwise
    """
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

    comparison_df = state.get_comparison_schedule()

    vis_token = ""
    if uploaded_visibility is not None:
        vis_token = f":{uploaded_visibility.name}:{uploaded_visibility.size}"
    file_token = f"{uploaded_json.name}:{uploaded_json.size}{vis_token}"
    last_token = st.session_state.get("comparison_file_token")

    if comparison_df is None or last_token != file_token:
        try:
            with st.spinner("Loading and processing comparison schedule..."):
                # Load JSON content directly using pandas/json
                import json
                
                content = uploaded_json.read()
                if isinstance(content, bytes):
                    content = content.decode("utf-8")
                uploaded_json.seek(0)  # Reset for potential re-read
                
                data = json.loads(content)
                
                # Extract scheduling blocks from the JSON structure
                # Handle multiple possible key names
                if "schedulingBlocks" in data:
                    blocks = data["schedulingBlocks"]
                elif "SchedulingBlock" in data:
                    blocks = data["SchedulingBlock"]
                elif "scheduling_blocks" in data:
                    blocks = data["scheduling_blocks"]
                elif isinstance(data, list):
                    blocks = data
                else:
                    raise ValueError(
                        "Could not find scheduling blocks in JSON. "
                        "Expected one of: 'schedulingBlocks', 'SchedulingBlock', 'scheduling_blocks', or a list"
                    )
                
                # Convert to DataFrame
                comparison_df = pd.DataFrame(blocks)
                
                # Standardize column names
                column_mapping = {
                    "scheduling_block_id": "schedulingBlockId",
                    "schedulingBlockId": "schedulingBlockId",
                    "ra_deg": "raInDeg",
                    "dec_deg": "decInDeg",
                    "raInDeg": "raInDeg",
                    "decInDeg": "decInDeg",
                }
                comparison_df = comparison_df.rename(columns={
                    k: v for k, v in column_mapping.items() if k in comparison_df.columns
                })

                if uploaded_visibility is not None:
                    try:
                        vis_content = uploaded_visibility.read()
                        if isinstance(vis_content, bytes):
                            vis_content = vis_content.decode("utf-8")
                        uploaded_visibility.seek(0)
                        
                        vis_data = json.loads(vis_content)
                        visibility_df = pd.DataFrame(vis_data if isinstance(vis_data, list) else vis_data.get("periods", []))
                        
                        if "schedulingBlockId" in visibility_df.columns:
                            comparison_df = comparison_df.merge(
                                visibility_df,
                                on="schedulingBlockId",
                                how="left",
                                suffixes=("", "_vis"),
                            )
                    except Exception as vis_err:
                        st.warning(f"⚠️ Could not load visibility data: {vis_err}")

                comparison_df = prepare_dataframe(comparison_df)

                state.set_comparison_schedule(comparison_df)
                st.session_state["comparison_file_token"] = file_token
                st.session_state["comparison_filename"] = uploaded_json.name.replace(".json", "")
                st.session_state["comparison_source"] = "file"

                st.success(f"✅ Processed {len(comparison_df)} scheduling blocks")

        except Exception as e:
            st.error(f"❌ Error loading comparison schedule: {str(e)}")
            st.exception(e)
            return (None, None)

    return (cast(pd.DataFrame | None, comparison_df), uploaded_json.name.replace(".json", ""))

