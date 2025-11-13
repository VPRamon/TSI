"""Theming and styling utilities."""

import streamlit as st

from tsi.config import ASSETS_DIR


def load_custom_css() -> None:
    """Load custom CSS styles from assets/styles.css."""
    css_file = ASSETS_DIR / "styles.css"

    if css_file.exists():
        with open(css_file) as f:
            st.markdown(f"<style>{f.read()}</style>", unsafe_allow_html=True)
    else:
        # Inline fallback CSS - Dark Mode
        st.markdown(
            """
            <style>
            /* Dataset title - shown above navigation */
            .dataset-title {
                font-size: 1.8rem;
                font-weight: 700;
                text-align: center;
                color: #4da6ff;
                padding: 1.2rem 0 1rem 0;
                margin: 0;
                background-color: #0e1117;
                letter-spacing: 1.5px;
                border-bottom: none;
                text-shadow: 1px 1px 2px rgba(0,0,0,0.3);
                line-height: 1;
            }

            /* Sticky navigation bar */
            .main-nav {
                position: sticky;
                top: 0;
                background-color: #0e1117;
                z-index: 999;
                padding: 1rem 0;
                border-bottom: 2px solid #262730;
                margin-bottom: 2rem;
            }

            /* Tighter padding - reduce side margins */
            .stMainBlockContainer.block-container,
            .main .block-container,
            section.main > div.block-container,
            [data-testid="stAppViewContainer"] .main .block-container,
            [data-testid="stMainBlockContainer"] {
                padding-left: 3rem !important;
                padding-right: 3rem !important;
                padding-top: 0.5rem !important;
                max-width: 100% !important;
            }

            /* Even tighter on wide screens for better space usage */
            @media (min-width: 1024px) {
                .stMainBlockContainer.block-container,
                .main .block-container,
                section.main > div.block-container,
                [data-testid="stAppViewContainer"] .main .block-container,
                [data-testid="stMainBlockContainer"] {
                    padding-left: 4rem !important;
                    padding-right: 4rem !important;
                    max-width: 100% !important;
                }
            }

            @media (min-width: 1400px) {
                .stMainBlockContainer.block-container,
                .main .block-container,
                section.main > div.block-container,
                [data-testid="stAppViewContainer"] .main .block-container,
                [data-testid="stMainBlockContainer"] {
                    padding-left: 5rem !important;
                    padding-right: 5rem !important;
                    max-width: 1600px !important;
                }
            }

            /* Force reduce padding in all Streamlit containers */
            div[data-testid="stVerticalBlock"] > div {
                padding-left: 0 !important;
                padding-right: 0 !important;
            }

            /* Landing page title */
            .landing-title {
                font-size: 4.5rem;
                font-weight: 700;
                text-align: center;
                color: #4da6ff;
                margin: 2rem 0 2rem 0;
                text-shadow: 2px 2px 4px rgba(0,0,0,0.5);
                line-height: 1.2;
            }

            /* Metric cards */
            .metric-card {
                background: #262730;
                padding: 1rem;
                border-radius: 8px;
                text-align: center;
                border: 1px solid #3d3d4d;
            }

            .metric-value {
                font-size: 2rem;
                font-weight: 700;
                color: #4da6ff;
            }

            .metric-label {
                font-size: 0.9rem;
                color: #b3b3b3;
                text-transform: uppercase;
            }

            /* Navigation pills */
            .nav-pills {
                display: flex;
                gap: 1rem;
                justify-content: center;
                flex-wrap: wrap;
            }

            .nav-pill {
                padding: 0.5rem 1.5rem;
                border-radius: 20px;
                background: #262730;
                cursor: pointer;
                transition: all 0.2s;
                border: 1px solid #3d3d4d;
            }

            .nav-pill:hover {
                background: #3d3d4d;
            }

            .nav-pill.active {
                background: #4da6ff;
                color: #0e1117;
                border-color: #4da6ff;
            }

            /* Responsive tables */
            table {
                width: 100%;
                border-collapse: collapse;
            }

            th, td {
                padding: 0.75rem;
                text-align: left;
                border-bottom: 1px solid #3d3d4d;
            }

            th {
                background-color: #262730;
                font-weight: 600;
                color: #fafafa;
            }

            /* Hide Streamlit branding */
            #MainMenu {visibility: hidden;}
            footer {visibility: hidden;}
            header {visibility: hidden;}

            /* Reduce top padding */
            .main .block-container {
                padding-top: 1rem;
            }
            </style>
            """,
            unsafe_allow_html=True,
        )


def apply_page_config() -> None:
    """Configure Streamlit page settings."""
    st.set_page_config(
        page_title="Telescope Scheduling Intelligence",
        page_icon="🔭",
        layout="wide",
        initial_sidebar_state="collapsed",
    )


def render_metric_card(label: str, value: str | int | float) -> None:
    """
    Render a styled metric card.

    Args:
        label: Metric label
        value: Metric value
    """
    st.markdown(
        f"""
        <div class="metric-card">
            <div class="metric-value">{value}</div>
            <div class="metric-label">{label}</div>
        </div>
        """,
        unsafe_allow_html=True,
    )


def render_landing_title(title: str) -> None:
    """
    Render the landing page title.

    Args:
        title: Title text
    """
    st.markdown(
        f'<h1 class="landing-title">🔭 {title}</h1>',
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
