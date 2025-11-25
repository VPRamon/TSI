"""Compare schedules page file upload components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi import state
from tsi.services.loaders import load_schedule_rust, prepare_dataframe


def render_file_upload() -> pd.DataFrame | None:
    """
    Render file upload UI and process comparison schedule.
    
    Returns:
        Comparison DataFrame if loaded successfully, None otherwise
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
        return None
    
    comparison_df = state.get_comparison_schedule()
    
    vis_token = ""
    if uploaded_visibility is not None:
        vis_token = f":{uploaded_visibility.name}:{uploaded_visibility.size}"
    file_token = f"{uploaded_json.name}:{uploaded_json.size}{vis_token}"
    last_token = st.session_state.get("comparison_file_token")
    
    if comparison_df is None or last_token != file_token:
        try:
            with st.spinner("Loading and processing comparison schedule..."):
                comparison_df = load_schedule_rust(uploaded_json, format="json")
                
                if uploaded_visibility is not None:
                    try:
                        visibility_df = load_schedule_rust(uploaded_visibility, format="json")
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
                
                st.success(f"✅ Processed {len(comparison_df)} scheduling blocks")
        
        except Exception as e:
            st.error(f"❌ Error loading comparison schedule: {str(e)}")
            st.exception(e)
            return None
    
    return comparison_df

