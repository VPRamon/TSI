"""Validation issues display components."""

from __future__ import annotations

from typing import TYPE_CHECKING, Literal

import pandas as pd
import streamlit as st

if TYPE_CHECKING:
    from tsi_rust import ValidationReport


def _calculate_width_category(series: pd.Series) -> Literal["small", "medium", "large"]:
    """
    Heuristically pick a Streamlit column width bucket based on content length.

    Streamlit only allows small/medium/large widths, so we map the longest string
    in the column into one of those buckets.
    """
    max_len = series.astype(str).str.len().max() if not series.empty else 0
    if max_len <= 10:
        return "small"
    if max_len <= 24:
        return "medium"
    return "large"


def _get_criticality_emoji(criticality: str) -> str:
    """Get emoji for criticality level."""
    return {
        "Critical": "🔴",
        "High": "🟠",
        "Medium": "🟡",
        "Low": "🟢",
    }.get(criticality, "⚪")


def render_unified_validation_table(validation_data: ValidationReport) -> None:
    """
    Render all validation issues in a unified table with criticality column.

    Args:
        validation_data: Dictionary with validation information
    """
    st.subheader("📋 All Validation Issues")

    st.markdown(
        """
        Complete list of all data quality and scheduling issues found in the schedule.
        Issues are categorized by criticality level:
        - 🔴 **Critical**: Makes blocks impossible to schedule
        - 🟠 **High**: Serious data issues that may prevent scheduling
        - 🟡 **Medium**: Data issues that can likely be auto-corrected
        - 🟢 **Low**: Minor concerns or informational notices

        **Note:** Block IDs shown are the original IDs from your JSON file.
        """
    )

    # Collect all issues into a unified list
    all_issues = [issue for issue in validation_data.impossible_blocks]
    all_issues.extend(validation_data.validation_errors)
    all_issues.extend(validation_data.validation_warnings)

    if not all_issues:
        st.success("✅ No validation issues found")
        return

    # TO DISCUSS: should we get rid of DF?
    # Convert ValidationIssue objects to dicts for DataFrame
    issue_dicts = []
    for issue in all_issues:
        issue_dicts.append(
            {
                "Block ID": (
                    issue.original_block_id
                    if issue.original_block_id is not None
                    else str(issue.block_id)
                ),
                "Issue Type": issue.issue_type,
                "Category": issue.category,
                "Criticality": issue.criticality,
                "Field": issue.field_name if issue.field_name is not None else "",
                "Current Value": issue.current_value if issue.current_value is not None else "",
                "Expected/Issue": issue.expected_value if issue.expected_value is not None else "",
                "Description": issue.description,
            }
        )
    all_issues = issue_dicts

    # Create DataFrame
    df = pd.DataFrame(all_issues)

    # Sort by criticality (Critical first, then High, Medium, Low)
    criticality_order = {"Critical": 0, "High": 1, "Medium": 2, "Low": 3}
    df["_sort_order"] = df["Criticality"].map(criticality_order)
    df = df.sort_values("_sort_order").drop("_sort_order", axis=1)

    # Store original criticality for filtering before adding emojis
    df["_criticality_plain"] = df["Criticality"]

    # Add emoji to criticality column for display
    df["Criticality"] = df["Criticality"].apply(lambda x: f"{_get_criticality_emoji(x)} {x}")

    # Show filter controls
    col1, col2 = st.columns([1, 3])

    with col1:
        # Filter by criticality - use plain values for dropdown
        # Only show options that exist in the data
        available_options = ["All"] + [
            c for c in ["Critical", "High", "Medium", "Low"] if c in df["_criticality_plain"].values
        ]
        selected_criticality = st.selectbox(
            "Filter by Criticality", options=available_options, key="criticality_filter"
        )

    with col2:
        # Filter by issue type
        issue_types = ["All"] + sorted(df["Issue Type"].unique().tolist())
        selected_issue_type = st.selectbox(
            "Filter by Issue Type", options=issue_types, key="issue_type_filter"
        )

    # Apply filters using plain criticality
    filtered_df = df.copy()
    if selected_criticality != "All":
        filtered_df = filtered_df[filtered_df["_criticality_plain"] == selected_criticality]
    if selected_issue_type != "All":
        filtered_df = filtered_df[filtered_df["Issue Type"] == selected_issue_type]

    # Drop the helper column before display
    filtered_df = filtered_df.drop("_criticality_plain", axis=1)

    # Show count
    st.info(f"Showing {len(filtered_df)} of {len(df)} issues")

    # Calculate width buckets dynamically from the current data so narrow fields
    # (like Field) stay compact while long descriptions get more room.
    width_map = {col: _calculate_width_category(filtered_df[col]) for col in filtered_df.columns}

    # Display table with styling
    st.dataframe(
        filtered_df,
        width="stretch",
        hide_index=True,
        height=min(600, len(filtered_df) * 35 + 38),
        column_config={
            "Criticality": st.column_config.TextColumn(
                "Criticality",
                help="Severity of the issue",
                width=width_map.get("Criticality", "small"),
            ),
            "Block ID": st.column_config.TextColumn(
                "Block ID",
                help="Original scheduling block ID from JSON file",
                width=width_map.get("Block ID", "medium"),
            ),
            "Issue Type": st.column_config.TextColumn(
                "Issue Type",
                help="Category of validation issue",
                width=width_map.get("Issue Type", "medium"),
            ),
            "Field": st.column_config.TextColumn(
                "Field",
                help="Data field with the issue",
                width=width_map.get("Field", "small"),
            ),
            "Current Value": st.column_config.TextColumn(
                "Current Value",
                help="Actual value in the data",
                width=width_map.get("Current Value", "medium"),
            ),
            "Expected/Issue": st.column_config.TextColumn(
                "Expected/Issue",
                help="Expected value or description of issue",
                width=width_map.get("Expected/Issue", "medium"),
            ),
            "Description": st.column_config.TextColumn(
                "Description",
                help="Detailed explanation",
                width=width_map.get("Description", "large"),
            ),
        },
    )

    # Download button
    st.download_button(
        label="📥 Download Issues as CSV",
        data=filtered_df.to_csv(index=False),
        file_name="validation_issues.csv",
        mime="text/csv",
        help="Download the filtered validation issues as a CSV file",
    )
