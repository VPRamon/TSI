"""Insights page control components."""

from __future__ import annotations

import streamlit as st

from tsi import state


def render_filter_controls(filter_supported: bool) -> str:
    """
    Render filter controls for insights page.
    
    Args:
        filter_supported: Whether filtering is supported (requires visibility columns)
        
    Returns:
        Selected filter mode ('all' or 'exclude_impossible')
    """
    if state.KEY_INSIGHTS_FILTER_MODE not in st.session_state:
        st.session_state[state.KEY_INSIGHTS_FILTER_MODE] = "all"

    if not filter_supported:
        st.session_state[state.KEY_INSIGHTS_FILTER_MODE] = "all"
        return "all"

    st.markdown("<div style='margin-top: 1.5rem;'></div>", unsafe_allow_html=True)

    filter_options = ("all", "exclude_impossible")
    filter_labels = {
        "all": "📋 All blocks",
        "exclude_impossible": "✅ Filter invalid requests",
    }

    return st.radio(
        "Filtrar:",
        options=filter_options,
        format_func=lambda x: filter_labels[x],
        key=state.KEY_INSIGHTS_FILTER_MODE,
        horizontal=False,
        label_visibility="collapsed",
    )


