"""
Example usage of the Rust-backed schedule preprocessing helpers.

⚠️ LEGACY EXAMPLE - Uses deprecated API

This script demonstrates the old core.loaders API which has been replaced by
the Rust backend. For current usage, see:
- tsi_rust_api.py (TSIBackend class)
- Rust backend functions in tsi_rust module

This example is kept for reference but may not work with current codebase.
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

# LEGACY: core.loaders no longer exists - use tsi_rust_api.TSIBackend instead
# from core.loaders import ScheduleLoadResult, load_schedule_from_json
from tsi_rust_api import TSIBackend


def _print_validation_summary(result: dict) -> None:
    """Pretty-print the validation payload returned by Rust."""

    # Adapted for new API
    print("\nSchedule loaded successfully")
    if isinstance(result, dict):
        print(f"  Keys: {list(result.keys())}")


def _export_csv(df: pd.DataFrame, output_path: Path) -> None:
    """Persist the dataframe as CSV, stringifying visibility lists."""

    df_export = df.copy()
    if "visibility" in df_export.columns:
        df_export["visibility"] = df_export["visibility"].apply(str)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    df_export.to_csv(output_path, index=False)
    size_kb = output_path.stat().st_size / 1024
    print(f"\n✓ CSV saved to {output_path} ({size_kb:.1f} KB)")


def example_single_file() -> None:
    """Process schedule and visibility JSON files from disk."""

    print("=" * 70)
    print("EXAMPLE 1: Process JSON files with the Rust backend (NEW API)")
    print("=" * 70)

    schedule_path = Path("data/schedule.json")
    visibility_path = Path("data/possible_periods.json")
    output_path = Path("data/preprocessed/schedule_example.csv")

    if not schedule_path.exists():
        print(f"\n⚠️  Schedule file not found: {schedule_path}")
        return

    backend = TSIBackend()
    result_dict = backend.load_schedule(
        str(schedule_path),
        str(visibility_path) if visibility_path.exists() else None,
    )

    df = pd.DataFrame(result_dict.get("blocks", []))
    print(f"\n✓ Loaded {len(df)} rows with {len(df.columns)} columns")
    _print_validation_summary(result_dict)
    _export_csv(df, output_path)


def example_uploaded_files() -> None:
    """Simulate Streamlit uploads using in-memory buffers."""

    print("\n" + "=" * 70)
    print("EXAMPLE 2: Process uploaded JSON buffers (NEW API)")
    print("=" * 70)

    schedule_path = Path("data/schedule.json")
    if not schedule_path.exists():
        print(f"\n⚠️  Schedule file not found: {schedule_path}")
        return

    # For new API, read and pass as strings
    with open(schedule_path) as sf:
        schedule_json = sf.read()

    visibility_json: str | None = None
    visibility_path = Path("data/possible_periods.json")
    if visibility_path.exists():
        with open(visibility_path) as vf:
            visibility_json = vf.read()

    backend = TSIBackend()
    result_dict = backend.load_schedule_from_string(schedule_json, visibility_json)
    df = pd.DataFrame(result_dict.get("blocks", []))
    print(f"\n✓ Uploaded files processed: {len(df)} rows")
    _print_validation_summary(result_dict)


def example_batch_processing() -> None:
    """Process multiple schedule files inside a directory."""

    print("\n" + "=" * 70)
    print("EXAMPLE 3: Batch preprocessing to CSV (NEW API)")
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
    backend = TSIBackend()

    for schedule_file in schedule_files:
        visibility_file = batch_dir / f"{schedule_file.stem}_visibility.json"
        if not visibility_file.exists():
            visibility_file = batch_dir / "possible_periods.json"

        try:
            result_dict = backend.load_schedule(
                str(schedule_file),
                str(visibility_file) if visibility_file.exists() else None,
            )
            df = pd.DataFrame(result_dict.get("blocks", []))
            csv_path = output_dir / f"{schedule_file.stem}_processed.csv"
            _export_csv(df, csv_path)
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
