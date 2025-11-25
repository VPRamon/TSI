"""Scheduling trends page smoothed trends components."""

from __future__ import annotations

from typing import TYPE_CHECKING, cast

import streamlit as st

if TYPE_CHECKING:
    from altair import Chart

from tsi.plots.trends import loess_trend


def render_smoothed_trends(
    smooth_vis: tuple | None,
    error_vis: str | None,
    smooth_time: tuple | None,
    error_time: str | None,
    plot_library: str,
) -> None:
    """
    Display smoothed trends section with visibility and requested time curves.
    
    Args:
        smooth_vis: Smoothed visibility trend data
        error_vis: Error message for visibility trend
        smooth_time: Smoothed requested time trend data
        error_time: Error message for requested time trend
        plot_library: Plotting library to use ('altair' or 'plotly')
    """
    st.subheader("2️⃣ Smoothed curves (trends)")
    st.caption(
        "📈 Smoothed trends using weighted moving average (similar to LOESS). "
        "Shows how the scheduling rate varies with visibility and requested time."
    )

    col1, col2 = st.columns(2)

    with col1:
        st.markdown("**Visibility → Scheduling rate**")

        if error_vis:
            st.warning(f"⚠️ {error_vis}")
        elif smooth_vis is not None:
            fig_vis = loess_trend(
                smooth_vis,
                library=plot_library,
                title="Trend: Visibility",
                x_label="Visibility (hours)",
                y_label="Scheduling rate",
            )

            if plot_library == "altair":
                st.altair_chart(cast("Chart", fig_vis), use_container_width=True)
            else:
                st.plotly_chart(fig_vis, use_container_width=True)

    with col2:
        st.markdown("**Requested time → Scheduling rate**")

        if error_time:
            st.warning(f"⚠️ {error_time}")
        elif smooth_time is not None:
            fig_time = loess_trend(
                smooth_time,
                library=plot_library,
                title="Trend: Requested time",
                x_label="Requested time (hours)",
                y_label="Scheduling rate",
            )

            if plot_library == "altair":
                st.altair_chart(cast("Chart", fig_time), use_container_width=True)
            else:
                st.plotly_chart(fig_time, use_container_width=True)
