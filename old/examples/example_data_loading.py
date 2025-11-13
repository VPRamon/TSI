#!/usr/bin/env python3
"""
Example script demonstrating the unified data loading system.

This shows how to use the centralized loaders for different use cases.
"""

import sys
from pathlib import Path


def get_project_root() -> Path:
    """Locate the repository root containing ``pyproject.toml``."""
    for parent in Path(__file__).resolve().parents:
        if (parent / "pyproject.toml").exists():
            return parent
    raise RuntimeError("Unable to locate project root")


# Add src to path
PROJECT_ROOT = get_project_root()
sys.path.insert(0, str(PROJECT_ROOT / 'src'))

from core.loaders import (
    load_schedule_from_json,
    load_schedule_from_csv,
    load_schedule_from_iteration,
)


def example_1_load_from_csv():
    """Example 1: Load from preprocessed CSV file."""
    print("\n" + "="*60)
    print("Example 1: Loading from CSV")
    print("="*60)
    
    csv_path = Path("data/schedule.csv")
    
    if not csv_path.exists():
        print(f"⚠️  CSV file not found: {csv_path}")
        print("To create a preprocessed CSV, run:")
        print("  python preprocess_schedules.py --schedule data/schedule.json --visibility data/possible_periods.json --output data/schedule.csv")
        return None
    
    result = load_schedule_from_csv(csv_path)
    
    print(f"✅ Loaded {len(result.dataframe)} scheduling blocks")
    print(f"📊 Source: {result.source_type}")
    print(f"📁 Path: {result.source_path}")
    print(f"\nValidation stats:")
    for key, value in result.validation.stats.items():
        print(f"  {key}: {value}")
    
    return result.dataframe


def example_2_load_from_json_files():
    """Example 2: Load directly from JSON files."""
    print("\n" + "="*60)
    print("Example 2: Loading from JSON files")
    print("="*60)
    
    schedule_json = Path("data/schedule.json")
    visibility_json = Path("data/possible_periods.json")
    
    if not schedule_json.exists():
        print(f"⚠️  JSON file not found: {schedule_json}")
        return None
    
    result = load_schedule_from_json(
        schedule_json,
        visibility_json if visibility_json.exists() else None
    )
    
    print(f"✅ Loaded {len(result.dataframe)} scheduling blocks")
    print(f"📊 Source: {result.source_type}")
    
    # Show validation warnings
    if result.validation.warnings:
        print(f"\n⚠️  {len(result.validation.warnings)} warnings:")
        for warning in result.validation.warnings[:3]:
            print(f"  • {warning}")
        if len(result.validation.warnings) > 3:
            print(f"  ... and {len(result.validation.warnings) - 3} more")
    
    # Show sample data
    print(f"\nFirst 3 scheduling blocks:")
    cols = ['schedulingBlockId', 'priority', 'scheduled_flag', 'total_visibility_hours']
    print(result.dataframe[cols].head(3).to_string(index=False))
    
    return result.dataframe


def example_4_analyze_data(df):
    """Example 4: Basic data analysis."""
    print("\n" + "="*60)
    print("Example 4: Basic data analysis")
    print("="*60)
    
    if df is None:
        print("⚠️  No data loaded")
        return
    
    total = len(df)
    scheduled = df['scheduled_flag'].sum()
    unscheduled = total - scheduled
    
    print(f"\n📈 Scheduling Statistics:")
    print(f"  Total blocks: {total:,}")
    print(f"  Scheduled: {scheduled:,} ({scheduled/total*100:.1f}%)")
    print(f"  Unscheduled: {unscheduled:,} ({unscheduled/total*100:.1f}%)")
    
    print(f"\n🎯 Priority Distribution:")
    print(df['priority_bin'].value_counts().to_string())
    
    print(f"\n⏰ Visibility Summary:")
    print(f"  Mean visibility hours: {df['total_visibility_hours'].mean():.2f}")
    print(f"  Median visibility hours: {df['total_visibility_hours'].median():.2f}")
    print(f"  Max visibility hours: {df['total_visibility_hours'].max():.2f}")
    
    print(f"\n🔍 Top 5 by priority:")
    top_cols = ['schedulingBlockId', 'priority', 'total_visibility_hours', 'scheduled_flag']
    print(df.nlargest(5, 'priority')[top_cols].to_string(index=False))


def main():
    """Run all examples."""
    print("\n" + "🌌" * 30)
    print("Telescope Scheduling Intelligence - Data Loading Examples")
    print("🌌" * 30)
    
    # Example 1: CSV (fastest, recommended)
    df = example_1_load_from_csv()
    
    # Example 2: Direct JSON loading (flexible)
    if df is None:
        df = example_2_load_from_json_files()
    
    # Example 3: Data directory loading (legacy support)
    # Uncomment if you have legacy directory structure
    # if df is None:
    #     df = example_3_load_from_iteration()
    
    # Example 4: Analyze the loaded data
    if df is not None:
        example_4_analyze_data(df)
    else:
        print("\n❌ No data could be loaded. Please check the data directory.")
        print("\nTo create sample data, run:")
        print("  python preprocess_schedules.py --schedule data/schedule.json --visibility data/possible_periods.json --output data/schedule.csv")
    
    print("\n" + "="*60)
    print("✅ Examples complete!")
    print("="*60)
    print("\nFor more information, see doc/DATA_LOADING_ARCHITECTURE.md")
    print("For preprocessing help: python preprocess_schedules.py --help")


if __name__ == "__main__":
    main()
