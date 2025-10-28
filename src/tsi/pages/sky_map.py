"""Sky Map page - celestial coordinate visualization with advanced filtering."""

from __future__ import annotations

from collections.abc import Sequence
from datetime import datetime, timedelta
from typing import cast

import pandas as pd
import streamlit as st

from tsi import state
from tsi.components.toolbar import (
    render_reset_filters_button,
    render_toggle,
)
from tsi.plots.sky_map import build_figure


def render() -> None:
    """Render the Sky Map page."""
    st.title("🌌 Sky Map")

    st.markdown(
        """
        Visualize the distribution of targets in celestial coordinates and apply advanced filters
        to understand how they vary by priority and scheduling status.
        """
    )

    df = state.get_prepared_data()

    if df is None:
        st.warning("No dataset loaded. Please return to the main screen.")
        return

    if "priority" not in df.columns:
        st.error("The dataset is missing the `priority` column required for this analysis.")
        return

    if "priority_bin" not in df.columns:
        st.error("The dataset is missing the `priority_bin` column required for the original bins.")
        return

    priority_values = df["priority"].dropna()
    if priority_values.empty:
        st.warning("No priority values available to build the map.")
        return

    priority_min = float(priority_values.min())
    priority_max = float(priority_values.max())
    if priority_min == priority_max:
        priority_max = priority_min + 1.0

    # Convert priority_bin to string, handling NaN values
    # Use a view instead of copy for better performance
    if df["priority_bin"].dtype != "string":
        df = df.copy()  # Only copy when we need to modify
        df["priority_bin"] = df["priority_bin"].astype("string").fillna("No priority")

    priority_bins = df["priority_bin"].dropna().unique().tolist()
    if not priority_bins:
        st.warning("No original priority bins available in the dataset.")
        return

    # Panel lateral izquierdo y sky map derecho
    sidebar_col, map_col = st.columns([1, 3], gap="large")

    with sidebar_col:
        st.markdown("### 🎛️ Settings")
        with st.container(border=True):
            # Selectores
            scheduled_filter = st.radio(
                "Scheduling Status",
                options=["All", "Scheduled", "Unscheduled"],
                key="sky_scheduled_filter",
            )
            state.set_scheduled_filter(scheduled_filter)

            color_options = {
                "Priority": "priority_bin",
                "Status": "scheduled_flag",
            }

            color_choice = st.radio(
                "Color by",
                options=list(color_options.keys()),
                index=0,
                key="sky_color_choice",
            )
            color_column = color_options[color_choice]

            st.markdown("---")

            # Sliders
            stored_range = state.get_priority_range()
            # If stored range is None, outside actual data range, or is the generic default,
            # use the full data range
            if (
                stored_range is None
                or stored_range[0] < priority_min
                or stored_range[1] > priority_max
                or stored_range == (0.0, 10.0)
            ):
                default_range = (priority_min, priority_max)
            else:
                default_range = (
                    max(priority_min, stored_range[0]),
                    min(priority_max, stored_range[1]),
                )

            priority_range = st.slider(
                "Priority Range",
                min_value=priority_min,
                max_value=priority_max,
                value=default_range,
                step=0.1,
                key="sky_priority_range",
            )
            state.set_priority_range(priority_range)

            # Use all priority bins (no bin filtering)
            selected_bins = priority_bins

            schedule_window = _render_schedule_window_control(df)
            state.set_schedule_window(schedule_window)

            st.markdown("---")

            # Botones
            flip_ra = render_toggle(
                "Invertir eje RA",
                default=True,
                key="sky_flip_ra",
            )

            if render_reset_filters_button():
                state.reset_filters()
                _reset_sky_map_controls()
                # Streamlit will auto-rerun on button click

    with map_col:
        filtered_df = _filter_dataframe(
            df,
            priority_range=priority_range,
            scheduled_filter=scheduled_filter,
            selected_bins=selected_bins,
            schedule_window=schedule_window,
        )

        if filtered_df.empty:
            st.warning("No targets match the selected filters.")
            return

        category_palette = None
        if color_column == "priority_bin":
            category_palette = _build_palette(priority_bins)

        fig = build_figure(
            df=filtered_df,
            color_by=color_column,
            size_by="requested_hours",
            flip_ra=flip_ra,
            category_palette=category_palette,
        )

        st.plotly_chart(fig, use_container_width=True, key="sky_map_chart")

        st.markdown("---")
        _render_stats(filtered_df)


