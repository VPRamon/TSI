"""Sky Map statistics and metrics components."""

from typing import Any

import streamlit as st


def render_stats(blocks: list[Any]) -> None:
    """
    Render summary metrics for the filtered dataset.

    Args:
        blocks: List of SchedulingBlock PyO3 objects to display statistics for
    """
    st.markdown("#### Subset summary")

    col1, col2, col3 = st.columns(3)

    with col1:
        st.metric("Observaciones mostradas", f"{len(blocks):,}")

    with col2:
        ra_values = [float(block.target_ra_deg) for block in blocks]
        if ra_values:
            ra_range = max(ra_values) - min(ra_values)
            st.metric("RA coverage", f"{ra_range:.1f}°")
        else:
            st.metric("RA coverage", "N/A")

    with col3:
        dec_values = [float(block.target_dec_deg) for block in blocks]
        if dec_values:
            dec_range = max(dec_values) - min(dec_values)
            st.metric("Dec coverage", f"{dec_range:.1f}°")
        else:
            st.metric("Dec coverage", "N/A")

    # Calculate scheduled share
    scheduled_count = sum(
        1 for block in blocks if getattr(block, "scheduled_period", None) is not None
    )
    if blocks:
        scheduled_share = (scheduled_count / len(blocks)) * 100
        st.caption(f"{scheduled_share:.1f}% of the filtered targets are scheduled.")
    else:
        st.caption("No targets to display.")
