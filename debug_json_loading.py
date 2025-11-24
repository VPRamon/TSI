"""Debug script to test JSON parsing"""
import tsi_rust
from pathlib import Path

json_path = Path("data/schedule.json")

print(f"Loading from: {json_path}")
print(f"File exists: {json_path.exists()}")

try:
    df = tsi_rust.load_schedule_from_json(str(json_path))
    print(f"✅ Successfully loaded {len(df.to_pandas())} blocks")
except Exception as e:
    print(f"❌ Error: {e}")
    import traceback
    traceback.print_exc()
    
    # Try to load with Python to see what's different
    print("\n--- Trying with Python json module ---")
    import json
    with open(json_path) as f:
        data = json.load(f)
    
    print(f"Total blocks in JSON: {len(data['SchedulingBlock'])}")
    
    # Check first few blocks for missing fields
    for i, block in enumerate(data['SchedulingBlock'][:3]):
        print(f"\nBlock {i}:")
        print(f"  Keys: {list(block.keys())}")
        print(f"  Has scheduled_period: {'scheduled_period' in block}")
        if 'scheduled_period' in block:
            print(f"  scheduled_period keys: {list(block['scheduled_period'].keys())}")
