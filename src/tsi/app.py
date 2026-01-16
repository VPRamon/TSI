"""Main application entry point for Telescope Scheduling Intelligence.

Run with: streamlit run src/tsi/app.py
"""

from tsi import state
from tsi.pages import landing
from tsi.pages import creative_workspace
from tsi.routing import render_navigation, route_to_page
from tsi.theme import apply_page_config, load_custom_css


def main() -> None:
    """Main application entry point."""
    # Configure page
    apply_page_config()

    # Load custom CSS
    load_custom_css()

    # Initialize session state
    state.initialize_state()

    # Check if creative mode is active
    if creative_workspace.is_creative_mode():
        # Show creative workspace
        creative_workspace.render()
    elif not state.has_data():
        # Show landing page
        landing.render()
    else:
        # Render navigation and get selected page
        selected_page = render_navigation()

        if selected_page:
            # Route to selected page
            route_to_page(selected_page)
        else:
            # Fallback to landing if something goes wrong
            landing.render()


if __name__ == "__main__":
    main()
