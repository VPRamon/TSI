"""Landing page for the Telescope Scheduling Intelligence app."""

from __future__ import annotations

import streamlit as st

from tsi.components.landing.landing_database import render_database_section
from tsi.components.landing.landing_upload import render_upload_section
from tsi.services.database import init_database
from tsi.theme import add_vertical_space, render_landing_title


def render() -> None:
    """Render the landing page with data selection options."""

    # Add significant vertical space at the top
    add_vertical_space(4)

    # Initialize database connection
    try:
        init_database()
    except Exception as e:
        st.error(f"Failed to initialize database: {e}")
        st.info("Please ensure DATABASE_URL environment variable is set")
        return

    render_landing_title("Telescope Scheduling Intelligence")

    st.markdown(
        """
        <div style='text-align: center; font-size: 1.4rem; color: #888; margin-bottom: 4rem; font-weight: 300;'>
        Analyze and visualize telescope scheduling data with interactive insights
        </div>
        """,
        unsafe_allow_html=True,
    )

    add_vertical_space(3)

    # Two-column layout for data selection
    col1, col2 = st.columns(2)

    with col1:
        render_database_section()

    with col2:
        render_upload_section()
