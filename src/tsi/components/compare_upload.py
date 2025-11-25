"""Compare schedules page file upload components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi import state
from tsi.services import prepare_dataframe


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
    
    # Optional visibility file for comparison schedule
    with st.expander("🔍 Add visibility data for comparison schedule (optional)", expanded=False):
        uploaded_visibility = st.file_uploader(
            "Choose possible_periods.json (optional)",
            type=["json"],
            help="Optional: upload visibility/possible periods data for the comparison schedule",
            key="comparison_visibility_uploader",
        )
    
    if uploaded_json is None:
        return None
    
    # Get comparison schedule from session state if already processed
    comparison_df = state.get_comparison_schedule()
    
    # Track if we need to reprocess - include visibility file in token
    vis_token = ""
    if uploaded_visibility is not None:
        vis_token = f":{uploaded_visibility.name}:{uploaded_visibility.size}"
    file_token = f"{uploaded_json.name}:{uploaded_json.size}{vis_token}"
    last_token = st.session_state.get("comparison_file_token")
    
    if comparison_df is None or last_token != file_token:
        # Load and process the comparison schedule
        try:
            with st.spinner("Loading and processing comparison schedule (with visibility data)..."):
                # Use the core loader which supports visibility data merging
                from core.loaders import load_schedule_from_json
                
                # Load with visibility data if provided
                result = load_schedule_from_json(
                    schedule_json=uploaded_json,
                    visibility_json=uploaded_visibility if uploaded_visibility is not None else None,
                    validate=True
                )
                
                comparison_df = result.dataframe
                
                # Show validation warnings if any
                if result.validation.warnings:
                    st.warning(f"⚠️ {len(result.validation.warnings)} data warnings found")
                    with st.expander("View warnings", expanded=False):
                        for warning in result.validation.warnings[:10]:
                            st.warning(f"  - {warning}")
                        if len(result.validation.warnings) > 10:
                            st.info(f"... and {len(result.validation.warnings) - 10} more")
                
                # Convert any list columns to strings BEFORE prepare_dataframe
                for col in comparison_df.columns:
                    if comparison_df[col].dtype == object and len(comparison_df) > 0:
                        sample_val = (
                            comparison_df[col].dropna().iloc[0]
                            if len(comparison_df[col].dropna()) > 0
                            else None
                        )
                        if isinstance(sample_val, list):
                            comparison_df[col] = comparison_df[col].apply(str)
                
                # Apply the same preparation transformations
                comparison_df = prepare_dataframe(comparison_df)
                
                # Store in session state
                state.set_comparison_schedule(comparison_df)
                st.session_state["comparison_file_token"] = file_token
                st.session_state["comparison_filename"] = uploaded_json.name.replace(".json", "")
                
                st.success(f"✅ Processed {len(comparison_df)} scheduling blocks")
        
        except Exception as e:
            st.error(f"❌ Error loading comparison schedule: {str(e)}")
            st.exception(e)
            return None
    
    return comparison_df
