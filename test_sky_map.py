#!/usr/bin/env python3
"""Test script for the new sky map data endpoint."""

import os

# Set database credentials from the credentials file
try:
    import sys
    sys.path.insert(0, 'scripts')
    import db_credentials
    
    os.environ['DB_SERVER'] = db_credentials.server
    os.environ['DB_DATABASE'] = db_credentials.database
    os.environ['DB_USERNAME'] = db_credentials.username
    os.environ['DB_PASSWORD'] = db_credentials.password
    os.environ['DB_PORT'] = '1433'
    os.environ['DB_TRUST_CERT'] = 'true'
    os.environ['DB_AUTH_METHOD'] = 'aad_password'
    
    print("✅ Database credentials loaded")
except Exception as e:
    print(f"⚠️  Warning: Could not load db_credentials: {e}")
    print("   Make sure environment variables are set")

from tsi.services.database import get_sky_map_data, init_database

def test_sky_map_data():
    """Test the sky map data endpoint."""
    try:
        # Initialize database connection
        print("\nInitializing database connection...")
        init_database()
        print("✅ Database initialized")
        
        # Try to load sky map data for schedule 1
        print("\nLoading sky map data for schedule_id=1...")
        sky_map_data = get_sky_map_data(schedule_id=1)
        
        print(f"\n✅ Success! Loaded sky map data:")
        print(f"  - Type: {type(sky_map_data)}")
        print(f"  - Total blocks: {sky_map_data.total_count}")
        print(f"  - Scheduled blocks: {sky_map_data.scheduled_count}")
        print(f"  - Priority range: [{sky_map_data.priority_min:.2f}, {sky_map_data.priority_max:.2f}]")
        print(f"  - RA range: [{sky_map_data.ra_min:.2f}, {sky_map_data.ra_max:.2f}]")
        print(f"  - Dec range: [{sky_map_data.dec_min:.2f}, {sky_map_data.dec_max:.2f}]")
        print(f"  - Number of priority bins: {len(sky_map_data.priority_bins)}")
        
        print(f"\n📊 Priority bins:")
        for i, bin_info in enumerate(sky_map_data.priority_bins):
            print(f"  {i+1}. {bin_info.label}")
            print(f"     Range: [{bin_info.min_priority:.2f}, {bin_info.max_priority:.2f}]")
            print(f"     Color: {bin_info.color}")
        
        # Check a few blocks
        if sky_map_data.blocks:
            print(f"\n🔍 Sample blocks (first 3):")
            for i, block in enumerate(sky_map_data.blocks[:3]):
                print(f"  Block {i+1}:")
                print(f"    ID: {block.id.value}")
                print(f"    Priority: {block.priority:.2f}")
                print(f"    Priority Bin: {block.priority_bin}")
                print(f"    RA: {block.target_ra_deg:.2f}°")
                print(f"    Dec: {block.target_dec_deg:.2f}°")
                print(f"    Scheduled: {block.scheduled_period is not None}")
        
        return True
        
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_sky_map_data()
    exit(0 if success else 1)
