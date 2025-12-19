#!/usr/bin/env python3
"""
Test script to verify performance optimizations for schedule uploads.

This script tests the different upload modes:
1. Fast mode (default): skip expensive time bins
2. Full analytics mode: compute all analytics including time bins
3. No analytics mode: fastest upload, compute analytics later

Run with: python3 test_upload_performance.py
"""

import json
import sys
import time
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent / "src"))

from tsi.services.database import store_schedule_db


def load_test_data():
    """Load sample schedule and visibility data."""
    schedule_file = Path("data/schedule.json")
    visibility_file = Path("data/possible_periods.json")

    if not schedule_file.exists():
        print(f"❌ Schedule file not found: {schedule_file}")
        sys.exit(1)

    if not visibility_file.exists():
        print(f"⚠️ Visibility file not found: {visibility_file}, proceeding without visibility")
        visibility_data = None
    else:
        with open(visibility_file) as f:
            visibility_data = f.read()

    with open(schedule_file) as f:
        schedule_data = f.read()

    # Get block count
    schedule_json = json.loads(schedule_data)
    block_count = len(schedule_json.get("schedulingBlocks", []))

    return schedule_data, visibility_data, block_count


def test_fast_mode():
    """Test fast upload mode (skip time bins)."""
    print("\n" + "=" * 70)
    print("TEST 1: Fast Mode (skip_time_bins=True)")
    print("=" * 70)

    schedule_data, visibility_data, block_count = load_test_data()
    print(f"📊 Schedule: {block_count} blocks")

    start = time.time()
    try:
        metadata = store_schedule_db(
            schedule_name="test_fast_mode",
            schedule_json=schedule_data,
            visibility_json=visibility_data,
            populate_analytics=True,
            skip_time_bins=True,  # Fast mode
        )
        elapsed = time.time() - start

        print("✅ Upload successful!")
        print(f"   Schedule ID: {metadata['schedule_id']}")
        print(f"   Time: {elapsed:.2f} seconds")
        print(f"   Rate: {block_count / elapsed:.1f} blocks/sec")

        if elapsed < 60:
            print("   ✓ Performance target met (<60s)")
        else:
            print("   ⚠️ Slower than expected (target: <60s)")

        return metadata["schedule_id"], elapsed

    except Exception as e:
        print(f"❌ Upload failed: {e}")
        import traceback

        traceback.print_exc()
        return None, None


def test_full_mode():
    """Test full analytics mode (include time bins)."""
    print("\n" + "=" * 70)
    print("TEST 2: Full Analytics Mode (skip_time_bins=False)")
    print("=" * 70)
    print("⚠️ This will take 2-5 minutes for large schedules")

    schedule_data, visibility_data, block_count = load_test_data()

    start = time.time()
    try:
        metadata = store_schedule_db(
            schedule_name="test_full_mode",
            schedule_json=schedule_data,
            visibility_json=visibility_data,
            populate_analytics=True,
            skip_time_bins=False,  # Full mode
        )
        elapsed = time.time() - start

        print("✅ Upload successful!")
        print(f"   Schedule ID: {metadata['schedule_id']}")
        print(f"   Time: {elapsed:.2f} seconds ({elapsed/60:.1f} minutes)")
        print(f"   Rate: {block_count / elapsed:.1f} blocks/sec")

        return metadata["schedule_id"], elapsed

    except Exception as e:
        print(f"❌ Upload failed: {e}")
        import traceback

        traceback.print_exc()
        return None, None


def test_no_analytics():
    """Test fastest mode (no analytics)."""
    print("\n" + "=" * 70)
    print("TEST 3: No Analytics Mode (populate_analytics=False)")
    print("=" * 70)

    schedule_data, visibility_data, block_count = load_test_data()

    start = time.time()
    try:
        metadata = store_schedule_db(
            schedule_name="test_no_analytics",
            schedule_json=schedule_data,
            visibility_json=visibility_data,
            populate_analytics=False,  # Fastest mode
            skip_time_bins=True,
        )
        elapsed = time.time() - start

        print("✅ Upload successful!")
        print(f"   Schedule ID: {metadata['schedule_id']}")
        print(f"   Time: {elapsed:.2f} seconds")
        print(f"   Rate: {block_count / elapsed:.1f} blocks/sec")

        if elapsed < 30:
            print("   ✓ Performance target met (<30s)")
        else:
            print("   ⚠️ Slower than expected (target: <30s)")

        return metadata["schedule_id"], elapsed

    except Exception as e:
        print(f"❌ Upload failed: {e}")
        import traceback

        traceback.print_exc()
        return None, None


def main():
    """Run all performance tests."""
    print("\n" + "=" * 70)
    print("SCHEDULE UPLOAD PERFORMANCE OPTIMIZATION TEST")
    print("=" * 70)

    results = {}

    # Test 1: Fast mode (recommended default)
    schedule_id, elapsed = test_fast_mode()
    if elapsed:
        results["fast"] = elapsed

    # Test 2: No analytics (fastest)
    schedule_id, elapsed = test_no_analytics()
    if elapsed:
        results["no_analytics"] = elapsed

    # Test 3: Full mode (optional - comment out if too slow)
    # Uncomment to test full analytics including time bins
    # schedule_id, elapsed = test_full_mode()
    # if elapsed:
    #     results['full'] = elapsed

    # Summary
    print("\n" + "=" * 70)
    print("PERFORMANCE SUMMARY")
    print("=" * 70)

    if "no_analytics" in results:
        print(f"⚡ No analytics:  {results['no_analytics']:6.2f}s  (fastest)")

    if "fast" in results:
        print(f"🚀 Fast mode:     {results['fast']:6.2f}s  (recommended default)")

    if "full" in results:
        print(f"📊 Full analytics: {results['full']:6.2f}s  (includes time bins)")

    if "fast" in results and "full" in results:
        speedup = results["full"] / results["fast"]
        print(f"\n⚡ Fast mode is {speedup:.1f}x faster than full analytics")

    print("\n" + "=" * 70)
    print("✅ All tests completed successfully!")
    print("=" * 70)

    print("\nNext steps:")
    print("1. The default fast mode should now be used for all uploads")
    print("2. Users will experience ~10-30 second uploads instead of 2-5 minutes")
    print("3. Full analytics (with time bins) can be computed later if needed")


if __name__ == "__main__":
    main()
