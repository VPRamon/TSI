#!/usr/bin/env python3
"""Test script for schedule timeline backend functionality."""

import sys
sys.path.insert(0, '/workspace/src')

from tsi.services.database import init_database, get_schedule_timeline_data

def test_schedule_timeline():
    """Test the schedule timeline data endpoint."""
    try:
        # Initialize database connection
        print("\nInitializing database connection...")
        init_database()
        print("✅ Database initialized")
        
        # Try to load timeline data for schedule 1
        print("\nLoading schedule timeline data for schedule_id=1...")
        timeline_data = get_schedule_timeline_data(schedule_id=1)
        
        print(f"\n✅ Success! Loaded schedule timeline data:")
        print(f"  - Type: {type(timeline_data)}")
        print(f"  - Total blocks: {timeline_data.total_count}")
        print(f"  - Scheduled blocks: {timeline_data.scheduled_count}")
        print(f"  - Priority range: [{timeline_data.priority_min:.2f}, {timeline_data.priority_max:.2f}]")
        print(f"  - Unique months: {len(timeline_data.unique_months)}")
        print(f"  - Dark periods: {len(timeline_data.dark_periods)}")
        
        if timeline_data.unique_months:
            print(f"\n  First few months: {timeline_data.unique_months[:5]}")
        
        if timeline_data.blocks:
            block = timeline_data.blocks[0]
            print(f"\n  First block example:")
            print(f"    - ID: {block.scheduling_block_id}")
            print(f"    - Priority: {block.priority:.2f}")
            print(f"    - Start MJD: {block.scheduled_start_mjd:.2f}")
            print(f"    - Stop MJD: {block.scheduled_stop_mjd:.2f}")
            print(f"    - RA: {block.ra_deg:.2f}°")
            print(f"    - Dec: {block.dec_deg:.2f}°")
            print(f"    - Requested hours: {block.requested_hours:.2f}")
            print(f"    - Visibility hours: {block.total_visibility_hours:.2f}")
            print(f"    - Visibility periods: {block.num_visibility_periods}")
        
        print("\n✅ All tests passed!")
        return 0
        
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(test_schedule_timeline())
