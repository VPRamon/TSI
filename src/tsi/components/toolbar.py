"""Toolbar components for filters and controls."""

import streamlit as st


def render_priority_filter(
    key: str = "priority_range",
    min_value: float = 0.0,
    max_value: float = 10.0,
    default: tuple[float, float] | None = None,
) -> tuple[float, float]:
    """
    Render priority range slider.

    Args:
        key: Session state key
        min_value: Minimum allowed priority
        max_value: Maximum allowed priority
        default: Default slider values (min, max)

    Returns:
        Tuple of (min_priority, max_priority)
    """
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
        "Priority Range",
        min_value=slider_min,
        max_value=slider_max,
        value=default,
        step=0.1,
        key=key,
        help="Filter observations by priority range",
    )
    return priority_range


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
