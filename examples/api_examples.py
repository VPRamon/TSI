"""
TSI Rust Backend - API Usage Examples

Demonstrates common use cases for the TSI Rust Backend Python API.
"""

from pathlib import Path
from tsi_rust_api import TSIBackend, load_schedule, compute_metrics

# Setup
DATA_DIR = Path(__file__).parent.parent / "data"
backend = TSIBackend(use_pandas=True)


def example_1_basic_loading():
    """Example 1: Load and inspect schedule data"""
    print("=" * 60)
    print("Example 1: Basic Data Loading")
    print("=" * 60)
    
    # Load CSV file
    df = backend.load_schedule(DATA_DIR / "schedule.json")
    
    print(f"\nLoaded {len(df)} observations")
    print(f"Columns: {list(df.columns)}")
    print(f"\nFirst 3 rows:")
    print(df.head(3))
    
    # Quick stats
    print(f"\nScheduled: {df['scheduled_flag'].sum()} / {len(df)}")
    print(f"Priority range: {df['priority'].min():.1f} - {df['priority'].max():.1f}")


def example_2_analytics():
    """Example 2: Compute scheduling metrics"""
    print("\n" + "=" * 60)
    print("Example 2: Analytics & Metrics")
    print("=" * 60)
    
    df = backend.load_schedule(DATA_DIR / "schedule.json")
    
    # Compute comprehensive metrics
    metrics = backend.compute_metrics(df)
    
    print(f"\nScheduling Metrics:")
    print(f"  Total observations: {metrics['total_observations']}")
    print(f"  Scheduled: {metrics['scheduled_count']} ({metrics['scheduled_percentage']:.1f}%)")
    print(f"  Unscheduled: {metrics['unscheduled_count']}")
    print(f"  Mean priority: {metrics['mean_priority']:.2f}")
    print(f"  Median priority: {metrics['median_priority']:.2f}")
    print(f"  Mean duration: {metrics['mean_duration_hours']:.2f} hours")
    print(f"  Total visibility: {metrics['total_visibility_hours']:.1f} hours")


def example_3_filtering():
    """Example 3: Filter observations by criteria"""
    print("\n" + "=" * 60)
    print("Example 3: Filtering Data")
    print("=" * 60)
    
    df = backend.load_schedule(DATA_DIR / "schedule.json")
    
    # Filter by priority
    high_priority = backend.filter_by_priority(df, min_priority=15.0)
    print(f"\nHigh priority (≥15): {len(high_priority)} observations")
    
    # Filter by scheduling status
    unscheduled = backend.filter_by_scheduled(df, "Unscheduled")
    print(f"Unscheduled: {len(unscheduled)} observations")
    
    # Complex multi-filter
    filtered = backend.filter_dataframe(
        df,
        priority_min=10.0,
        priority_max=20.0,
        scheduled_filter="Scheduled",
        priority_bins=["High", "Very High"]
    )
    print(f"\nComplex filter (priority 10-20, scheduled, high bins):")
    print(f"  Result: {len(filtered)} observations")


def example_4_top_observations():
    """Example 4: Get top observations"""
    print("\n" + "=" * 60)
    print("Example 4: Top Observations")
    print("=" * 60)
    
    df = backend.load_schedule(DATA_DIR / "schedule.json")
    
    # Top 10 by priority
    top_priority = backend.get_top_observations(df, n=10, by="priority")
    
    print(f"\nTop 10 observations by priority:")
    print(top_priority[['schedulingBlockId', 'priority', 'scheduled_flag', 'requested_hours']].to_string())


def example_5_conflicts():
    """Example 5: Find scheduling conflicts"""
    print("\n" + "=" * 60)
    print("Example 5: Conflict Detection")
    print("=" * 60)
    
    df = backend.load_schedule(DATA_DIR / "schedule.json")
    
    # Find conflicts
    conflicts = backend.find_conflicts(df)
    
    print(f"\nFound {len(conflicts)} scheduling conflicts")
    if len(conflicts) > 0:
        print(f"\nFirst 5 conflicts:")
        print(conflicts.head(5))


