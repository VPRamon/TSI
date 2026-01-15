"""
Utilities Module for py_lab

Common helper functions for time conversion, data filtering, 
visibility period manipulation, and result export.
"""

from typing import List, Dict, Any, Tuple, Optional, Union
from pathlib import Path
import json
import pandas as pd
from datetime import datetime, timezone

# Try importing time utilities from src
try:
    from src.utils.time import mjd_to_timestamp, parse_period
    TIME_UTILS_AVAILABLE = True
except ImportError:
    TIME_UTILS_AVAILABLE = False


# ========================
# Time Conversion Utilities
# ========================

def mjd_to_datetime(mjd: float) -> datetime:
    """
    Convert Modified Julian Date to datetime.
    
    Args:
        mjd: Modified Julian Date (days since November 17, 1858)
        
    Returns:
        datetime object in UTC
    """
    if TIME_UTILS_AVAILABLE:
        return mjd_to_timestamp(mjd)
    else:
        # Fallback implementation
        # MJD = JD - 2400000.5
        # JD 0 = January 1, 4713 BC 12:00 UTC
        # Unix epoch (Jan 1, 1970) = JD 2440587.5 = MJD 40587.0
        unix_epoch_mjd = 40587.0
        seconds_per_day = 86400.0
        
        days_since_epoch = mjd - unix_epoch_mjd
        seconds_since_epoch = days_since_epoch * seconds_per_day
        
        return datetime.fromtimestamp(seconds_since_epoch, tz=timezone.utc)


def datetime_to_mjd(dt: datetime) -> float:
    """
    Convert datetime to Modified Julian Date.
    
    Args:
        dt: datetime object
        
    Returns:
        Modified Julian Date (days)
    """
    unix_epoch_mjd = 40587.0
    seconds_per_day = 86400.0
    
    timestamp = dt.timestamp()
    days_since_epoch = timestamp / seconds_per_day
    
    return unix_epoch_mjd + days_since_epoch


def format_mjd_period(start: float, stop: float) -> str:
    """
    Format an MJD period as a human-readable string.
    
    Args:
        start: Start MJD
        stop: Stop MJD
        
    Returns:
        Formatted string like "2024-01-15 10:30 to 2024-01-15 14:45 (4.25 hours)"
    """
    start_dt = mjd_to_datetime(start)
    stop_dt = mjd_to_datetime(stop)
    
    duration_days = stop - start
    duration_hours = duration_days * 24
    
    return (
        f"{start_dt.strftime('%Y-%m-%d %H:%M')} to "
        f"{stop_dt.strftime('%Y-%m-%d %H:%M')} "
        f"({duration_hours:.2f} hours)"
    )


# ========================
# Visibility Period Utilities
# ========================

def merge_visibility_periods(
    periods: List[Union[Dict[str, float], Tuple[float, float]]]
) -> List[Tuple[float, float]]:
    """
    Merge overlapping or adjacent visibility periods.
    
    Args:
        periods: List of periods as dicts {'start': ..., 'stop': ...} or tuples (start, stop)
        
    Returns:
        List of merged periods as tuples (start, stop)
    """
    if not periods:
        return []
    
    # Convert to tuples
    tuples = []
    for p in periods:
        if isinstance(p, dict):
            tuples.append((p['start'], p['stop']))
        elif isinstance(p, (tuple, list)):
            tuples.append((p[0], p[1]))
    
    # Sort by start time
    tuples = sorted(tuples, key=lambda x: x[0])
    
    # Merge overlapping
    merged = [tuples[0]]
    for start, stop in tuples[1:]:
        last_start, last_stop = merged[-1]
        
        # If overlapping or adjacent (within 1 minute)
        if start <= last_stop + (1.0 / 1440.0):  # 1 minute in MJD
            merged[-1] = (last_start, max(last_stop, stop))
        else:
            merged.append((start, stop))
    
    return merged


def compute_total_visibility_duration(
    periods: List[Union[Dict[str, float], Tuple[float, float]]]
) -> float:
    """
    Compute total duration of visibility periods in days.
    
    Args:
        periods: List of periods
        
    Returns:
        Total duration in days (MJD units)
    """
    merged = merge_visibility_periods(periods)
    return sum(stop - start for start, stop in merged)


def filter_visibility_by_period(
    visibility_periods: List[Union[Dict[str, float], Tuple[float, float]]],
    period_start: float,
    period_stop: float
) -> List[Tuple[float, float]]:
    """
    Filter visibility periods to only include those within a given time range.
    
    Args:
        visibility_periods: List of visibility periods
        period_start: Start of filter period (MJD)
        period_stop: Stop of filter period (MJD)
        
    Returns:
        Filtered and clipped visibility periods
    """
    filtered = []
    
    for period in visibility_periods:
        if isinstance(period, dict):
            start, stop = period['start'], period['stop']
        else:
            start, stop = period[0], period[1]
        
        # Check for overlap
        if stop <= period_start or start >= period_stop:
            continue
        
        # Clip to period boundaries
        clipped_start = max(start, period_start)
        clipped_stop = min(stop, period_stop)
        
        if clipped_stop > clipped_start:
            filtered.append((clipped_start, clipped_stop))
    
    return filtered


