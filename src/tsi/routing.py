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
                # Update current page immediately when clicked
                state.set_current_page(page)
                # Force rerun to update the UI immediately
                st.rerun()

    st.markdown("</div>", unsafe_allow_html=True)

    # Return the current page from state (which was just updated if a button was clicked)
    current = state.get_current_page()
    return str(current) if current else None


def route_to_page(page_name: str) -> None:
    """
    Route to the specified page and render it.

    Args:
        page_name: Name of the page to render
    """
    # Import page modules dynamically to avoid circular imports
    from tsi.pages import (
        distributions,
        insights,
        scheduled_timeline,
        scheduling_trends,
        sky_map,
        visibility_schedule,
    )

    page_map = {
        "Sky Map": sky_map.render,
        "Distributions": distributions.render,
        "Visibility Map": visibility_schedule.render,
        "Schedule": scheduled_timeline.render,
        "Insights": insights.render,
        "Trends": scheduling_trends.render,
    }

    render_func = page_map.get(page_name)

    if render_func:
        render_func()
    else:
        st.error(f"Unknown page: {page_name}")
