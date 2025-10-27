"""
Example of using the schedule pre-processor.

This script demonstrates how to use the preprocessing module
to convert JSON files to optimized CSVs.

Requires: pandas, numpy (see requirements.txt)
"""

import sys
from pathlib import Path


def get_project_root() -> Path:
    """Returns the root of the project that contains ``pyproject.toml``."""
    for parent in Path(__file__).resolve().parents:
        if (parent / "pyproject.toml").exists():
            return parent
    raise RuntimeError("Could not locate project root")


# Add project root to path
PROJECT_ROOT = get_project_root()
sys.path.insert(0, str(PROJECT_ROOT))

from src.core.preprocessing import (
    SchedulePreprocessor,
    preprocess_iteration,
    preprocess_schedule,
)


def example_single_file():
    """Example: Process schedule and visibility JSON files directly."""
    print("=" * 70)
    print("EXAMPLE 1: Process JSON files with SchedulePreprocessor")
    print("=" * 70)

    # Paths to JSON files
    schedule_path = Path("data/schedule.json")
    visibility_path = Path("data/possible_periods.json")
    output_path = Path("data/preprocessed/schedule_example.csv")

    if not schedule_path.exists():
        print(f"\n⚠️  Schedule file not found: {schedule_path}")
        print("Please ensure data/schedule.json exists")
        return

    # Create preprocessor
    preprocessor = SchedulePreprocessor(
        schedule_path, 
        visibility_path if visibility_path.exists() else None
    )

    # Load data
    print(f"\n1. Loading data from {schedule_path.name}...")
    preprocessor.load_data()

    # Extract DataFrame
    print("2. Extracting scheduling blocks...")
    df = preprocessor.extract_dataframe()
    print(f"   ✓ {len(df)} blocks extracted")

    # Enrich with visibility
    if visibility_path.exists():
        print("3. Enriching with visibility data...")
        preprocessor.enrich_with_visibility()
    else:
        print("3. Skipping visibility enrichment (file not found)")

    # Add derived columns
    print("4. Adding derived columns...")
    preprocessor.add_derived_columns()

    # Validate
    print("5. Validating data...")
    validation = preprocessor.validate()
    
    if validation.is_valid:
        print("   ✓ Validation successful")
    else:
        print("   ✗ Validation failed:")
        for error in validation.errors:
            print(f"     - {error}")
    
    if validation.warnings:
        print("   ⚠ Warnings:")
        for warning in validation.warnings[:3]:
            print(f"     - {warning}")

    # Show statistics
    print("\n6. Statistics:")
    print(f"   - Total blocks: {validation.stats['total_blocks']}")
    print(f"   - Scheduled: {validation.stats['scheduled_blocks']}")
    print(f"   - Unscheduled: {validation.stats['unscheduled_blocks']}")

    if 'avg_visibility_periods' in validation.stats:
        print(f"   - Average visibility periods: {validation.stats['avg_visibility_periods']:.1f}")
        print(f"   - Average visibility hours: {validation.stats['avg_visibility_hours']:.1f}")

    # Export
    print(f"\n7. Exporting to {output_path}...")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    result_path = preprocessor.to_csv(output_path, validate=True)
    print(f"   ✓ CSV saved: {result_path}")
    print(f"   ✓ Size: {result_path.stat().st_size / 1024:.1f} KB")


def example_convenience_function():
    """Example: Use convenience function preprocess_schedule()."""
    print("\n" + "=" * 70)
    print("EXAMPLE 2: Use convenience function preprocess_schedule()")
    print("=" * 70)
    
    schedule_path = Path("data/schedule.json")
    visibility_path = Path("data/possible_periods.json")
    output_path = Path("data/preprocessed/schedule_convenience.csv")

    if not schedule_path.exists():
        print(f"\n⚠️  Schedule file not found: {schedule_path}")
        return

    print(f"\nProcessing {schedule_path.name}...")

    try:
        result_path = preprocess_schedule(
            schedule_path,
            visibility_path if visibility_path.exists() else None,
            output_path,
            validate=True
        )

        print(f"✓ Processing complete")
        print(f"  File: {result_path}")
        print(f"  Size: {result_path.stat().st_size / 1024:.1f} KB")

    except Exception as e:
        print(f"✗ Error: {e}")


