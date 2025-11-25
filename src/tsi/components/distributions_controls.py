"""Distribution page control components for filtering."""

from __future__ import annotations

import pandas as pd

from tsi import state
from tsi.components.toolbar import render_impossible_filter_control


def render_filter_control(df: pd.DataFrame) -> tuple[str, bool]:
    """
    Render filter control for impossible observations.

    Args:
        df: The prepared DataFrame

    Returns:
        Tuple of (filter_mode, filter_supported)
    """
    from tsi.services.impossible_filters import check_filter_support
    
    filter_mode = render_impossible_filter_control(
        df,
        key=state.KEY_DIST_FILTER_MODE,
    )
    
    filter_supported = check_filter_support(df)
    
    return filter_mode, filter_supported

