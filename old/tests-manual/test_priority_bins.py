#!/usr/bin/env python3
"""Quick test of priority bins."""

import sys
from pathlib import Path

from core.loaders import load_schedule_from_json


def get_project_root() -> Path:
    """Locate the repository root so local imports work when run as a script."""
    for parent in Path(__file__).resolve().parents:
        if (parent / "pyproject.toml").exists():
            return parent
    raise RuntimeError("Project root not found")


sys.path.insert(0, str(get_project_root() / "src"))


def test_priority_bins():
    print("Testing priority bins with new validation...")
    print("=" * 60)

    schedule_path = Path("data/schedule.json")
    visibility_path = Path("data/possible_periods.json")

    result = load_schedule_from_json(schedule_path, visibility_path)
    df = result.dataframe

    print(f"\n✅ Loaded {len(df)} blocks")

    # Check validation
    print("\n📋 Validation Results:")
    print(f"   Valid: {result.validation.is_valid}")
    print(f"   Errors: {len(result.validation.errors)}")
    print(f"   Warnings: {len(result.validation.warnings)}")

    if result.validation.errors:
        print("\n❌ Errors:")
        for error in result.validation.errors:
            print(f"   - {error}")

    if result.validation.warnings:
        print("\n⚠️  Warnings:")
        for warning in result.validation.warnings:
            print(f"   - {warning}")

    # Check priority statistics
    print("\n📊 Priority Statistics:")
    print(f"   Min: {df['priority'].min():.2f}")
    print(f"   Max: {df['priority'].max():.2f}")
    print(f"   Mean: {df['priority'].mean():.2f}")
    print(f"   Median: {df['priority'].median():.2f}")

    # Check priority bins
    print("\n🎯 Priority Bins Distribution:")
    bin_counts = df["priority_bin"].value_counts().sort_index()
    for bin_name, count in bin_counts.items():
        pct = count / len(df) * 100
        print(f"   {bin_name:20s}: {count:4d} ({pct:5.1f}%)")

    # Show examples of high priority blocks
    print("\n🔝 Top 5 Highest Priority Blocks:")
    top5 = df.nlargest(5, "priority")[
        ["schedulingBlockId", "priority", "priority_bin", "scheduled_flag"]
    ]
    for _, row in top5.iterrows():
        print(
            f"   ID {row['schedulingBlockId']}: priority={row['priority']:.1f}, bin={row['priority_bin']}, scheduled={row['scheduled_flag']}"
        )

    print("\n" + "=" * 60)
    print("✅ Test complete!")


if __name__ == "__main__":
    test_priority_bins()
