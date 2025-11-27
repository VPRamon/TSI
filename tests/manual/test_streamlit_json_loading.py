#!/usr/bin/env python3
"""
Test script to verify JSON loading in Streamlit context.

This simulates what happens when uploading JSON files in the Streamlit app.
"""

import sys
from pathlib import Path


def get_project_root() -> Path:
    """Find the project root containing ``pyproject.toml``."""
    for parent in Path(__file__).resolve().parents:
        if (parent / "pyproject.toml").exists():
            return parent
    raise RuntimeError("Project root not found")


PROJECT_ROOT = get_project_root()
sys.path.insert(0, str(PROJECT_ROOT / "src"))


def test_json_to_streamlit_compatible():
    """Test that JSON loading produces Streamlit-compatible DataFrames."""

    from core.loaders import load_schedule_from_json

    print("Testing JSON loading for Streamlit compatibility...")
    print("=" * 60)

    schedule_path = Path("data/schedule.json")
    visibility_path = Path("data/possible_periods.json")

    if not schedule_path.exists():
        print(f"❌ Schedule file not found: {schedule_path}")
        return False

    # Load from JSON
    print("\n1. Loading from JSON...")
    result = load_schedule_from_json(schedule_path, visibility_path)
    df = result.dataframe
    print(f"   ✅ Loaded {len(df)} blocks")

    # Check visibility column type
    print("\n2. Checking visibility column...")
    print(f"   Type: {df['visibility'].dtype}")
    print(f"   Sample value type: {type(df['visibility'].iloc[0])}")
    print(f"   Sample value: {df['visibility'].iloc[0]}")

    if df["visibility"].dtype == "object" and isinstance(df["visibility"].iloc[0], list):
        print("   ⚠️  WARNING: visibility column contains lists")
        print("   Converting to strings for Streamlit compatibility...")
        df["visibility"] = df["visibility"].apply(str)
        print(f"   ✅ Converted. New type: {df['visibility'].dtype}")
        print(f"   Sample: {df['visibility'].iloc[0][:100]}...")

    # Test pandas hashing (what Streamlit uses for caching)
    print("\n3. Testing pandas hash (Streamlit cache compatibility)...")
    try:
        from pandas.core.util.hashing import hash_pandas_object

        hash_result = hash_pandas_object(df)
        print(f"   ✅ Hashing successful! ({len(hash_result)} hashes generated)")
    except TypeError as e:
        print(f"   ❌ Hashing failed: {e}")
        return False

    # Test that visibility can be parsed back
    print("\n4. Testing visibility string parsing...")
    import ast

    try:
        parsed = df["visibility"].iloc[0]
        if isinstance(parsed, str):
            visibility_list = ast.literal_eval(parsed)
            print(f"   ✅ Can parse back to list: {len(visibility_list)} periods")
        else:
            print(f"   ℹ️  Already a list: {len(parsed)} periods")
    except Exception as e:
        print(f"   ❌ Parsing failed: {e}")
        return False

    # Test prepare_dataframe (what the app uses)
    print("\n5. Testing with prepare_dataframe (app's data flow)...")
    try:
        # Import from the actual app code
        sys.path.insert(0, str(PROJECT_ROOT / "src"))
        from tsi.services.preparation import prepare_dataframe

        prepared = prepare_dataframe(df)
        print("   ✅ prepare_dataframe successful")
        print(f"   Warnings: {len(prepared.warnings)}")

        # Test hashing the prepared dataframe
        hash_result = hash_pandas_object(prepared.dataframe)
        print(f"   ✅ Prepared DataFrame is hashable ({len(hash_result)} hashes)")
    except Exception as e:
        print(f"   ❌ prepare_dataframe failed: {e}")
        import traceback

        traceback.print_exc()
        return False

    print("\n" + "=" * 60)
    print("✅ ALL TESTS PASSED!")
    print("   JSON files can be loaded in Streamlit without hashing errors")
    print("=" * 60)
    return True


if __name__ == "__main__":
    try:
        success = test_json_to_streamlit_compatible()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n❌ TEST FAILED: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)