def example_6_data_cleaning():
    """Example 6: Data cleaning operations"""
    print("\n" + "=" * 60)
    print("Example 6: Data Cleaning")
    print("=" * 60)
    
    df = backend.load_schedule(DATA_DIR / "schedule.json")
    
    print(f"\nOriginal data: {len(df)} rows")
    
    # Remove duplicates
    unique_df = backend.remove_duplicates(df, subset=["schedulingBlockId"])
    print(f"After removing duplicates: {len(unique_df)} rows")
    
    # Remove invalid coordinates
    valid_coords = backend.remove_missing_coordinates(df)
    print(f"With valid coordinates: {len(valid_coords)} rows")
    
    # Validate data quality
    is_valid, issues = backend.validate_dataframe(df)
    print(f"\nData validation: {'✓ PASS' if is_valid else '✗ FAIL'}")
    if issues:
        print("Issues found:")
        for issue in issues:
            print(f"  - {issue}")


def example_7_time_conversions():
    """Example 7: MJD ↔ Datetime conversions"""
    print("\n" + "=" * 60)
    print("Example 7: Time Conversions")
    print("=" * 60)
    
    # MJD to datetime
    mjd = 59580.5
    dt = TSIBackend.mjd_to_datetime(mjd)
    print(f"\nMJD {mjd} → {dt}")
    
    # Datetime to MJD
    dt_str = "2022-01-01T12:00:00Z"
    mjd_back = TSIBackend.datetime_to_mjd(dt_str)
    print(f"{dt_str} → MJD {mjd_back}")
    
    # Roundtrip verification
    print(f"\nRoundtrip error: {abs(mjd - mjd_back):.10f}")


def example_8_optimization():
    """Example 8: Greedy scheduling optimization"""
    print("\n" + "=" * 60)
    print("Example 8: Scheduling Optimization")
    print("=" * 60)
    
    df = backend.load_schedule(DATA_DIR / "schedule.json")
    
    # Run greedy scheduler
    result = backend.greedy_schedule(df, max_iterations=500)
    
    print(f"\nOptimization Results:")
    print(f"  Selected: {len(result['selected_ids'])} observations")
    print(f"  Total duration: {result['total_duration']:.1f} hours")
    print(f"  Iterations: {result['iterations_run']}")
    print(f"  Converged: {result['converged']}")
    
    # Show selected IDs
    print(f"\nFirst 10 selected IDs:")
    for i, obs_id in enumerate(result['selected_ids'][:10], 1):
        print(f"  {i}. {obs_id}")


def example_9_convenience_functions():
    """Example 9: Quick convenience functions"""
    print("\n" + "=" * 60)
    print("Example 9: Convenience Functions")
    print("=" * 60)
    
    # Load without creating backend instance
    df = load_schedule(DATA_DIR / "schedule.json")
    print(f"\nLoaded {len(df)} observations (convenience function)")
    
    # Compute metrics without instance
    metrics = compute_metrics(df)
    print(f"Scheduled: {metrics['scheduled_percentage']:.1f}%")


def example_10_polars_mode():
    """Example 10: Using Polars (zero-copy mode)"""
    print("\n" + "=" * 60)
    print("Example 10: Polars Mode (Maximum Performance)")
    print("=" * 60)
    
    # Use Polars for zero-copy performance
    backend_polars = TSIBackend(use_pandas=False)
    
    df_polars = backend_polars.load_schedule(DATA_DIR / "schedule.json")
    print(f"\nLoaded as Polars DataFrame: {len(df_polars)} rows")
    print(f"Type: {type(df_polars)}")
    
    # All operations work with Polars
    metrics = backend_polars.compute_metrics(df_polars)
    print(f"Metrics computed on Polars: {metrics['scheduled_percentage']:.1f}% scheduled")


def run_all_examples():
    """Run all examples"""
    examples = [
        example_1_basic_loading,
        example_2_analytics,
        example_3_filtering,
        example_4_top_observations,
        example_5_conflicts,
        example_6_data_cleaning,
        example_7_time_conversions,
        example_8_optimization,
        example_9_convenience_functions,
        example_10_polars_mode,
    ]
    
    print("\n" + "=" * 60)
    print("TSI Rust Backend - API Examples")
    print("=" * 60)
    
    for example in examples:
        try:
            example()
        except Exception as e:
            print(f"\n❌ Error in {example.__name__}: {e}")
    
    print("\n" + "=" * 60)
    print("All examples completed!")
    print("=" * 60)


if __name__ == "__main__":
    run_all_examples()
