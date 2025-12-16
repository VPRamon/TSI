"""Landing page for the Telescope Scheduling Intelligence app."""

from __future__ import annotations

import logging
import streamlit as st

from app_config import get_settings
from tsi.components.landing.landing_database import render_database_section
from tsi.components.landing.landing_upload import render_upload_section
from tsi.exceptions import ServerError
from tsi.services.database import init_database
from tsi.theme import add_vertical_space, render_landing_title

logger = logging.getLogger(__name__)


def render() -> None:
    """Render the landing page with data selection options."""
    settings = get_settings()

    # Add significant vertical space at the top
    add_vertical_space(4)

    # Initialize database connection
    try:
        if settings.enable_database:
            init_database()
        else:
            st.info("Database features are disabled in configuration")
    except ServerError as e:
        # Log detailed error for debugging
        logger.error(f"Server error during initialization: {e}", exc_info=True, extra={"details": e.details})
        # Show user-friendly message
        st.error(e.to_user_message())
        st.info("If the problem persists, please contact support or check the application logs.")
        return
    except Exception as e:
        # Log unexpected errors with full details
        logger.error(f"Unexpected error during initialization: {e}", exc_info=True)
        # Show generic message to user
        st.error("An unexpected error occurred. Please try again later.")
        st.info("Please check application logs for more details")
        return

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

    # Two-column layout for data selection
    col1, col2 = st.columns(2)

    with col1:
        render_database_section()

    with col2:
        render_upload_section()
