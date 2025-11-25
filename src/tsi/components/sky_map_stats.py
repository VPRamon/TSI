"""Sky Map statistics and metrics components."""

import pandas as pd
import streamlit as st


def render_stats(df: pd.DataFrame) -> None:
    """
    Render summary metrics for the filtered dataset.

    Args:
        df: Filtered DataFrame to display statistics for
    """
    st.markdown("#### Subset summary")

    col1, col2, col3 = st.columns(3)

    with col1:
        st.metric("Observaciones mostradas", f"{len(df):,}")

    with col2:
        ra_range = df["raInDeg"].max() - df["raInDeg"].min()
        st.metric("RA coverage", f"{ra_range:.1f}°")

    with col3:
        dec_range = df["decInDeg"].max() - df["decInDeg"].min()
        st.metric("Dec coverage", f"{dec_range:.1f}°")

    scheduled_share = df["scheduled_flag"].mean() * 100
    st.caption(f"{scheduled_share:.1f}% of the filtered targets are scheduled.")
