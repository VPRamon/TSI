"""Theming and styling utilities."""

import streamlit as st

from tsi.config import ASSETS_DIR, APP_TITLE, APP_ICON


def load_custom_css() -> None:
    """Load custom CSS styles from assets/styles.css."""
    with open(ASSETS_DIR / "styles.css") as f:
        st.markdown(f"<style>{f.read()}</style>", unsafe_allow_html=True)


def apply_page_config() -> None:
    """Configure Streamlit page settings using centralized configuration."""
    from app_config import get_settings
    
    settings = get_settings()
    
    st.set_page_config(
        page_title=settings.app_title,
        page_icon=settings.app_icon,
        layout=settings.layout,
        initial_sidebar_state=settings.initial_sidebar_state,
    )


def render_landing_title(title: str | None = None) -> None:
    """
    Render the landing page title.

    Args:
        title: Title text (defaults to APP_TITLE from config)
    """
    if title is None:
        title = APP_TITLE
    
    st.markdown(
        f'<h1 class="landing-title">{APP_ICON} {title}</h1>',
        unsafe_allow_html=True,
    )


def add_vertical_space(lines: int = 1) -> None:
    """
    Add vertical spacing.

    Args:
        lines: Number of blank lines to add
    """
    for _ in range(lines):
        st.markdown("<br>", unsafe_allow_html=True)


def render_dataset_title(filename: str) -> None:
    """
    Render the dataset title above the navigation.

    Args:
        filename: Name of the loaded dataset file
    """
    import re

    # Clean filename: remove extension, convert to uppercase, remove special characters
    clean_name = filename
    # Remove common extensions
    clean_name = re.sub(r"\.(csv|json)$", "", clean_name, flags=re.IGNORECASE)
    # Replace underscores and dashes with spaces
    clean_name = clean_name.replace("_", " ").replace("-", " ")
    # Remove special characters, keep only alphanumeric and spaces
    clean_name = re.sub(r"[^A-Za-z0-9\s]", "", clean_name)
    # Convert to uppercase
    clean_name = clean_name.upper().strip()

    st.markdown(
        f'<div class="dataset-title">{clean_name}</div>',
        unsafe_allow_html=True,
    )
