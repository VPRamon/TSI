#!/usr/bin/env python3
"""
Test script to verify that analytics are populated when uploading schedules.
"""
import os
import sys

import pytest

# Add the src directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "src"))

import tsi_rust


def test_analytics_population():
    """Test that analytics are populated after schedule upload."""
    pytest.skip(
        "API changed: use tsi.services.upload_schedule; this test requires backend database access."
    )

    # No explicit database init required; Rust backend lazy-initializes repository
    print("Rust backend module imported; repository will initialize on first use")

    # Read a test schedule
    schedule_path = "data/schedule.json"
    visibility_path = "data/possible_periods.json"

    print("\nReading test files...")
    with open(schedule_path) as f:
        schedule_json = f.read()

    with open(visibility_path) as f:
        visibility_json = f.read()

    # Upload the schedule
    print("\n1. Uploading schedule...")
    metadata = tsi_rust.py_store_schedule("Analytics Test Schedule", schedule_json, visibility_json)

    schedule_id = metadata["schedule_id"]
    print(f"   ✓ Schedule uploaded: ID={schedule_id}")

    # Try to fetch sky map analytics data (Rust will populate analytics as needed)
    print(f"\n2. Retrieving sky map analytics for schedule_id={schedule_id}...")
    try:
        sky_map = tsi_rust.py_get_sky_map_data_analytics(schedule_id)
        print("   ✓ Sky map data retrieved successfully")
        print(f"   - Total blocks: {len(sky_map['blocks'])}")
        print("\n✅ TEST PASSED: Analytics are populated and accessible!")
        return True
    except Exception as e:
        print(f"\n❌ TEST FAILED: {str(e)}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_analytics_population()
    sys.exit(0 if success else 1)
