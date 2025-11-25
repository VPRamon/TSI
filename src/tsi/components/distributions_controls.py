"""Distribution page control components for filtering."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi import state

FILTER_OPTIONS = ("all", "exclude_impossible")
FILTER_LABELS = {
    "all": "📋 All blocks",
    "exclude_impossible": "✅ Filter invalid requests",
}


def render_filter_control(df: pd.DataFrame) -> tuple[str, bool]:
    """
    Render filter control for impossible observations.

    Args:
        df: The prepared DataFrame

    Returns:
        Tuple of (filter_mode, filter_supported)
    """
    # Initialize session state only if not present
    if state.KEY_DIST_FILTER_MODE not in st.session_state:
        st.session_state[state.KEY_DIST_FILTER_MODE] = "all"

    filter_supported = _check_filter_support(df)

    # Render filter control if supported
    filter_mode = "all"
    if filter_supported:
        # Add empty space to align vertically with title
        st.markdown("<div style='margin-top: 1.5rem;'></div>", unsafe_allow_html=True)
        filter_mode = st.radio(
            "Filtrar:",
            options=FILTER_OPTIONS,
            format_func=lambda x: FILTER_LABELS[x],
            key=state.KEY_DIST_FILTER_MODE,
            horizontal=False,
            label_visibility="collapsed",
        )
    else:
        st.session_state[state.KEY_DIST_FILTER_MODE] = "all"

    return filter_mode, filter_supported


def _check_filter_support(df: pd.DataFrame) -> bool:
    """
    Check if filtering by impossible observations is supported.

    Args:
        df: The prepared DataFrame

    Returns:
        True if filter is supported
    """
    return "total_visibility_hours" in df.columns and (
        "minObservationTimeInSec" in df.columns or "requested_hours" in df.columns
    )
