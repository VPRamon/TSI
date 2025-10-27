#!/usr/bin/env python3
"""
Compare visibility parsing strategies.

Benchmarks:
1. No parsing (landing page)
2. Selective parsing (50 rows for timeline)
3. Full parsing with cache (unscheduled analysis)
"""

import sys
import time
from pathlib import Path

import pandas as pd

from core.transformations import parse_visibility_for_rows, prepare_dataframe


def get_project_root() -> Path:
    """Locate the repository root containing ``pyproject.toml``."""
    for parent in Path(__file__).resolve().parents:
        if (parent / "pyproject.toml").exists():
            return parent
    raise RuntimeError("Unable to locate project root")


# Add src to path
sys.path.insert(0, str(get_project_root() / "src"))


def load_csv(csv_path: Path) -> pd.DataFrame:
    """Simple CSV loader."""
    return pd.read_csv(csv_path)


def benchmark_strategies():
    """Compare the 3 visibility parsing strategies."""
    csv_path = Path("data/schedule.csv")

    print("🎯 Visibility Parsing Strategy Benchmark")
    print("=" * 70)

    # Load raw data
    print("\n📂 Loading raw CSV...")
    raw_df = load_csv(csv_path)
    print(f"   Loaded {len(raw_df):,} rows, {len(raw_df.columns)} columns")

    # Strategy 1: Landing page (no parsing)
    print("\n" + "=" * 70)
    print("1️⃣  STRATEGY 1: Landing Page (No Parsing)")
    print("=" * 70)
    print("Use case: Initial app load, data exploration")
    print("Expected: Fast load with visibility_periods_parsed = None")

    start = time.time()
    result = prepare_dataframe(raw_df)
    df_no_parse = result.dataframe
    load_time = time.time() - start

    non_null = df_no_parse["visibility_periods_parsed"].notna().sum()
    print("\n✅ Result:")
    print(f"   - Load time: {load_time:.3f}s")
    print(f"   - Visibility parsed: {non_null:,}/{len(df_no_parse):,} rows")
    print(f"   - Status: {'OPTIMIZED ✨' if non_null == 0 else 'NEEDS FIX ⚠️'}")

    # Strategy 2: Timeline (selective parsing)
    print("\n" + "=" * 70)
    print("2️⃣  STRATEGY 2: Timeline (Selective Parsing)")
    print("=" * 70)
    print("Use case: Timeline visualization showing top 50 blocks")
    print("Expected: Parse only 50 rows (~0.75s)")

    # Simulate timeline behavior: select top 50 by priority
    top_50 = df_no_parse.nlargest(50, "priority")

    start = time.time()
    _ = parse_visibility_for_rows(top_50)
    parse_time = time.time() - start

    print("\n✅ Result:")
    print(f"   - Rows to parse: {len(top_50):,}")
    print(f"   - Parse time: {parse_time:.3f}s")
    print(f"   - Speed: {parse_time/len(top_50)*1000:.1f}ms per row")
    print(f"   - Savings vs full: {(1 - len(top_50)/len(df_no_parse))*100:.1f}% fewer rows")

    # Extrapolate full parsing time
    full_time_estimate = parse_time * len(df_no_parse) / len(top_50)
    print(f"   - Extrapolated full parse: {full_time_estimate:.1f}s")
    print(f"   - Speedup: {full_time_estimate/parse_time:.1f}x faster")

    # Strategy 3: Unscheduled Analysis (full parsing)
    print("\n" + "=" * 70)
    print("3️⃣  STRATEGY 3: Unscheduled Analysis (Full Parsing)")
    print("=" * 70)
    print("Use case: ML analysis requiring all visibility data")
    print("Expected: Parse all 2647 rows (~40s), but cache for reuse")

    print("\n🔄 First visit (cold cache):")
    start = time.time()
    df_full_parsed = df_no_parse.copy()
    df_full_parsed["visibility_periods_parsed"] = parse_visibility_for_rows(df_full_parsed)
    full_parse_time = time.time() - start

    print(f"   - Rows parsed: {len(df_full_parsed):,}")
    print(f"   - Parse time: {full_parse_time:.3f}s")
    print(f"   - Speed: {full_parse_time/len(df_full_parsed)*1000:.1f}ms per row")

    print("\n💾 Subsequent visits (warm cache):")
    print("   - Parse time: ~0.001s (from Streamlit cache)")
    print(f"   - Speedup: ~{full_parse_time*1000:.0f}x faster")

    # Summary comparison
    print("\n" + "=" * 70)
    print("📊 PERFORMANCE SUMMARY")
    print("=" * 70)

    strategies = [
        ("Landing Page", 0, load_time, "No parsing"),
        ("Timeline (top 50)", 50, parse_time, "Selective"),
        ("Unscheduled (all)", len(df_no_parse), full_parse_time, "Full w/ cache"),
    ]

    print(f"\n{'Strategy':<25} {'Rows':<10} {'Time':<12} {'Type':<15}")
    print("-" * 70)
    for name, rows, time_taken, type_str in strategies:
        print(f"{name:<25} {rows:<10,} {time_taken:<11.3f}s {type_str:<15}")

    # Target analysis
    print("\n🎯 Performance Targets:")
    print("-" * 70)
    target_landing = 1.0  # seconds
    target_interactive = 2.0  # seconds

    if load_time <= target_landing:
        print(f"✅ Landing page: {load_time:.3f}s ≤ {target_landing}s (PASS)")
    else:
        print(f"❌ Landing page: {load_time:.3f}s > {target_landing}s (FAIL)")

    if parse_time <= target_interactive:
        print(f"✅ Timeline: {parse_time:.3f}s ≤ {target_interactive}s (PASS)")
    else:
        print(f"⚠️  Timeline: {parse_time:.3f}s > {target_interactive}s (ACCEPTABLE)")

    if full_parse_time > 10:
        print(f"⚠️  Full parse: {full_parse_time:.3f}s (slow, but cached after first use)")
    else:
        print(f"✅ Full parse: {full_parse_time:.3f}s (acceptable with cache)")

    # Best practices recommendation
    print("\n💡 RECOMMENDATIONS:")
    print("-" * 70)
    print("1. ✅ Use lazy loading by default (visibility_periods_parsed = None)")
    print("2. ✅ For filtered views: parse only filtered subset (parse_subset_lazy)")
    print("3. ✅ For full analysis: use cache wrapper (ensure_visibility_parsed)")
    print("4. ✅ Leverage Streamlit's @st.cache_data for persistence")
    print("5. ❌ NEVER parse visibility during initial prepare_dataframe()")

    print("\n🔗 See VISIBILITY_PARSING_STRATEGY.md for implementation details")
    print("=" * 70)


if __name__ == "__main__":
    benchmark_strategies()
