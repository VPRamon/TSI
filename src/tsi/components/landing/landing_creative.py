"""Landing page creative section component."""

from __future__ import annotations

import streamlit as st


def render_creative_section() -> None:
    """Render the creative workspace section on the landing page."""
    st.markdown("### 🎨 Creative Mode")
    st.markdown("Build proposals and run scheduling simulations")
    
    st.markdown(
        """
        <div style='
            padding: 1rem;
            border-radius: 8px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            margin: 1rem 0;
        '>
            <p style='margin: 0; font-size: 0.95em;'>
                ✨ Create observation proposals interactively<br>
                🌳 Visualize proposal structure as a tree<br>
                ⚙️ Configure scheduling algorithms<br>
                🚀 Run simulations and analyze results
            </p>
        </div>
        """,
        unsafe_allow_html=True,
    )
    
    if st.button(
        "🎨 Enter Creative Mode",
        type="primary",
        key="enter_creative_btn",
        width="stretch",
    ):
        _enter_creative_mode()


def _enter_creative_mode() -> None:
    """Navigate to the creative workspace."""
    # Set creative mode flag
    st.session_state["creative_mode"] = True
    st.rerun()
