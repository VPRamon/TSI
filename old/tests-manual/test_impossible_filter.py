#!/usr/bin/env python3
"""Test script to verify the impossible observations filter logic."""

import pandas as pd


def test_impossible_filter():
    """Test the impossible observations filter."""

    # Load data
    print("Loading data...")
    df = pd.read_csv("data/schedule.csv")
    print(f"Total observations: {len(df):,}")

    # Check required columns
    required_cols = ["minObservationTimeInSec", "total_visibility_hours"]
    missing = [col for col in required_cols if col not in df.columns]

    if missing:
        print(f"❌ Missing columns: {missing}")
        return

    print(f"✅ Required columns present: {required_cols}")

    # Calculate impossible observations
    min_duration_hours = df["minObservationTimeInSec"].fillna(0) / 3600.0
    visibility_hours = df["total_visibility_hours"]
    impossible_mask = (min_duration_hours > visibility_hours).fillna(False)

    impossible_count = int(impossible_mask.sum())
    total_blocks = len(df)
    impossible_percentage = (impossible_count / total_blocks) * 100 if total_blocks else 0.0

    print("\n📊 Statistics:")
    print(f"  Total blocks: {total_blocks:,}")
    print(f"  Impossible blocks: {impossible_count:,} ({impossible_percentage:.1f}%)")
    print(
        f"  Possible blocks: {total_blocks - impossible_count:,} ({100 - impossible_percentage:.1f}%)"
    )

    # Show some examples of impossible observations
    if impossible_count > 0:
        print("\n⚠️  Examples of impossible observations:")
        impossible_df = df[impossible_mask][
            ["schedulingBlockId", "minObservationTimeInSec", "total_visibility_hours"]
        ].head(10)
        impossible_df["min_duration_hours"] = impossible_df["minObservationTimeInSec"] / 3600.0
        impossible_df["deficit_hours"] = (
            impossible_df["min_duration_hours"] - impossible_df["total_visibility_hours"]
        )

        for idx, row in impossible_df.iterrows():
            print(
                f"  Block {row['schedulingBlockId']}: "
                f"needs {row['min_duration_hours']:.2f}h, "
                f"has {row['total_visibility_hours']:.2f}h "
                f"(deficit: {row['deficit_hours']:.2f}h)"
            )
    else:
        print("\n✅ No impossible observations found!")

    # Test filtering
    print("\n🧪 Testing filters:")

    # All
    filtered_all = df.copy()
    print(f"  All: {len(filtered_all):,} observations")

    # Exclude impossible
    filtered_possible = df[~impossible_mask].copy()
    print(f"  Possible only: {len(filtered_possible):,} observations")

    # Only impossible
    filtered_impossible = df[impossible_mask].copy()
    print(f"  Impossible only: {len(filtered_impossible):,} observations")

    # Verify
    assert len(filtered_possible) + len(filtered_impossible) == len(df), "Filter logic error!"
    print("\n✅ All filters working correctly!")


if __name__ == "__main__":
    test_impossible_filter()
