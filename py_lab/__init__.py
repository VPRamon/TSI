"""
py_lab - Data Science Laboratory for Schedule Analysis

A toolkit for loading, analyzing, and optimizing observation schedules
using constraint programming and statistical analysis.
"""

from .data_loader import (
    ScheduleLoader,
    load_schedule
)

from .conflict_analyzer import (
    ConflictAnalyzer,
    ConflictResult,
    detect_conflicts,
    get_schedulable_subset
)

from .utils import (
    mjd_to_datetime,
    datetime_to_mjd,
    format_mjd_period,
    merge_visibility_periods,
    compute_total_visibility_duration,
    filter_visibility_by_period,
    filter_by_priority,
    filter_by_sky_region,
    export_to_json,
    export_to_csv,
    validate_schedule_dataframe
)

__version__ = "0.1.0"

__all__ = [
    # Data loading
    "ScheduleLoader",
    "load_schedule",
    
    # Conflict analysis (CP-SAT)
    "ConflictAnalyzer",
    "ConflictResult",
    "detect_conflicts",
    "get_schedulable_subset",
    
    # Utilities
    "mjd_to_datetime",
    "datetime_to_mjd",
    "format_mjd_period",
    "merge_visibility_periods",
    "compute_total_visibility_duration",
    "filter_visibility_by_period",
    "filter_by_priority",
    "filter_by_sky_region",
    "export_to_json",
    "export_to_csv",
    "validate_schedule_dataframe",
]
