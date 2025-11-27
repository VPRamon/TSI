#!/usr/bin/env python3
"""
Demonstration script showing the Rust schedule preprocessor in action.

This script processes a schedule using the new Rust backend and displays
comprehensive statistics and validation results.
"""

import sys
from pathlib import Path

import tsi_rust


def main():
    print("=" * 70)
    print("TSI Rust Schedule Preprocessor - Demonstration")
    print("=" * 70)
    print()

    # Input files
    schedule_path = "data/schedule.json"
    visibility_path = "data/possible_periods.json"

    if not Path(schedule_path).exists():
        print(f"Error: {schedule_path} not found")
        return 1

    print(f"Processing:")
    print(f"  Schedule: {schedule_path}")
    print(f"  Visibility: {visibility_path}")
    print()

    # Process with Rust backend
    print("Running Rust preprocessing pipeline...")
    df, validation = tsi_rust.py_preprocess_schedule(
        schedule_path, visibility_path, validate=True
    )

    # Convert to pandas for display
    df_pd = df.to_pandas()

    print("✅ Processing complete!")
    print()

    # Display DataFrame info
    print("-" * 70)
    print("DataFrame Summary")
    print("-" * 70)
    print(f"  Rows: {len(df_pd):,}")
    print(f"  Columns: {len(df_pd.columns)}")
    print()
    print("  Column Names:")
    for i, col in enumerate(df_pd.columns, 1):
        print(f"    {i:2d}. {col}")
    print()

    # Display validation results
    print("-" * 70)
    print("Validation Results")
    print("-" * 70)
    if validation.is_valid:
        print("  Status: ✅ PASS")
    else:
        print("  Status: ❌ FAIL")

    print(f"  Errors: {len(validation.errors)}")
    if validation.errors:
        for error in validation.errors[:5]:
            print(f"    - {error}")
        if len(validation.errors) > 5:
            print(f"    ... and {len(validation.errors) - 5} more")

    print(f"  Warnings: {len(validation.warnings)}")
    if validation.warnings:
        for warning in validation.warnings[:5]:
            print(f"    - {warning}")
        if len(validation.warnings) > 5:
            print(f"    ... and {len(validation.warnings) - 5} more")
    print()

    # Display statistics
    stats = validation.get_stats()
    print("-" * 70)
    print("Statistics")
    print("-" * 70)
    print(f"  Total Blocks: {stats['total_blocks']:,}")
    print(f"  Scheduled: {stats['scheduled_blocks']:,}")
    print(f"  Unscheduled: {stats['unscheduled_blocks']:,}")
    print(
        f"  Scheduling Rate: {stats['scheduled_blocks']/stats['total_blocks']*100:.1f}%"
    )
    print()
    print(f"  Blocks with Visibility: {stats['blocks_with_visibility']:,}")
    print(f"  Avg Visibility Periods: {stats['avg_visibility_periods']:.2f}")
    print(f"  Avg Visibility Hours: {stats['avg_visibility_hours']:.2f}")
    print()
    print(f"  Missing Coordinates: {stats['missing_coordinates']:,}")
    print(f"  Missing Constraints: {stats['missing_constraints']:,}")
    print(f"  Duplicate IDs: {stats['duplicate_ids']:,}")
    print(f"  Invalid Priorities: {stats['invalid_priorities']:,}")
    print(f"  Invalid Durations: {stats['invalid_durations']:,}")
    print()

    # Sample data
    print("-" * 70)
    print("Sample Data (First 3 Rows)")
    print("-" * 70)
    sample_cols = [
        "schedulingBlockId",
        "targetId",
        "targetName",
        "priority",
        "scheduled_flag",
    ]
    print(df_pd[sample_cols].head(3).to_string(index=False))
    print()

    # Priority distribution
    print("-" * 70)
    print("Priority Distribution")
    print("-" * 70)
    priority_dist = df_pd["priority_bin"].value_counts().sort_index()
    for bin_name, count in priority_dist.items():
        pct = count / len(df_pd) * 100
        bar = "█" * int(pct / 2)
        print(f"  {bin_name:20s} {count:4d} ({pct:5.1f}%) {bar}")
    print()

    print("=" * 70)
    print("✅ Demonstration Complete")
    print("=" * 70)

    return 0


if __name__ == "__main__":
    sys.exit(main())
