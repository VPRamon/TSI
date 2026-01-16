"""Navigation and routing logic."""

import streamlit as st

from tsi import state
from tsi.config import PAGES
from tsi.theme import render_dataset_title


def render_navigation() -> str | None:
    """
    Render sticky navigation bar and return selected page.

    Returns:
        Selected page name or None if no data loaded
    """
    if not state.has_data():
        return None

    # Render dataset title if available
    filename = state.get_data_filename()
    if filename:
        render_dataset_title(filename)

    # Navigation bar with sticky positioning
    st.markdown('<div class="main-nav">', unsafe_allow_html=True)

    # Create columns for navigation items
    cols = st.columns(len(PAGES))

    current_page = state.get_current_page()

    # Process navigation clicks
    for idx, (col, page) in enumerate(zip(cols, PAGES)):
        with col:
            # Use button for navigation
            if st.button(
                page,
                key=f"nav_{page}",
                width="stretch",
                type="primary" if page == current_page else "secondary",
            ):
                # Update current page and force rerun to update button highlight
                state.set_current_page(page)
                st.rerun()

    st.markdown("</div>", unsafe_allow_html=True)

    # Return the current page from state, defaulting to first page if not set
    current = state.get_current_page()
    if current is None and PAGES:
        current = PAGES[0]
        state.set_current_page(current)
    return str(current) if current else None


def route_to_page(page_name: str) -> None:
    """
    Route to the specified page and render it.

    Args:
        page_name: Name of the page to render
    """
    # Lazy import - only load the needed page module for better performance
    page_map = {
        "Validation": ("tsi.pages.validation_report", "render"),
        "Sky Map": ("tsi.pages.sky_map", "render"),
        "Distributions": ("tsi.pages.distributions", "render"),
        "Visibility Map": ("tsi.pages.visibility_map", "render"),
        "Schedule": ("tsi.pages.scheduled_timeline", "render"),
        "Insights": ("tsi.pages.insights", "render"),
        "Trends": ("tsi.pages.scheduling_trends", "render"),
        "Compare": ("tsi.pages.compare_schedules", "render"),
        "Creative": ("tsi.pages.creative_workspace", "render"),
    }

    page_info = page_map.get(page_name)

    if page_info:
        module_name, func_name = page_info
        # Import only the specific module needed
        import importlib

        module = importlib.import_module(module_name)
        render_func = getattr(module, func_name)
        render_func()
    else:
        st.error(f"Unknown page: {page_name}")
