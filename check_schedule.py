#!/usr/bin/env python3
"""Check schedule 1 status in the database."""
import tsi_rust

# Initialize database
tsi_rust.py_init_database()

# Check if schedule exists
try:
    schedules = tsi_rust.py_list_schedules()
    print(f"Total schedules in database: {len(schedules)}")
    print("\nSchedules:")
    for s in schedules:
        print(f"  ID={s['schedule_id']}, Name={s['schedule_name']}")
    
    if len(schedules) == 0:
        print("\n❌ No schedules found! You need to upload a schedule first.")
        exit(1)
    
    # Check schedule 1 specifically
    print("\n" + "="*60)
    print("Checking schedule_id=1...")
    
    try:
        schedule = tsi_rust.py_get_schedule(schedule_id=1, schedule_name=None)
        print(f"✓ Schedule found: {schedule.name}")
        print(f"  Blocks: {len(schedule.blocks)}")
        print(f"  Checksum: {schedule.checksum}")
    except Exception as e:
        print(f"❌ Schedule 1 not found: {e}")
        exit(1)
    
    # Check analytics
    print("\nChecking analytics for schedule_id=1...")
    has_analytics = tsi_rust.py_has_analytics_data(1)
    print(f"  Has analytics: {has_analytics}")
    
    if not has_analytics:
        print("\n⚠️  Analytics missing! Attempting to populate...")
        try:
            rows = tsi_rust.py_populate_analytics(1)
            print(f"✓ Populated {rows} analytics rows")
            
            # Verify
            has_analytics = tsi_rust.py_has_analytics_data(1)
            print(f"  Has analytics now: {has_analytics}")
            
            if has_analytics:
                print("\n✅ Analytics successfully populated!")
            else:
                print("\n❌ Analytics population failed!")
        except Exception as e:
            print(f"❌ Failed to populate analytics: {e}")
            import traceback
            traceback.print_exc()
    else:
        print("✅ Analytics already exist!")
        
except Exception as e:
    print(f"❌ Error: {e}")
    import traceback
    traceback.print_exc()
