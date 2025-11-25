"""Scheduling trends page empirical analysis components."""

from __future__ import annotations

from typing import TYPE_CHECKING, cast

import streamlit as st

if TYPE_CHECKING:
    from altair import Chart

from tsi.modeling.trends import EmpiricalRates
from tsi.plots.trends import bar_rate_by_priority


def render_empirical_proportions(
    empirical: EmpiricalRates,
    plot_library: str,
) -> None:
    """
    Display empirical proportions by priority section.
    
    Args:
        empirical: Computed empirical rates
        plot_library: Plotting library to use ('altair' or 'plotly')
    """
    st.subheader("1️⃣ Empirical proportions by priority")
    st.caption(
        "📊 Shows the **empirical rate** of scheduling for each priority level. "
        "The tooltip includes the number of observations (n)."
    )
    
    try:
        fig_priority = bar_rate_by_priority(
            empirical.by_priority,
            library=plot_library,
            title="Scheduling rate by priority",
        )
        
        if plot_library == "altair":
            st.altair_chart(cast("Chart", fig_priority), use_container_width=True)
        else:
            st.plotly_chart(fig_priority, use_container_width=True)
    
    except Exception as e:
        st.error(f"❌ Error computing empirical rates: {e}")