# ========================
# DataFrame Filtering Utilities
# ========================

def filter_by_priority(
    df: pd.DataFrame,
    min_priority: Optional[float] = None,
    max_priority: Optional[float] = None
) -> pd.DataFrame:
    """
    Filter DataFrame by priority range.
    
    Args:
        df: DataFrame with 'priority' column
        min_priority: Minimum priority (inclusive)
        max_priority: Maximum priority (inclusive)
        
    Returns:
        Filtered DataFrame
    """
    if 'priority' not in df.columns:
        raise ValueError("DataFrame must have 'priority' column")
    
    result = df.copy()
    
    if min_priority is not None:
        result = result[result['priority'] >= min_priority]
    
    if max_priority is not None:
        result = result[result['priority'] <= max_priority]
    
    return result


def filter_by_sky_region(
    df: pd.DataFrame,
    ra_min: Optional[float] = None,
    ra_max: Optional[float] = None,
    dec_min: Optional[float] = None,
    dec_max: Optional[float] = None
) -> pd.DataFrame:
    """
    Filter DataFrame by sky coordinates.
    
    Args:
        df: DataFrame with 'target_ra' and 'target_dec' columns
        ra_min: Minimum RA in degrees
        ra_max: Maximum RA in degrees
        dec_min: Minimum Dec in degrees
        dec_max: Maximum Dec in degrees
        
    Returns:
        Filtered DataFrame
    """
    if 'target_ra' not in df.columns or 'target_dec' not in df.columns:
        raise ValueError("DataFrame must have 'target_ra' and 'target_dec' columns")
    
    result = df.copy()
    
    if ra_min is not None:
        result = result[result['target_ra'] >= ra_min]
    if ra_max is not None:
        result = result[result['target_ra'] <= ra_max]
    if dec_min is not None:
        result = result[result['target_dec'] >= dec_min]
    if dec_max is not None:
        result = result[result['target_dec'] <= dec_max]
    
    return result


# Scheduled/unscheduled filtering removed - not needed for planning sessions


# ========================
# Export Utilities
# ========================

def export_to_json(
    data: Union[pd.DataFrame, Dict, List],
    output_path: Union[str, Path],
    indent: int = 2
) -> None:
    """
    Export data to JSON file.
    
    Args:
        data: DataFrame, dictionary, or list to export
        output_path: Output file path
        indent: JSON indentation level
    """
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    if isinstance(data, pd.DataFrame):
        # Convert DataFrame to records
        data = data.to_dict('records')
    
    with open(output_path, 'w') as f:
        json.dump(data, f, indent=indent, default=str)


def export_to_csv(
    df: pd.DataFrame,
    output_path: Union[str, Path],
    **kwargs
) -> None:
    """
    Export DataFrame to CSV file.
    
    Args:
        df: DataFrame to export
        output_path: Output file path
        **kwargs: Additional arguments for pd.to_csv()
    """
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    df.to_csv(output_path, **kwargs)


def export_conflict_report(
    conflict_results: Dict[str, Any],
    output_path: Union[str, Path]
) -> None:
    """
    Export conflict analysis results to a formatted JSON report.
    
    Args:
        conflict_results: Results from conflict_analyzer
        output_path: Output file path
    """
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    report = {
        'timestamp': datetime.now(timezone.utc).isoformat(),
        'analysis': conflict_results
    }
    
    with open(output_path, 'w') as f:
        json.dump(report, f, indent=2, default=str)


# ========================
# Data Validation Utilities
# ========================

def validate_schedule_dataframe(df: pd.DataFrame) -> Tuple[bool, List[str]]:
    """
    Validate that a DataFrame has the required columns for CP-SAT planning.
    
    Args:
        df: DataFrame to validate
        
    Returns:
        Tuple of (is_valid, list_of_missing_columns)
    """
    required_columns = [
        'id',
        'priority',
        'requested_duration',
        'visibility_periods'
    ]
    
    missing = [col for col in required_columns if col not in df.columns]
    
    return len(missing) == 0, missing


def get_dataframe_memory_usage(df: pd.DataFrame) -> Dict[str, Any]:
    """
    Get memory usage statistics for a DataFrame.
    
    Args:
        df: DataFrame to analyze
        
    Returns:
        Dictionary with memory usage information
    """
    memory_bytes = df.memory_usage(deep=True).sum()
    
    return {
        'total_bytes': int(memory_bytes),
        'total_mb': float(memory_bytes / (1024 * 1024)),
        'per_row_bytes': float(memory_bytes / len(df)) if len(df) > 0 else 0,
        'shape': df.shape,
        'columns': list(df.columns),
        'dtypes': df.dtypes.astype(str).to_dict()
    }


