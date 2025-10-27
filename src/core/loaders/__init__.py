"""Data loading module for telescope scheduling data.

This module provides unified interfaces for loading scheduling data from various
sources (JSON, CSV) into standardized DataFrames ready for analysis and visualization.
"""

from .schedule_loader import (
    ScheduleLoadResult,
    load_schedule_from_csv,
    load_schedule_from_iteration,
    load_schedule_from_json,
)

__all__ = [
    "load_schedule_from_json",
    "load_schedule_from_csv",
    "load_schedule_from_iteration",
    "ScheduleLoadResult",
]