def _render_schedule_window_control(df: pd.DataFrame) -> tuple[datetime, datetime] | None:
    """Render slider to filter by scheduled start datetime."""
    if "scheduled_flag" not in df.columns or "scheduled_start_dt" not in df.columns:
        st.caption("The dataset does not include scheduling dates.")
        return None

    scheduled = df[df["scheduled_flag"]]["scheduled_start_dt"].dropna()
    if scheduled.empty:
        st.caption("No observations include a scheduled date.")
        return None

    min_dt = scheduled.min()
    max_dt = scheduled.max()

    min_dt_dt = _ts_to_datetime(min_dt)
    max_dt_dt = _ts_to_datetime(max_dt)

    if min_dt_dt == max_dt_dt:
        max_dt_dt = max_dt_dt + timedelta(hours=1)

    stored_window = state.get_schedule_window()
    if stored_window:
        default_window = (
            max(min_dt_dt, stored_window[0]),
            min(max_dt_dt, stored_window[1]),
        )
    else:
        default_window = (min_dt_dt, max_dt_dt)

    return st.slider(
        "Scheduled window (UTC)",
        min_value=min_dt_dt,
        max_value=max_dt_dt,
        value=default_window,
        format="YYYY-MM-DD HH:mm",
        key="sky_schedule_window",
        help="Show only scheduled targets whose start time falls within this window.",
    )


def _filter_dataframe(
    df: pd.DataFrame,
    priority_range: tuple[float, float],
    scheduled_filter: str,
    selected_bins: Sequence[str],
    schedule_window: tuple[datetime, datetime] | None,
) -> pd.DataFrame:
    """Apply priority, bin, scheduled status and time filters."""
    # Use boolean indexing without copying until the final result
    mask = (df["priority"] >= priority_range[0]) & (df["priority"] <= priority_range[1])

    if selected_bins:
        mask &= df["priority_bin"].isin(selected_bins)

    if scheduled_filter == "Scheduled":
        mask &= df["scheduled_flag"]
    elif scheduled_filter == "Unscheduled":
        mask &= ~df["scheduled_flag"]

    # Apply time window filter if specified
    if schedule_window:
        start_ts = _to_utc_timestamp(schedule_window[0])
        end_ts = _to_utc_timestamp(schedule_window[1])

        scheduled_mask = (
            df["scheduled_flag"]
            & df["scheduled_start_dt"].notna()
            & (df["scheduled_start_dt"] >= start_ts)
            & (df["scheduled_start_dt"] <= end_ts)
        )

        if scheduled_filter == "All":
            # Include all unscheduled + scheduled within window
            unscheduled_mask = ~df["scheduled_flag"]
            mask &= (unscheduled_mask | scheduled_mask)
        elif scheduled_filter == "Scheduled":
            mask &= scheduled_mask
        # For unscheduled, window doesn't apply - already filtered above

    # Only create copy at the end when returning filtered result
    return df[mask].copy()


def _render_stats(df: pd.DataFrame) -> None:
    """Render summary metrics for the filtered dataset."""
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


def _ts_to_datetime(ts: pd.Timestamp) -> datetime:
    """Convert pandas Timestamp to naive datetime preserving UTC."""
    # Floor to seconds to avoid nanoseconds warning
    ts = ts.floor("s")
    if ts.tzinfo is not None:
        return cast(datetime, ts.tz_convert(None).to_pydatetime())
    return cast(datetime, ts.to_pydatetime())


def _to_utc_timestamp(value: datetime) -> pd.Timestamp:
    """Convert naive or aware datetime to UTC pandas Timestamp."""
    ts = pd.Timestamp(value)
    if ts.tzinfo is None:
        ts = ts.tz_localize("UTC")
    else:
        ts = ts.tz_convert("UTC")
    return ts


def _build_palette(labels: Sequence[str]) -> dict:
    """Generate a simple color palette for categorical bins."""
    base_colors = [
        "#1f77b4",
        "#ff7f0e",
        "#2ca02c",
        "#d62728",
        "#9467bd",
        "#8c564b",
        "#e377c2",
        "#7f7f7f",
        "#bcbd22",
        "#17becf",
    ]
    palette = {}
    for idx, label in enumerate(labels):
        palette[label] = base_colors[idx % len(base_colors)]
    return palette


def _reset_sky_map_controls() -> None:
    """Clear widget-level state so defaults apply after reset."""
    # Delete all widget keys that start with 'sky_' to reset all sky map widgets
    # This includes: sky_priority_range, sky_scheduled_filter, sky_schedule_window,
    # sky_flip_ra, sky_color_choice, and all sky_selected_bins_* checkbox keys
    keys_to_delete = [
        key for key in st.session_state.keys() if isinstance(key, str) and key.startswith("sky_")
    ]

    for key in keys_to_delete:
        del st.session_state[key]
