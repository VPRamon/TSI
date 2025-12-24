"""Validation summary metrics display."""

from __future__ import annotations

from typing import TYPE_CHECKING, Literal

import streamlit as st

DeltaColor = Literal["normal", "inverse", "off"]


if TYPE_CHECKING:
    from tsi_rust import ValidationReport


def render_summary_metrics(validation_data: ValidationReport) -> None:
    """
    Render summary metrics for validation report.

    Args:
        validation_data: ValidationReport instance with validation information
    """
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        total_blocks = validation_data.total_blocks
        st.metric("Total Blocks", f"{total_blocks:,}")

    with col2:
        impossible_count = len(validation_data.impossible_blocks)
        delta_color: DeltaColor = "off" if impossible_count == 0 else "inverse"
        st.metric(
            "Impossible to Schedule",
            impossible_count,
            delta=f"{(impossible_count / total_blocks * 100):.1f}%" if total_blocks > 0 else "0%",
            delta_color=delta_color,
        )

    with col3:
        error_count = len(validation_data.validation_errors)
        delta_color_err: DeltaColor = "off" if error_count == 0 else "inverse"
        st.metric("Validation Errors", error_count, delta_color=delta_color_err)

    with col4:
        warning_count = len(validation_data.validation_warnings)
        delta_color_warn: DeltaColor = "off" if warning_count == 0 else "inverse"
        st.metric("Warnings", warning_count, delta_color=delta_color_warn)


def render_criticality_stats(validation_data: ValidationReport) -> None:
    """
    Render statistics by criticality level.

    Args:
        validation_data: Dictionary with validation information
    """
    st.subheader("📊 Issues by Criticality")

    # Count issues by criticality
    critical_count = 0
    high_count = 0
    medium_count = 0
    low_count = 0

    # Impossible blocks are always critical
    critical_count += len(validation_data.impossible_blocks)

    # Count validation errors by criticality
    for error in validation_data.validation_errors:
        criticality = error.get("criticality", "Medium")
        if criticality == "Critical":
            critical_count += 1
        elif criticality == "High":
            high_count += 1
        elif criticality == "Medium":
            medium_count += 1
        else:
            low_count += 1

    # Count warnings by criticality
    for warning in validation_data.validation_warnings:
        criticality = warning.criticality
        if criticality == "High":
            high_count += 1
        elif criticality == "Medium":
            medium_count += 1
        else:
            low_count += 1

    # Display in columns
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric(
            "🔴 Critical", critical_count, help="Issues that make blocks impossible to schedule"
        )

    with col2:
        st.metric(
            "🟠 High", high_count, help="Serious data quality issues that may prevent scheduling"
        )

    with col3:
        st.metric(
            "🟡 Medium", medium_count, help="Data issues that can likely be corrected automatically"
        )

    with col4:
        st.metric("🟢 Low", low_count, help="Minor concerns or informational notices")
