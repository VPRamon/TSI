#!/usr/bin/env python3
"""Benchmark data loading performance."""

import sys
import time
from pathlib import Path

import pandas as pd
from tsi.services.preparation import prepare_dataframe


def get_project_root() -> Path:
    """Locate the repository root containing ``pyproject.toml``."""
    for parent in Path(__file__).resolve().parents:
        if (parent / "pyproject.toml").exists():
            return parent
    raise RuntimeError("Unable to locate project root")


# Add project src directory to path
sys.path.insert(0, str(get_project_root() / "src"))


def load_json(csv_path: Path) -> pd.DataFrame:
    """Simple CSV loader."""
    return pd.read_json(csv_path)


def benchmark_load():
    """Measure loading and preparation time."""
    csv_path = Path("data/schedule.json")

    print("🚀 Benchmarking Data Loading Performance")
    print("=" * 60)

    # Test 1: CSV reading
    print("\n1️⃣  Loading CSV from disk...")
    start = time.time()
    raw_df = load_json(csv_path)
    csv_time = time.time() - start
    print(f"   ✅ Loaded {len(raw_df):,} rows in {csv_time:.3f}s")

    # Test 2: Data preparation (type conversion + datetime parsing)
    print("\n2️⃣  Preparing dataframe (type conversion, datetime parsing)...")
    start = time.time()
    result = prepare_dataframe(raw_df)
    prep_time = time.time() - start
    print(f"   ✅ Prepared dataframe in {prep_time:.3f}s")

    if result.warnings:
        print(f"   ⚠️  Warnings: {len(result.warnings)}")
        for warning in result.warnings[:3]:
            print(f"      - {warning}")

    # Test 3: Total time
    total_time = csv_time + prep_time
    print(f"\n📊 Total Loading Time: {total_time:.3f}s")
    print(f"   - CSV reading: {csv_time:.3f}s ({csv_time/total_time*100:.1f}%)")
    print(f"   - Preparation: {prep_time:.3f}s ({prep_time/total_time*100:.1f}%)")

    # Test 4: Check visibility parsing is disabled
    print("\n3️⃣  Checking visibility_periods_parsed column...")
    if "visibility_periods_parsed" in result.dataframe.columns:
        non_null = result.dataframe["visibility_periods_parsed"].notna().sum()
        print(f"   ✅ Column exists with {non_null:,}/{len(result.dataframe):,} non-null values")
        if non_null == 0:
            print("   🎉 SUCCESS: Lazy loading enabled (0 rows parsed during initial load)")
    else:
        print("   ⚠️  Column missing")

    # Test 5: On-demand parsing sample
    print("\n4️⃣  Testing on-demand visibility parsing (10 rows)...")
    from tsi.services.preparation import parse_visibility_for_rows

    sample = result.dataframe.head(10)
    start = time.time()
    _ = parse_visibility_for_rows(sample)
    parse_time = time.time() - start
    print(f"   ✅ Parsed 10 rows in {parse_time:.4f}s ({parse_time*1000:.1f}ms)")
    print(f"   📈 Extrapolated full parse time: {parse_time * len(result.dataframe) / 10:.3f}s")

    # Performance target
    print("\n🎯 Performance Target: < 1.0s")
    if total_time < 1.0:
        print(f"   ✅ PASSED ({total_time:.3f}s)")
    elif total_time < 2.0:
        print(f"   ⚠️  ACCEPTABLE ({total_time:.3f}s)")
    else:
        print(f"   ❌ FAILED ({total_time:.3f}s)")

    return total_time


if __name__ == "__main__":
    benchmark_load()
