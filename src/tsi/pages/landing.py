"""Landing page for the Telescope Scheduling Intelligence app."""

from __future__ import annotations

import logging

import streamlit as st

from app_config import get_settings
from tsi.components.landing.landing_backend import render_schedules_section
from tsi.components.landing.landing_creative import render_creative_section
from tsi.components.landing.landing_upload import render_upload_section
from tsi.theme import add_vertical_space, render_landing_title

logger = logging.getLogger(__name__)


def render() -> None:
    """Render the landing page with data selection options."""
    get_settings()

    # Add significant vertical space at the top
    add_vertical_space(4)

    render_landing_title()  # Uses APP_TITLE from config

    st.markdown(
        """
        <div style='text-align: center; font-size: 1.4rem; color: #888; margin-bottom: 4rem; font-weight: 300;'>
        Analyze and visualize telescope scheduling data with interactive insights
        </div>
        """,
        unsafe_allow_html=True,
    )

    add_vertical_space(3)

    # Three-column layout for data selection options
    col1, col2, col3 = st.columns(3)

    with col1:
        render_schedules_section()

    with col2:
        render_upload_section()

    with col3:
        render_creative_section()