# ========================
# Sample Data Generation
# ========================

def generate_sample_task_ids(df: pd.DataFrame, n: int = 10, method: str = 'random') -> List[str]:
    """
    Generate sample task IDs from a DataFrame.
    
    Args:
        df: DataFrame with 'id' column
        n: Number of samples
        method: Sampling method ('random', 'high_priority', 'low_priority')
        
    Returns:
        List of task IDs
    """
    if 'id' not in df.columns:
        raise ValueError("DataFrame must have 'id' column")
    
    if method == 'random':
        sample = df.sample(min(n, len(df)))
    elif method == 'high_priority' and 'priority' in df.columns:
        sample = df.nlargest(min(n, len(df)), 'priority')
    elif method == 'low_priority' and 'priority' in df.columns:
        sample = df.nsmallest(min(n, len(df)), 'priority')
    else:
        sample = df.head(n)
    
    return sample['id'].astype(str).tolist()


if __name__ == "__main__":
    """Example usage and testing"""
    import sys
    from pathlib import Path
    
    # Add parent to path
    sys.path.insert(0, str(Path(__file__).parent.parent))
    
    print("=" * 60)
    print("Utilities Module - py_lab")
    print("=" * 60)
    
    # Test time conversion
    print("\n" + "=" * 60)
    print("Test 1: Time Conversion")
    print("=" * 60)
    
    mjd_now = 60000.5  # Example MJD
    dt = mjd_to_datetime(mjd_now)
    mjd_back = datetime_to_mjd(dt)
    
    print(f"MJD: {mjd_now}")
    print(f"DateTime: {dt}")
    print(f"Back to MJD: {mjd_back}")
    print(f"Difference: {abs(mjd_now - mjd_back):.10f} days")
    
    # Test period formatting
    print("\n" + "=" * 60)
    print("Test 2: Period Formatting")
    print("=" * 60)
    
    period_str = format_mjd_period(60000.0, 60000.25)
    print(f"Period: {period_str}")
    
    # Test visibility period merging
    print("\n" + "=" * 60)
    print("Test 3: Visibility Period Merging")
    print("=" * 60)
    
    periods = [
        {'start': 60000.0, 'stop': 60000.1},
        {'start': 60000.09, 'stop': 60000.15},  # Overlaps
        {'start': 60000.3, 'stop': 60000.4}     # Separate
    ]
    
    merged = merge_visibility_periods(periods)
    print(f"Original periods: {len(periods)}")
    print(f"Merged periods: {len(merged)}")
    for start, stop in merged:
        print(f"  {start} - {stop} ({(stop-start)*24:.2f} hours)")
    
    total_duration = compute_total_visibility_duration(periods)
    print(f"Total duration: {total_duration*24:.2f} hours")
    
    # Test DataFrame filtering
    print("\n" + "=" * 60)
    print("Test 4: DataFrame Filtering")
    print("=" * 60)
    
    # Create sample DataFrame
    sample_df = pd.DataFrame({
        'id': range(100),
        'priority': [i % 10 + 1 for i in range(100)],
        'target_ra': [i * 3.6 for i in range(100)],
        'target_dec': [(i % 180) - 90 for i in range(100)],
        'requested_duration': [3600 + i*100 for i in range(100)],
        'visibility_periods': [[(60000 + i*0.01, 60000 + i*0.01 + 0.1)] for i in range(100)]
    })
    
    print(f"Original DataFrame: {len(sample_df)} rows")
    
    high_priority = filter_by_priority(sample_df, min_priority=8)
    print(f"High priority (>= 8): {len(high_priority)} rows")
    
    northern = filter_by_sky_region(sample_df, dec_min=0)
    print(f"Northern hemisphere: {len(northern)} rows")
    
    # Test validation
    print("\n" + "=" * 60)
    print("Test 5: DataFrame Validation")
    print("=" * 60)
    
    valid, missing = validate_schedule_dataframe(sample_df)
    print(f"Valid: {valid}")
    if missing:
        print(f"Missing columns: {missing}")
    
    # Test memory usage
    print("\n" + "=" * 60)
    print("Test 6: Memory Usage")
    print("=" * 60)
    
    mem_info = get_dataframe_memory_usage(sample_df)
    print(f"Total memory: {mem_info['total_mb']:.2f} MB")
    print(f"Per row: {mem_info['per_row_bytes']:.1f} bytes")
    
    print("\n" + "=" * 60)
    print("Utilities module is ready!")
    print("=" * 60)
