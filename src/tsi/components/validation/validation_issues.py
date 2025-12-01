"""Validation issues display components."""

from __future__ import annotations

from typing import Any

import pandas as pd
import streamlit as st


def _get_criticality_emoji(criticality: str) -> str:
    """Get emoji for criticality level."""
    return {
        "Critical": "🔴",
        "High": "🟠",
        "Medium": "🟡",
        "Low": "🟢",
    }.get(criticality, "⚪")


def _get_criticality_color(criticality: str) -> str:
    """Get color for criticality level."""
    return {
        "Critical": "#ff4444",
        "High": "#ff8800",
        "Medium": "#ffcc00",
        "Low": "#44ff44",
    }.get(criticality, "#999999")


def render_unified_validation_table(validation_data: dict[str, Any]) -> None:
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
    all_issues = []
    
    # Process impossible blocks (always Critical)
    for block in validation_data.get("impossible_blocks", []):
        # Use original_block_id if available, otherwise fall back to block_id
        block_display_id = block.get("original_block_id") or str(block.get("block_id", "N/A"))
        all_issues.append({
            "Criticality": "Critical",
            "Block ID": block_display_id,
            "Issue Type": block.get("reason", "Unknown"),
            "Field": "Scheduling Constraint",
            "Current Value": f"{block.get('total_visibility_hours', 0):.2f}h available",
            "Expected/Issue": f"Needs {block.get('requested_duration_hours', 0):.2f}h",
            "Description": block.get("details", ""),
        })
    
    # Process validation errors
    for error in validation_data.get("validation_errors", []):
        # Use original_block_id if available, otherwise fall back to block_id
        block_display_id = error.get("original_block_id") or str(error.get("block_id", "N/A"))
        
        # Determine criticality based on error type
        error_type = error.get("error_type", "")
        if "out of range" in error_type.lower() or "negative" in error_type.lower():
            criticality = "High"
        elif "invalid" in error_type.lower():
            criticality = "Medium"
        else:
            criticality = "Medium"
        
        expected = error.get("expected_range", error.get("issue", ""))
        
        all_issues.append({
            "Criticality": criticality,
            "Block ID": block_display_id,
            "Issue Type": error.get("error_type", "Validation Error"),
            "Field": error.get("field", "N/A"),
            "Current Value": error.get("value", "N/A"),
            "Expected/Issue": expected,
            "Description": error.get("description", ""),
        })
    
    # Process validation warnings
    for warning in validation_data.get("validation_warnings", []):
        # Use original_block_id if available, otherwise fall back to block_id
        block_display_id = warning.get("original_block_id") or str(warning.get("block_id", "N/A"))
        
        # Determine criticality based on warning type
        warning_type = warning.get("warning_type", "")
        if "narrow" in warning_type.lower():
            criticality = "Medium"
        else:
            criticality = "Low"
        
        all_issues.append({
            "Criticality": criticality,
            "Block ID": block_display_id,
            "Issue Type": warning.get("warning_type", "Warning"),
            "Field": warning.get("field", "N/A"),
            "Current Value": warning.get("value", "N/A"),
            "Expected/Issue": warning.get("note", ""),
            "Description": warning.get("description", ""),
        })
    
    if not all_issues:
        st.success("✅ No validation issues found")
        return
    
    # Create DataFrame
    df = pd.DataFrame(all_issues)
    
    # Sort by criticality (Critical first, then High, Medium, Low)
    criticality_order = {"Critical": 0, "High": 1, "Medium": 2, "Low": 3}
    df["_sort_order"] = df["Criticality"].map(criticality_order)
    df = df.sort_values("_sort_order").drop("_sort_order", axis=1)
    
    # Store original criticality for filtering before adding emojis
    df["_criticality_plain"] = df["Criticality"]
    
    # Add emoji to criticality column for display
    df["Criticality"] = df["Criticality"].apply(
        lambda x: f"{_get_criticality_emoji(x)} {x}"
    )
    
    # Show filter controls
    col1, col2 = st.columns([1, 3])
    
    with col1:
        # Filter by criticality - use plain values for dropdown
        criticality_plain_options = ["All", "Critical", "High", "Medium", "Low"]
        # Only show options that exist in the data
        available_options = ["All"] + [c for c in ["Critical", "High", "Medium", "Low"] 
                                       if c in df["_criticality_plain"].values]
        selected_criticality = st.selectbox(
            "Filter by Criticality",
            options=available_options,
            key="criticality_filter"
        )
    
    with col2:
        # Filter by issue type
        issue_types = ["All"] + sorted(df["Issue Type"].unique().tolist())
        selected_issue_type = st.selectbox(
            "Filter by Issue Type",
            options=issue_types,
            key="issue_type_filter"
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
    
    # Display table with styling
    st.dataframe(
        filtered_df,
        use_container_width=True,
        hide_index=True,
        height=min(600, len(filtered_df) * 35 + 38),
            column_config={
            "Criticality": st.column_config.TextColumn(
                "Criticality",
                help="Severity of the issue",
                width="small",
            ),
            "Block ID": st.column_config.TextColumn(
                "Block ID",
                help="Original scheduling block ID from JSON file",
                width="medium",
            ),
            "Issue Type": st.column_config.TextColumn(
                "Issue Type",
                help="Category of validation issue",
                width="medium",
            ),
            "Field": st.column_config.TextColumn(
                "Field",
                help="Data field with the issue",
                width="small",
            ),
            "Current Value": st.column_config.TextColumn(
                "Current Value",
                help="Actual value in the data",
                width="medium",
            ),
            "Expected/Issue": st.column_config.TextColumn(
                "Expected/Issue",
                help="Expected value or description of issue",
                width="medium",
            ),
            "Description": st.column_config.TextColumn(
                "Description",
                help="Detailed explanation",
                width="large",
            ),
        }
    )
    
    # Download button
    st.download_button(
        label="📥 Download Issues as CSV",
        data=filtered_df.to_csv(index=False),
        file_name="validation_issues.csv",
        mime="text/csv",
        help="Download the filtered validation issues as a CSV file"
    )
