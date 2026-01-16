"""Creative workspace page for building proposals and running scheduling simulations.

This page provides an interactive environment for:
- Building observation proposals with tasks
- Visualizing proposal structure as a tree
- Configuring and running the scheduler
- Chatting with an AI assistant for guidance
"""

from __future__ import annotations

import logging

import streamlit as st

from tsi.components.creative.chat_panel import render_chat_panel
from tsi.components.creative.proposal_builder import render_proposal_builder
from tsi.components.creative.proposal_canvas import render_proposal_canvas
from tsi.components.creative.scheduler_config import render_scheduler_config
from tsi.theme import add_vertical_space

logger = logging.getLogger(__name__)

# Session state keys
KEY_CREATIVE_MODE = "creative_mode"
KEY_CREATIVE_RESULT_READY = "creative_result_ready"
KEY_CREATIVE_RESULT_SCHEDULE_ID = "creative_result_schedule_id"


def render() -> None:
    """
    Render the creative workspace page.
    
    Layout:
    - Left column (1/3): Chat panel for AI assistance
    - Right column (2/3): 
        - Top: Proposal canvas (tree visualization)
        - Middle: Proposal builder panel
        - Bottom: Scheduler configuration and run controls
    """
    # Page header
    st.markdown(
        """
        <div style='text-align: center; margin-bottom: 1rem;'>
            <h1>🎨 Creative Scheduling Workspace</h1>
            <p style='color: #888;'>Build proposals, configure scheduling, and run simulations</p>
        </div>
        """,
        unsafe_allow_html=True,
    )
    
    # Check if we have a result ready to navigate
    if st.session_state.get(KEY_CREATIVE_RESULT_READY, False):
        _render_result_navigation()
    
    # Main layout: Chat on left, Canvas + Controls on right
    col_chat, col_main = st.columns([1, 2])
    
    with col_chat:
        render_chat_panel()
    
    with col_main:
        # Canvas at the top
        render_proposal_canvas()
        
        st.markdown("---")
        
        # Two columns for builder and config
        col_builder, col_config = st.columns(2)
        
        with col_builder:
            render_proposal_builder()
        
        with col_config:
            render_scheduler_config()
    
    # Footer with actions
    _render_footer()


def _render_result_navigation() -> None:
    """Render navigation options after successful scheduling."""
    schedule_id = st.session_state.get(KEY_CREATIVE_RESULT_SCHEDULE_ID)
    
    st.success("✅ Schedule generated successfully!")
    
    col1, col2, col3, col4 = st.columns(4)
    
    with col1:
        if st.button("📊 View Validation", type="primary", width="stretch"):
            _navigate_to_schedule(schedule_id, "Validation")
    
    with col2:
        if st.button("🗺️ View Sky Map", width="stretch"):
            _navigate_to_schedule(schedule_id, "Sky Map")
    
    with col3:
        if st.button("📅 View Schedule", width="stretch"):
            _navigate_to_schedule(schedule_id, "Schedule")
    
    with col4:
        if st.button("🔄 Continue Editing", width="stretch"):
            st.session_state[KEY_CREATIVE_RESULT_READY] = False
            st.rerun()
    
    st.markdown("---")


def _navigate_to_schedule(schedule_id: int | None, page: str) -> None:
    """Navigate to a TSI page with the generated schedule."""
    from tsi import state
    from tsi_rust import ScheduleId
    
    if schedule_id is None:
        st.error("No schedule ID available for navigation")
        return
    
    # Set the schedule reference
    state.set_schedule_ref(ScheduleId(schedule_id))
    state.set_schedule_name(f"Creative Schedule {schedule_id}")
    state.set_data_filename(f"creative_schedule_{schedule_id}")
    
    # Set the target page
    state.set_current_page(page)
    
    # Clear creative result flags
    st.session_state[KEY_CREATIVE_RESULT_READY] = False
    st.session_state[KEY_CREATIVE_MODE] = False
    
    st.rerun()


def _render_footer() -> None:
    """Render footer with additional actions."""
    add_vertical_space(2)
    
    st.markdown("---")
    
    col1, col2, col3 = st.columns([1, 2, 1])
    
    with col1:
        if st.button("🏠 Back to Home", width="stretch"):
            _go_back_home()
    
    with col2:
        st.markdown(
            """
            <div style='text-align: center; color: #666; font-size: 0.9em;'>
                💡 <b>Tip:</b> Use the chat assistant for help with scheduling concepts and proposal building.
            </div>
            """,
            unsafe_allow_html=True,
        )
    
    with col3:
        if st.button("📥 Export Proposals", width="stretch"):
            _export_proposals()


def _go_back_home() -> None:
    """Navigate back to the landing page."""
    from tsi import state
    
    # Clear schedule data to show landing page
    st.session_state["schedule_ref"] = None
    st.session_state[KEY_CREATIVE_MODE] = False
    st.session_state[KEY_CREATIVE_RESULT_READY] = False
    
    st.rerun()


def _export_proposals() -> None:
    """Export proposals to JSON file."""
    import json
    from tsi.components.creative.proposal_builder import export_proposals_to_schedule_json
    
    schedule_data = export_proposals_to_schedule_json()
    
    if not schedule_data.get('schedulingBlocks'):
        st.warning("No proposals to export!")
        return
    
    json_str = json.dumps(schedule_data, indent=2)
    
    st.download_button(
        label="📥 Download JSON",
        data=json_str,
        file_name="creative_proposals.json",
        mime="application/json",
    )


def is_creative_mode() -> bool:
    """Check if creative mode is active."""
    return st.session_state.get(KEY_CREATIVE_MODE, False)


def set_creative_mode(active: bool) -> None:
    """Set creative mode state."""
    st.session_state[KEY_CREATIVE_MODE] = active
