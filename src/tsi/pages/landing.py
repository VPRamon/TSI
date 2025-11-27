"""Landing page for the Telescope Scheduling Intelligence app."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import streamlit as st

from app_config import get_settings
from tsi import state
from tsi.services.loaders import load_csv, prepare_dataframe, validate_dataframe
from tsi.services.rust_compat import load_dark_periods_rust
from tsi.theme import add_vertical_space, render_landing_title


def render() -> None:
    """Render the landing page with data selection options."""

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
                # Pass the uploaded visibility file if it exists
                visibility_file = uploaded_visibility if uploaded_visibility is not None else None
                _load_data(
                    uploaded_json,
                    source="uploaded_json",
                    file_type="json",
                    visibility_file=visibility_file,
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
                # Use Rust backend for loading (10x faster)
                dark_periods_df = load_dark_periods_rust(dark_periods_path)
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

    # Try to auto-load dark periods if not already loaded
    # TODO: This shall be the input from the user, or calculated in backend.
    if state.get_dark_periods() is None:
        _try_auto_load_dark_periods()

    try:
        with st.spinner("Loading and processing data..."):
            if file_type == "json":
                # Use Rust backend for JSON preprocessing
                import tsi_rust
                
                # Handle file-like objects by reading content
                if hasattr(file_or_path, 'read'):
                    schedule_content = file_or_path.read()
                    if isinstance(schedule_content, bytes):
                        schedule_content = schedule_content.decode('utf-8')
                    # Reset file pointer if possible
                    if hasattr(file_or_path, 'seek'):
                        file_or_path.seek(0)
                    
                    visibility_content = None
                    if visibility_file is not None:
                        visibility_content = visibility_file.read()
                        if isinstance(visibility_content, bytes):
                            visibility_content = visibility_content.decode('utf-8')
                        if hasattr(visibility_file, 'seek'):
                            visibility_file.seek(0)
                    
                    # Preprocess using Rust backend (returns Polars DataFrame + ValidationResult)
                    df_polars, validation = tsi_rust.py_preprocess_schedule_str(
                        schedule_content, visibility_content, validate=True
                    )
                    raw_df = df_polars.to_pandas()
                else:
                    # File path - use file-based loader
                    df_polars, validation = tsi_rust.py_preprocess_schedule(
                        str(file_or_path), 
                        str(visibility_file) if visibility_file else None,
                        validate=True
                    )
                    raw_df = df_polars.to_pandas()

                # Show validation warnings if any
                if validation.warnings:
                    st.warning(f"⚠️ {len(validation.warnings)} data warnings found")
                    with st.expander("View warnings", expanded=False):
                        for warning in validation.warnings[:10]:
                            st.warning(f"  - {warning}")
                        if len(validation.warnings) > 10:
                            st.info(f"... and {len(validation.warnings) - 10} more")

                # Convert visibility lists to strings for Streamlit caching compatibility
                # Streamlit's cache_data uses pandas hashing which doesn't support list columns
                if "visibility" in raw_df.columns:
                    raw_df["visibility"] = raw_df["visibility"].apply(str)

            else:  # CSV
                # Load raw CSV (uses Rust backend - 10x faster)
                raw_df = load_csv(file_or_path)

            # For CSV files, perform additional validation
            if file_type == "csv":
                is_valid, issues = validate_dataframe(raw_df)
                if not is_valid and issues:
                    st.warning("⚠️ Additional data validation warnings:")
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
