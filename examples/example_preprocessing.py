"""
Example usage of the Rust-backed schedule preprocessing helpers.

This script shows how to call :func:`core.loaders.load_schedule_from_json`
to convert raw scheduling exports into CSV files compatible with the Streamlit app.
"""

from __future__ import annotations

import io
import sys
from pathlib import Path

import pandas as pd


def get_project_root() -> Path:
    """Return the repository root (contains ``pyproject.toml``)."""

    for parent in Path(__file__).resolve().parents:
        if (parent / "pyproject.toml").exists():
            return parent
    raise RuntimeError("Could not locate project root")


PROJECT_ROOT = get_project_root()
SRC_PATH = PROJECT_ROOT / "src"
if str(SRC_PATH) not in sys.path:
    sys.path.insert(0, str(SRC_PATH))

from core.loaders import ScheduleLoadResult, load_schedule_from_json


def _print_validation_summary(result: ScheduleLoadResult) -> None:
    """Pretty-print the validation payload returned by Rust."""

    validation = result.validation
    print("\nValidation")
    print(f"  Status   : {'PASS' if validation.is_valid else 'FAIL'}")
    print(f"  Errors   : {len(validation.errors)}")
    if validation.errors:
        for error in validation.errors[:3]:
            print(f"    • {error}")
    print(f"  Warnings : {len(validation.warnings)}")
    if validation.warnings:
        for warning in validation.warnings[:3]:
            print(f"    • {warning}")

    if validation.stats:
        print("\nStats")
        for key, value in validation.stats.items():
            print(f"  - {key}: {value}")


def _export_csv(result: ScheduleLoadResult, output_path: Path) -> None:
    """Persist the dataframe as CSV, stringifying visibility lists."""

    df_export = result.dataframe.copy()
    if "visibility" in df_export.columns:
        df_export["visibility"] = df_export["visibility"].apply(str)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    df_export.to_csv(output_path, index=False)
    size_kb = output_path.stat().st_size / 1024
    print(f"\n✓ CSV saved to {output_path} ({size_kb:.1f} KB)")


def example_single_file() -> None:
    """Process schedule and visibility JSON files from disk."""

    print("=" * 70)
    print("EXAMPLE 1: Process JSON files with the Rust backend")
    print("=" * 70)

    schedule_path = Path("data/schedule.json")
    visibility_path = Path("data/possible_periods.json")
    output_path = Path("data/preprocessed/schedule_example.csv")

    if not schedule_path.exists():
        print(f"\n⚠️  Schedule file not found: {schedule_path}")
        return

    result = load_schedule_from_json(
        schedule_path,
        visibility_path if visibility_path.exists() else None,
    )

    print(f"\n✓ Loaded {len(result.dataframe)} rows with {len(result.dataframe.columns)} columns")
    _print_validation_summary(result)
    _export_csv(result, output_path)


def example_uploaded_files() -> None:
    """Simulate Streamlit uploads using in-memory buffers."""

    print("\n" + "=" * 70)
    print("EXAMPLE 2: Process uploaded JSON buffers")
    print("=" * 70)

    schedule_path = Path("data/schedule.json")
    if not schedule_path.exists():
        print(f"\n⚠️  Schedule file not found: {schedule_path}")
        return

    with open(schedule_path) as sf:
        schedule_buffer = io.StringIO(sf.read())
        schedule_buffer.name = "uploaded_schedule.json"

    visibility_buffer: io.StringIO | None = None
    visibility_path = Path("data/possible_periods.json")
    if visibility_path.exists():
        with open(visibility_path) as vf:
            visibility_buffer = io.StringIO(vf.read())
            visibility_buffer.name = "uploaded_visibility.json"

    result = load_schedule_from_json(schedule_buffer, visibility_buffer)
    print(f"\n✓ Uploaded files processed: {len(result.dataframe)} rows")
    _print_validation_summary(result)


def example_batch_processing() -> None:
    """Process multiple schedule files inside a directory."""

    print("\n" + "=" * 70)
    print("EXAMPLE 3: Batch preprocessing to CSV")
    print("=" * 70)

    batch_dir = Path("data/batch_schedules")
    output_dir = Path("data/preprocessed")

    if not batch_dir.exists():
        print(f"\n⚠️  Batch directory not found: {batch_dir}")
        print(
            "Use `python preprocess_schedules.py --batch-dir data/schedules --output-dir data/preprocessed` instead."
        )
        return

    schedule_files = sorted(batch_dir.glob("schedule*.json"))
    if not schedule_files:
        print(f"\n⚠️  No schedule files found in {batch_dir}")
        return

    output_dir.mkdir(parents=True, exist_ok=True)
    successful = 0
    failed = 0

    for schedule_file in schedule_files:
        visibility_file = batch_dir / f"{schedule_file.stem}_visibility.json"
        if not visibility_file.exists():
            visibility_file = batch_dir / "possible_periods.json"

        try:
            result = load_schedule_from_json(
                schedule_file,
                visibility_file if visibility_file.exists() else None,
            )
            csv_path = output_dir / f"{schedule_file.stem}_processed.csv"
            _export_csv(result, csv_path)
            successful += 1
        except Exception as exc:  # pragma: no cover - demo script
            failed += 1
            print(f"✗ Failed to process {schedule_file.name}: {exc}")

    print(f"\nSummary: {successful} successful, {failed} failed")


def example_reading_preprocessed() -> None:
    """Read an already preprocessed CSV to inspect the schema."""

    print("\n" + "=" * 70)
    print("EXAMPLE 4: Read a preprocessed CSV")
    print("=" * 70)

    csv_path = Path("data/schedule.csv")

    if not csv_path.exists():
        print(f"\n⚠️  CSV not found: {csv_path}")
        print("  Run one of the previous examples first or use:")
        print(
            "  python preprocess_schedules.py --schedule data/schedule.json "
            "--visibility data/possible_periods.json --output data/schedule.csv"
        )
        return

    print(f"\nLoading {csv_path}...")
    df = pd.read_csv(csv_path)

    print("✓ Loaded successfully")
    print(f"  Rows   : {len(df)}")
    print(f"  Columns: {len(df.columns)}")
    sample_cols = ', '.join(df.columns[:10])
    print(f"  First columns: {sample_cols}")

    indicators = ["scheduled_flag", "requested_hours", "elevation_range_deg", "priority_bin"]
    if all(col in df.columns for col in indicators):
        print("\n✓ This CSV is PRE-PROCESSED (the app will load instantly)")
    else:
        print("\n⚠ CSV missing derived columns (expect slower loading in the app)")


def main() -> None:
    """Run all examples."""

    print("\n" + "=" * 70)
    print("RUST PREPROCESSING EXAMPLES")
    print("=" * 70)

    try:
        example_single_file()
        example_uploaded_files()
        # example_batch_processing()  # Enable when batch directory is available
        example_reading_preprocessed()

        print("\nAll examples completed successfully.")
        print("For CLI usage see: python preprocess_schedules.py --help")
    except ImportError as exc:
        print(f"\n✗ Import error: {exc}")
        print("Install dependencies with: pip install -r requirements.txt")
    except Exception as exc:  # pragma: no cover - demo script
        print(f"\n✗ Unexpected error: {exc}")
        import traceback

        traceback.print_exc()


if __name__ == "__main__":
    main()
