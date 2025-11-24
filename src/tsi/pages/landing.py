"""Landing page for the Telescope Scheduling Intelligence app."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import streamlit as st

from app_config import get_settings
from core.time import format_datetime_utc
from tsi import state
from tsi.components.data_preview import render_data_preview
from tsi.components.metrics import render_kpi_cards
from tsi.services import load_dark_periods
from tsi.services.loaders import load_csv, prepare_dataframe, validate_dataframe
from tsi.theme import add_vertical_space, render_landing_title


def render() -> None:
    """Render the landing page with data selection options."""

    # Try to auto-load dark periods if not already loaded
    if state.get_dark_periods() is None:
        _try_auto_load_dark_periods()

    # Add significant vertical space at the top
    add_vertical_space(4)

    # Get settings at runtime to allow for test environment variable overrides
    settings = get_settings()

    render_landing_title("Telescope Scheduling Intelligence")

    st.markdown(
        """
        <div style='text-align: center; font-size: 1.4rem; color: #888; margin-bottom: 4rem; font-weight: 300;'>
        Analyze and visualize telescope scheduling data with interactive insights
        </div>
        """,
        unsafe_allow_html=True,
    )

    add_vertical_space(3)

    # Three-column layout for data selection
    col1, col2, col3 = st.columns(3)

    with col1:
        st.markdown("### 📤 Upload CSV")
        st.markdown("Upload a preprocessed CSV file")

        uploaded_csv = st.file_uploader(
            "Choose a CSV file",
            type=["csv"],
            help="Upload a preprocessed CSV file with scheduling data",
            key="csv_uploader",
        )

        if uploaded_csv is not None:
            if st.button("Load CSV File", type="primary", width="stretch"):
                _load_data(
                    uploaded_csv, source="uploaded_csv", file_type="csv", filename=uploaded_csv.name
                )

    with col2:
        st.markdown("### 📋 Upload JSON")
        st.markdown("Upload raw schedule.json file")

        uploaded_json = st.file_uploader(
            "Choose a schedule.json file",
            type=["json"],
            help="Upload a raw schedule.json file - will be processed automatically",
            key="json_uploader",
        )

        if uploaded_json is not None:
            # Optional: also allow visibility file
            with st.expander("🔍 Add visibility data (optional)", expanded=False):
                uploaded_visibility = st.file_uploader(
                    "Choose possible_periods.json (optional)",
                    type=["json"],
                    help="Optional: upload visibility/possible periods data",
                    key="visibility_uploader",
                )

            if st.button("Load JSON File", type="primary", width="stretch"):
                visibility_file = st.session_state.get("uploaded_visibility_file", None)
                _load_data(
                    uploaded_json,
                    source="uploaded_json",
                    file_type="json",
                    visibility_file=(
                        uploaded_visibility if "uploaded_visibility" in locals() else None
                    ),
                    filename=uploaded_json.name,
                )

    with col3:
        st.markdown("### 📊 Use Sample Data")
        st.markdown("Load the pre-configured sample dataset")

        add_vertical_space(1)

        if st.button("Load Sample Dataset", type="secondary", width="stretch"):
            if not settings.sample_dataset.exists():
                st.error(
                    f"""
                    ⚠️ **Sample data file not found!**

                    Expected location: `{settings.sample_dataset}`

                    Please ensure the sample CSV file exists at this location.
                    """
                )
            else:
                _load_data(
                    str(settings.sample_dataset),
                    source="sample",
                    file_type="csv",
                    filename=settings.sample_dataset.name,
                )

    # Show data preview if data is loaded
    if state.has_data():
        add_vertical_space(2)
        st.divider()

        df = state.get_prepared_data()
        existing_dark_periods = state.get_dark_periods()

        st.success("✅ Data loaded successfully!")

        # Show KPIs
        scheduled_count = df["scheduled_flag"].sum()
        mean_priority = df["priority"].mean()
        total_visibility = df["total_visibility_hours"].sum()

        render_kpi_cards(
            total_obs=len(df),
            scheduled=int(scheduled_count),
            mean_priority=mean_priority,
            total_visibility=total_visibility,
        )

        add_vertical_space(1)

        # Preview table with key columns
        preview_columns = [
            "schedulingBlockId",
            "priority",
            "priority_bin",
            "requested_hours",
            "total_visibility_hours",
            "scheduled_flag",
            "raInDeg",
            "decInDeg",
        ]

        render_data_preview(
            df,
            max_rows=15,
            columns=preview_columns,
            title="📋 Data Preview",
        )

        st.markdown("---")
        st.subheader("🌑 Dark periods (optional)")
        st.caption(
            "Load a `dark_periods.json` file to highlight dark windows on the"
            " **Planned Schedule** page. This helps identify why there are"
            " gaps without observations."
        )

        # Auto-load dark periods from local file if available and not already loaded
        existing_dark_periods = state.get_dark_periods()
        if existing_dark_periods is None:
            _try_auto_load_dark_periods()
            existing_dark_periods = state.get_dark_periods()

        # Show auto-load message once
        if existing_dark_periods is not None and st.session_state.get("dark_periods_auto_loaded"):
            st.success(
                f"✅ Dark periods loaded automatically from data/dark_periods.json ({len(existing_dark_periods)} periods)"
            )
            # Remove the flag so message doesn't repeat
            st.session_state.pop("dark_periods_auto_loaded", None)

        dark_periods_file = st.file_uploader(
            "Select dark_periods.json",
            type=["json"],
            key="dark_periods_uploader",
            help="File exported from CTA with the dark periods of the year",
        )

        if dark_periods_file is not None:
            file_token = (
                f"{getattr(dark_periods_file, 'name', '')}:{getattr(dark_periods_file, 'size', '')}"
            )
            if st.session_state.get("dark_periods_last_token") != file_token:
                try:
                    dark_periods_df = load_dark_periods(dark_periods_file)
                except Exception as exc:  # pragma: no cover - Streamlit feedback only
                    st.error(f"❌ Could not load dark periods: {exc}")
                else:
                    if dark_periods_df.empty:
                        state.set_dark_periods(None)
                        st.warning("⚠️ The file does not contain valid dark periods.")
                    else:
                        state.set_dark_periods(dark_periods_df)
                        st.session_state["dark_periods_last_token"] = file_token
                        st.success(f"🌑 {len(dark_periods_df):,} dark periods loaded.")

        existing_dark_periods = state.get_dark_periods()

        if existing_dark_periods is not None and not existing_dark_periods.empty:
            min_dark = existing_dark_periods["start_dt"].min()
            max_dark = existing_dark_periods["stop_dt"].max()
            total_dark_hours = existing_dark_periods["duration_hours"].sum()

            st.caption(
                f"Currently there are {len(existing_dark_periods):,} dark periods loaded"
                f" (total {total_dark_hours:,.1f} h)."
                f" Time range: {format_datetime_utc(min_dark)} → {format_datetime_utc(max_dark)}."
            )

            with st.expander("View loaded dark periods", expanded=False):
                preview_cols = existing_dark_periods.copy()
                preview_cols["start"] = preview_cols["start_dt"].dt.strftime("%Y-%m-%d %H:%M")
                preview_cols["end"] = preview_cols["stop_dt"].dt.strftime("%Y-%m-%d %H:%M")
                preview_cols = preview_cols[["start", "end", "duration_hours", "months"]]
                preview_cols = preview_cols.rename(
                    columns={"duration_hours": "Duration (h)", "months": "Months"}
                )
                st.dataframe(
                    preview_cols,
                    hide_index=True,
                    width="stretch",
                    height=240,
                )

            if st.button("Remove dark periods", key="clear_dark_periods"):
                state.set_dark_periods(None)
                st.session_state.pop("dark_periods_last_token", None)
                # Streamlit will auto-rerun on button click

        # Provide navigation hint
        st.info("👆 Use the top navigation to explore visualizations and insights")


def _try_auto_load_dark_periods() -> None:
    """Try to auto-load dark periods from the local data directory if available."""
    try:
        # Build path relative to the repository root
        # This module is in src/tsi/pages/, so go up 3 levels to get to repo root
        current_file = Path(__file__).resolve()
        repo_root = current_file.parent.parent.parent.parent
        dark_periods_path = repo_root / "data" / "dark_periods.json"

        if dark_periods_path.exists():
            import traceback

            try:
                dark_periods_df = load_dark_periods(dark_periods_path)
                if not dark_periods_df.empty:
                    state.set_dark_periods(dark_periods_df)
                    # Mark that auto-load was successful
                    if "dark_periods_auto_loaded" not in st.session_state:
                        st.session_state["dark_periods_auto_loaded"] = True
            except Exception as load_error:
                # Log error for debugging
                st.error(f"Error loading dark_periods.json automatically: {load_error}")
                st.code(traceback.format_exc())
    except Exception:
        # Path check failed - silently continue
        pass


def _load_data(
    file_or_path: Any,
    source: str,
    file_type: str = "csv",
    visibility_file: Any | None = None,
    filename: str | None = None,
) -> None:
    """
    Load and prepare data from file or path.

    Args:
        file_or_path: File buffer or path string
        source: Data source identifier
        file_type: Type of file ('csv' or 'json')
        visibility_file: Optional visibility JSON file (only for JSON uploads)
        filename: Name of the file being loaded
    """
    try:
        with st.spinner("Loading and processing data..."):
            if file_type == "json":
                # Use Rust backend for loading (10x faster)
                from tsi.services.rust_compat import load_schedule_rust

                # Process JSON directly to DataFrame
                st.info("🔄 Processing schedule.json file (using Rust backend - 10x faster)...")
                raw_df = load_schedule_rust(file_or_path)

                # Convert visibility lists to strings for Streamlit caching compatibility
                # Streamlit's cache_data uses pandas hashing which doesn't support list columns
                if "visibility" in raw_df.columns:
                    raw_df["visibility"] = raw_df["visibility"].apply(str)

                # Show processing stats
                # Note: Rust backend doesn't return warnings yet, could be added in future
                # if result.validation.warnings:
                #     with st.expander("⚠️ Processing warnings", expanded=False):
                # Note: Rust backend doesn't return warnings yet, could be added in future
                # if result.validation.warnings:
                #     with st.expander("⚠️ Processing warnings", expanded=False):
                #         for warning in result.validation.warnings:
                #             st.warning(f"  - {warning}")

                st.success(f"✅ Processed {len(raw_df)} scheduling blocks from JSON (Rust backend)")

            else:  # CSV
                # Load raw CSV (uses Rust backend - 10x faster)
                st.info("🔄 Loading CSV file (using Rust backend - 10x faster)...")
                raw_df = load_csv(file_or_path)
                st.success(f"✅ Loaded {len(raw_df)} scheduling blocks from CSV (Rust backend)")

            # Validate
            is_valid, issues = validate_dataframe(raw_df)
            if not is_valid:
                st.warning("⚠️ Data validation warnings:")
                for issue in issues:
                    st.warning(f"  - {issue}")

            # Prepare and enrich
            prepared_df = prepare_dataframe(raw_df)

            # Store in session state
            state.set_prepared_data(prepared_df)
            # Don't clear dark_periods - keep them loaded if they exist
            # Only clear the upload token so user can re-upload if needed
            st.session_state.pop("dark_periods_last_token", None)
            st.session_state[state.KEY_DATA_SOURCE] = source

            # Store filename
            if filename:
                # For JSON files, try to extract schedule name from the dataframe
                if file_type == "json" and "scheduleName" in prepared_df.columns:
                    # Use the first schedule name from the data
                    schedule_names = prepared_df["scheduleName"].dropna().unique()
                    if len(schedule_names) > 0:
                        state.set_data_filename(schedule_names[0])
                    else:
                        # Fallback to filename without extension
                        state.set_data_filename(filename.replace(".json", ""))
                else:
                    # For CSV or if no scheduleName column, use filename without extension
                    clean_filename = filename.replace(".csv", "").replace(".json", "")
                    state.set_data_filename(clean_filename)

            # Auto-navigate to first page
            state.set_current_page("Sky Map")

            # Try to auto-load dark periods after loading data
            if state.get_dark_periods() is None:
                _try_auto_load_dark_periods()

            # Force rerun to navigate away from landing page to the selected page
            st.rerun()

    except Exception as e:
        st.error(f"❌ Error loading data: {str(e)}")
        st.exception(e)
