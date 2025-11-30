"""Landing page for the Telescope Scheduling Intelligence app."""

from __future__ import annotations

import streamlit as st

from app_config import get_settings
from tsi.components.landing.landing_database import render_database_section
from tsi.components.landing.landing_upload import render_upload_section
from tsi.exceptions import DatabaseConnectionError, ConfigurationError
from tsi.services.database import init_database
from tsi.theme import add_vertical_space, render_landing_title


def render() -> None:
    """Render the landing page with data selection options."""
    settings = get_settings()

    # Add significant vertical space at the top
    add_vertical_space(4)

    # Initialize database connection
    try:
        if settings.enable_database:
            if not settings.validate_database_config():
                raise ConfigurationError(
                    "Database configuration is incomplete. "
                    "Please set DATABASE_URL or individual DB_* environment variables."
                )
            init_database()
        else:
            st.info("Database features are disabled in configuration")
    except ConfigurationError as e:
        st.error(f"Configuration Error: {e.message}")
        st.info(
            "Please ensure DATABASE_URL environment variable is set, or configure:\n"
            "- DB_SERVER\n"
            "- DB_DATABASE\n"
            "- DB_USERNAME\n"
            "- DB_PASSWORD"
        )
        return
    except DatabaseConnectionError as e:
        st.error(f"Database Connection Failed: {e.message}")
        st.info("Please check your database configuration and ensure the server is accessible")
        return
    except Exception as e:
        st.error(f"Failed to initialize database: {e}")
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
