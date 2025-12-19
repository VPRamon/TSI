#!/usr/bin/env python3
"""
Test script to verify that analytics are populated when uploading schedules.
"""
import os
import sys

# Add the src directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "src"))

import tsi_rust


def test_analytics_population():
    """Test that analytics are populated after schedule upload."""

    # Initialize database connection
    print("Initializing database connection...")
    try:
        tsi_rust.py_init_database()
        print("   ✓ Database connection initialized")
    except Exception as e:
        print(f"   ❌ Failed to initialize database: {e}")
        return False

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

    # Check if analytics exist
    print(f"\n2. Checking analytics for schedule_id={schedule_id}...")
    try:
        has_analytics = tsi_rust.py_has_analytics_data(schedule_id)

        if has_analytics:
            print("   ✓ Analytics data found!")

            # Try to get sky map data
            print("\n3. Testing sky map data retrieval...")
            sky_map = tsi_rust.py_get_sky_map_data_analytics(schedule_id)
            print("   ✓ Sky map data retrieved successfully")
            print(f"   - Total blocks: {len(sky_map['blocks'])}")

            print("\n✅ TEST PASSED: Analytics are populated correctly!")
            return True
        else:
            print("   ❌ No analytics data found!")
            print("\n❌ TEST FAILED: Analytics were not populated")
            return False

    except Exception as e:
        print(f"\n❌ TEST FAILED: {str(e)}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_analytics_population()
    sys.exit(0 if success else 1)
