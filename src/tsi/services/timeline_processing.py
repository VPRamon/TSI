"""Scheduled timeline data processing services."""

from __future__ import annotations

import pandas as pd


def prepare_scheduled_data(df: pd.DataFrame) -> pd.DataFrame | None:
    """
    Prepare and validate scheduled data.

    Args:
        df: Source DataFrame

    Returns:
        Prepared DataFrame with scheduled observations, or None if no valid data
    """
    # Check if there are any scheduled observations
    if "scheduled_flag" not in df.columns or not df["scheduled_flag"].any():
        return None

    # Filter only scheduled observations with valid datetime fields
    scheduled_df = df[
        (df["scheduled_flag"])
        & (df["scheduled_start_dt"].notna())
        & (df["scheduled_stop_dt"].notna())
    ].copy()

    if len(scheduled_df) == 0:
        return None

    # Add auxiliary columns for monthly grouping
    scheduled_df["scheduled_month"] = (
        scheduled_df["scheduled_start_dt"].dt.tz_localize(None).dt.to_period("M")
    )
    scheduled_df["scheduled_month_label"] = scheduled_df["scheduled_start_dt"].dt.strftime("%Y-%m")
    scheduled_df["duration_hours"] = (
        scheduled_df["scheduled_stop_dt"] - scheduled_df["scheduled_start_dt"]
    ).dt.total_seconds() / 3600.0

    return scheduled_df


def filter_scheduled_data(
    df: pd.DataFrame,
    priority_range: tuple[float, float],
    selected_months: list[str] | None = None,
    show_short_blocks: bool = False,
    duration_threshold: float = 0.0,
) -> pd.DataFrame:
    """
    Apply filters to scheduled data.

    Args:
        df: Source DataFrame
        priority_range: Tuple of (min_priority, max_priority)
        selected_months: List of month labels to include
        show_short_blocks: Whether to filter by duration
        duration_threshold: Minimum duration threshold in hours

    Returns:
        Filtered DataFrame
    """
    filtered_df = df[
        (df["priority"] >= priority_range[0])
        & (df["priority"] <= priority_range[1])
    ]

    if selected_months:
        filtered_df = filtered_df[filtered_df["scheduled_month_label"].isin(selected_months)]

    if show_short_blocks and duration_threshold > 0:
        filtered_df = filtered_df[filtered_df["duration_hours"] >= duration_threshold]

    return filtered_df


def filter_dark_periods(
    dark_periods_df: pd.DataFrame | None,
    selected_months: list[str] | None = None,
) -> pd.DataFrame | None:
    """
    Filter dark periods by selected months.

    Args:
        dark_periods_df: DataFrame containing dark period information
        selected_months: List of month labels to include

    Returns:
        Filtered dark periods DataFrame or None
    """
    if dark_periods_df is None or dark_periods_df.empty:
        return None

    filtered_dark = dark_periods_df.copy()
    if selected_months:
        filtered_dark = filtered_dark[
            filtered_dark["months"].apply(
                lambda month_list: any(month in selected_months for month in month_list)
            )
        ]

    return filtered_dark if not filtered_dark.empty else None


def prepare_display_dataframe(filtered_df: pd.DataFrame) -> pd.DataFrame:
    """
    Prepare DataFrame for display with formatted columns.

    Args:
        filtered_df: Filtered DataFrame

    Returns:
        DataFrame formatted for display
    """
    display_columns = [
        "schedulingBlockId",
        "scheduled_month_label",
        "priority",
        "duration_hours",
        "scheduled_start_dt",
        "scheduled_stop_dt",
    ]

    # Add optional columns if they exist
    optional_columns = [
        "raInDeg",
        "decInDeg",
        "requested_hours",
        "total_visibility_hours",
        "num_visibility_periods",
    ]

    for col in optional_columns:
        if col in filtered_df.columns:
            display_columns.append(col)

    display_df = filtered_df[display_columns].copy()

    # Add day information
    display_df["start_day"] = display_df["scheduled_start_dt"].dt.day
    display_df["end_day"] = display_df["scheduled_stop_dt"].dt.day
    display_df["start_time"] = display_df["scheduled_start_dt"].dt.strftime("%H:%M")
    display_df["end_time"] = display_df["scheduled_stop_dt"].dt.strftime("%H:%M")

    # Rename columns for better display
    column_renames = {
        "schedulingBlockId": "Block ID",
        "scheduled_month_label": "Month",
        "priority": "Priority",
        "duration_hours": "Duration (h)",
        "scheduled_start_dt": "Start Date",
        "scheduled_stop_dt": "End Date",
        "start_day": "Day",
        "end_day": "End Day",
        "start_time": "Start Time",
        "end_time": "End Time",
        "raInDeg": "RA (°)",
        "decInDeg": "Dec (°)",
        "requested_hours": "Requested (h)",
        "total_visibility_hours": "Total Visibility (h)",
        "num_visibility_periods": "# Vis. Periods",
    }

    display_df = display_df.rename(columns=column_renames)

    # Reorder columns
    base_columns = [
        "Block ID",
        "Month",
        "Day",
        "Start Time",
        "End Time",
        "Priority",
        "Duration (h)",
    ]

    # Add optional columns that exist
    extra_columns = []
    for original, renamed in column_renames.items():
        if renamed not in base_columns and renamed in display_df.columns:
            extra_columns.append(renamed)

    display_df = display_df[base_columns + extra_columns]

    # Sort by month and day
    display_df = display_df.sort_values(["Month", "Day"])

    return display_df


def apply_search_filters(
    display_df: pd.DataFrame,
    search_id: str | None,
    search_month: str | None,
    min_priority: float,
) -> pd.DataFrame:
    """
    Apply search filters to display DataFrame.

    Args:
        display_df: Display DataFrame
        search_id: Block ID search string
        search_month: Month filter
        min_priority: Minimum priority filter

    Returns:
        Filtered DataFrame
    """
    filtered = display_df.copy()

    if search_id:
        filtered = filtered[
            filtered["Block ID"].astype(str).str.contains(search_id, case=False, na=False)
        ]

    if search_month and search_month != "All":
        filtered = filtered[filtered["Month"] == search_month]

    if min_priority > 0:
        filtered = filtered[filtered["Priority"] >= min_priority]

    return filtered
