"""Toolbar components for filters and controls."""

from typing import cast

import pandas as pd
import streamlit as st

from tsi.services.filters.impossible import check_filter_support


def render_priority_filter(
    key: str = "priority_range",
    min_value: float = 0.0,
    max_value: float = 10.0,
    default: tuple[float, float] | None = None,
    label: str = "Priority Range",
    subheader: str | None = None,
) -> tuple[float, float]:
    """
    Render priority range slider.

    Args:
        key: Session state key
        min_value: Minimum allowed priority
        max_value: Maximum allowed priority
        default: Default slider values (min, max)
        label: Label text for the slider control
        subheader: Optional subheader text to render above the control

    Returns:
        Tuple of (min_priority, max_priority)
    """
    if subheader:
        st.subheader(subheader)

    slider_min = float(min_value)
    slider_max = float(max_value)

    if slider_max <= slider_min:
        slider_max = slider_min + 1.0

    if default is None:
        default = (slider_min, slider_max)

    low = max(slider_min, min(default[0], slider_max))
    high = max(slider_min, min(default[1], slider_max))
    if low > high:
        low, high = high, low
    default = (low, high)

    priority_range = st.slider(
        label,
        min_value=slider_min,
        max_value=slider_max,
        value=default,
        step=0.1,
        key=key,
        help="Filter observations by priority range",
    )
    return priority_range


def render_priority_range_control(
    min_value: float,
    max_value: float,
    stored_range: tuple[float, float] | None,
    key: str,
) -> tuple[float, float]:
    """
    Render priority range slider with state handling.

    Handles default range logic consistently across pages.

    Args:
        min_value: Minimum priority in dataset
        max_value: Maximum priority in dataset
        stored_range: Previously stored range from state
        key: Session state key

    Returns:
        Selected priority range tuple
    """
    if (
        stored_range is None
        or stored_range[0] < min_value
        or stored_range[1] > max_value
        or stored_range == (0.0, 10.0)
    ):
        default_range = (min_value, max_value)
    else:
        default_range = (
            max(min_value, stored_range[0]),
            min(max_value, stored_range[1]),
        )

    result = st.slider(
        "Priority Range",
        min_value=min_value,
        max_value=max_value,
        value=default_range,
        step=0.1,
        key=key,
        help="Filter observations by priority range",
    )
    return cast(tuple[float, float], result)


def render_impossible_filter_control(
    df: pd.DataFrame,
    key: str,
    label_visible: bool = False,
) -> str:
    """
    Render impossible observation filter control.

    Provides radio buttons to filter out observations where required duration
    exceeds total visibility hours.

    Args:
        df: Source DataFrame
        key: Session state key
        label_visible: Whether to show the label (default: collapsed)

    Returns:
        Selected filter mode ('all' or 'exclude_impossible')
    """
    if key not in st.session_state:
        st.session_state[key] = "all"

    filter_supported = check_filter_support(df)

    if not filter_supported:
        st.session_state[key] = "all"
        return "all"

    filter_options = ("all", "exclude_impossible")
    filter_labels = {
        "all": "📋 All blocks",
        "exclude_impossible": "✅ Filter invalid requests",
    }

    st.markdown("<div style='margin-top: 1.5rem;'></div>", unsafe_allow_html=True)

    result = st.radio(
        "Filtrar:",
        options=filter_options,
        format_func=lambda x: filter_labels[x],
        key=key,
        horizontal=False,
        label_visibility="collapsed" if not label_visible else "visible",
    )
    return cast(str, result)


def render_toggle(label: str, default: bool = True, key: str | None = None) -> bool:
    """
    Render a simple toggle checkbox.

    Args:
        label: Toggle label
        default: Default value
        key: Optional session state key

    Returns:
        Toggle value
    """
    return st.checkbox(label, value=default, key=key)


def render_number_input(
    label: str,
    min_value: int = 1,
    max_value: int = 100,
    default: int = 20,
    key: str | None = None,
) -> int:
    """
    Render number input.

    Args:
        label: Input label
        min_value: Minimum value
        max_value: Maximum value
        default: Default value
        key: Optional session state key

    Returns:
        Input value
    """
    return st.number_input(
        label,
        min_value=min_value,
        max_value=max_value,
        value=default,
        step=1,
        key=key,
    )


def render_reset_filters_button() -> bool:
    """
    Render a reset filters button.

    Returns:
        True if button was clicked
    """
    return st.button("Reset Filters", type="secondary")