def example_batch_processing():
    """Example: Process multiple schedule files."""
    print("\n" + "=" * 70)
    print("EXAMPLE 3: Batch processing of multiple schedule files")
    print("=" * 70)
    
    # For this example, we'll process files in a batch directory
    batch_dir = Path("data/batch_schedules")
    output_dir = Path("data/preprocessed")

    if not batch_dir.exists():
        print(f"\n⚠️  Batch directory not found: {batch_dir}")
        print("This example requires a directory with multiple schedule JSON files")
        print("\nAlternatively, use the CLI tool for batch processing:")
        print("  python preprocess_schedules.py --batch-dir data/schedules --output-dir data/preprocessed")
        return

    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)

    # Find schedule JSON files
    schedule_files = list(batch_dir.glob("schedule*.json"))

    if not schedule_files:
        print(f"\n⚠️  No schedule files found in {batch_dir}")
        return

    print(f"\nFound {len(schedule_files)} schedule files")

    successful = 0
    failed = 0
    
    for schedule_path in schedule_files[:3]:  # Limit to 3 for the example
        output_path = output_dir / f"{schedule_path.stem}_processed.csv"
        
        # Try to find matching visibility file
        visibility_path = batch_dir / f"{schedule_path.stem}_visibility.json"
        if not visibility_path.exists():
            visibility_path = batch_dir / "possible_periods.json"
        
        try:
            print(f"\nProcessing {schedule_path.name}...")
            preprocess_schedule(
                schedule_path,
                visibility_path if visibility_path.exists() else None,
                output_path,
                validate=True
            )
            print(f"  ✓ Successful")
            successful += 1
        except Exception as e:
            print(f"  ✗ Error: {e}")
            failed += 1
    
    print(f"\n{'=' * 70}")
    print(f"Summary: {successful} successful, {failed} failed")
    print("\nFor production batch processing, use the CLI tool:")
    print("  python preprocess_schedules.py --batch-dir <dir> --output-dir <output>")


def example_reading_preprocessed():
    """Example: Read a pre-processed CSV in the app."""
    print("\n" + "=" * 70)
    print("EXAMPLE 4: Read a pre-processed CSV in the app")
    print("=" * 70)
    
    import pandas as pd
    
    csv_path = Path("data/schedule.csv")
    
    if not csv_path.exists():
        print(f"\n⚠️  CSV not found: {csv_path}")
        print("  Run one of the previous examples first, or use:")
        print("  python preprocess_schedules.py --schedule data/schedule.json --visibility data/possible_periods.json --output data/schedule.csv")
        return

    print(f"\nLoading {csv_path}...")
    df = pd.read_csv(csv_path)

    print(f"✓ Loaded successfully")
    print(f"\nDataFrame information:")
    print(f"  - Rows: {len(df)}")
    print(f"  - Columns: {len(df.columns)}")
    print(f"\nFirst columns:")
    for col in list(df.columns)[:10]:
        print(f"  - {col}")

    # Check if pre-processed
    preprocessed_indicators = ['scheduled_flag', 'requested_hours', 'elevation_range_deg', 'priority_bin']
    is_preprocessed = all(col in df.columns for col in preprocessed_indicators)
    
    if is_preprocessed:
        print(f"\n✓ This CSV is PRE-PROCESSED")
        print(f"  The app will load very quickly (light mode)")
    else:
        print(f"\n⚠ This CSV is NOT pre-processed")
        print(f"  The app will perform full processing (slower)")

    # Mostrar estadísticas rápidas
    if 'scheduled_flag' in df.columns:
        scheduled = df['scheduled_flag'].sum()
        print(f"\nStatistics:")
        print(f"  - Scheduled: {scheduled}")
        print(f"  - Unscheduled: {len(df) - scheduled}")


def main():
    """Run all examples."""
    print("\n" + "=" * 70)
    print("EXAMPLES OF USING THE SCHEDULE PREPROCESSOR")
    print("=" * 70)
    
    try:
        # Example 1: Full class usage with JSON files
        example_single_file()

        # Example 2: Convenience function
        example_convenience_function()
        
        # Example 3: Batch processing (commented out by default)
        # example_batch_processing()

        # Example 4: Read a pre-processed CSV
        example_reading_preprocessed()
        
        print("\n" + "=" * 70)
        print("EXAMPLES COMPLETED")
        print("=" * 70)
        print("\nFor more information, see:")
        print("  - doc/preprocessing_pipeline.md")
        print("  - PREPROCESS_SCHEDULES_README.md")
        print("\nFor CLI help:")
        print("  python preprocess_schedules.py --help")
        
    except ImportError as e:
        print(f"\n✗ Import error: {e}")
        print("\nMake sure you have the dependencies installed:")
        print("  pip install pandas numpy")
        print("\nOr install all project dependencies:")
        print("  pip install -r requirements.txt")
    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
