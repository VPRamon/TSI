"""Lightweight analytics helpers for the Streamlit frontend."""

from tsi.services.data.analytics import (
    AnalyticsSnapshot,
    generate_correlation_insights,
    generate_insights,
)

__all__ = [
    "AnalyticsSnapshot",
    "generate_insights",
    "generate_correlation_insights",
]
